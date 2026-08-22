use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use colored::Colorize;
use core::viewmodel::capabilities::Capabilities;
use core::viewmodel::message::Message;
use core::viewmodel::task_viewmodel::TaskViewModel;
use core::viewmodel::BakeViewModel;
use serde_json::json;

use crate::capabilities::{SHELL, SWITCH};

/// a task run in progress (shared between the worker thread and http handlers)
struct RunEntry {
    id: u64,
    name: String,
    started_at: Instant,
    abort: AtomicBool,
    /// pid of the currently running command (process group leader)
    active_child: Mutex<Option<u32>>,
}

impl RunEntry {
    fn aborted(&self) -> bool {
        self.abort.load(Ordering::SeqCst)
    }

    fn set_active_pid(&self, pid: Option<u32>) {
        *self.active_child.lock().unwrap() = pid;
    }

    fn elapsed_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }
}

static RUNS: OnceLock<Mutex<Vec<Arc<RunEntry>>>> = OnceLock::new();
static NEXT_RUN_ID: AtomicU64 = AtomicU64::new(1);

fn runs() -> &'static Mutex<Vec<Arc<RunEntry>>> {
    RUNS.get_or_init(Mutex::default)
}

fn next_run_id() -> u64 {
    NEXT_RUN_ID.fetch_add(1, Ordering::SeqCst)
}

/// kills the process group of a running task
/// returns false if there is no such running task
fn kill_run(id: u64) -> bool {
    let entry = {
        let runs = runs().lock().unwrap();
        match runs.iter().find(|x| x.id == id) {
            Some(x) => Arc::clone(x),
            None => return false,
        }
    };
    entry.abort.store(true, Ordering::SeqCst);
    if let Some(pid) = entry.active_child.lock().unwrap().take() {
        kill_process_tree(pid);
    }
    true
}

#[cfg(unix)]
fn kill_process_tree(pid: u32) {
    // commands are spawned with process_group(0) so -pid kills the whole tree
    unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
    }
}

#[cfg(windows)]
fn kill_process_tree(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/F", "/T", "/PID"])
        .arg(pid.to_string())
        .spawn();
}

// ---------- auth ----------

/// password read once at first request (empty/unset means no protection)
static PASSWORD: OnceLock<Option<String>> = OnceLock::new();

fn server_password() -> Option<&'static str> {
    PASSWORD
        .get_or_init(|| {
            std::env::var("BAKE_PASSWORD")
                .ok()
                .filter(|x| !x.is_empty())
        })
        .as_deref()
}

/// http basic auth check, the header value is everything after 'Basic '
fn basic_auth_matches(header_value: &str, password: &str) -> bool {
    let value = header_value.trim();
    // drop the 'Basic ' scheme prefix (case-insensitive per rfc)
    let encoded = value
        .strip_prefix("Basic ")
        .or_else(|| value.strip_prefix("basic "))
        .unwrap_or(value);
    let decoded = match base64_decode(encoded) {
        Some(x) => x,
        None => return false,
    };
    // user name is ignored, only password matters ('user:password')
    let given = String::from_utf8_lossy(&decoded);
    let given = given.split_once(':').map(|(_, p)| p).unwrap_or(&given);
    constant_time_eq(given, password)
}

fn authorized(req: &Req) -> bool {
    match server_password() {
        None => true,
        Some(password) => req.headers.iter().any(|(name, value)| {
            name.eq_ignore_ascii_case("authorization") && basic_auth_matches(value, password)
        }),
    }
}

/// small standard base64 decoder (no dependency)
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    fn val(b: u8) -> Option<u32> {
        match b {
            b'A'..=b'Z' => Some((b - b'A') as u32),
            b'a'..=b'z' => Some((b - b'a') as u32 + 26),
            b'0'..=b'9' => Some((b - b'0') as u32 + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let input: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if !input.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    for chunk in input.chunks(4) {
        let mut acc: u32 = 0;
        let mut n = 0usize;
        for &b in chunk {
            if b == b'=' {
                break;
            }
            acc = (acc << 6) | val(b)?;
            n += 1;
        }
        // rest must be padding
        if chunk[n..].iter().any(|&b| b != b'=') {
            return None;
        }
        match n {
            2 => out.push((acc >> 4) as u8),
            3 => {
                out.push((acc >> 10) as u8);
                out.push((acc >> 2) as u8);
            }
            4 => {
                out.push((acc >> 16) as u8);
                out.push((acc >> 8) as u8);
                out.push(acc as u8);
            }
            _ => return None,
        }
    }
    Some(out)
}

/// compares without early exit so timing leaks nothing
fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = a.len() ^ b.len();
    for i in 0..a.len().min(b.len()) {
        diff |= (a[i] ^ b[i]) as usize;
    }
    diff == 0
}

/// [Capabilities] implementation for `bake serve`
/// instead of printing to stdout it streams every message/command output
/// into a channel which is consumed by the http response body
struct ServeCapabilities {
    tx: Sender<Vec<u8>>,
    /// present while this instance runs a task (None for read only usage)
    run: Option<Arc<RunEntry>>,
}

impl ServeCapabilities {
    fn aborted(&self) -> bool {
        self.run.as_ref().is_some_and(|r| r.aborted())
    }

    fn set_active_pid(&self, pid: Option<u32>) {
        if let Some(run) = &self.run {
            run.set_active_pid(pid);
        }
    }
}

impl Capabilities for ServeCapabilities {
    fn read_file(&self, file_name: &str) -> Option<String> {
        std::fs::read_to_string(file_name).ok()
    }

    fn execute(&self, command: &str, working_directory: Option<&str>) -> bool {
        if self.aborted() {
            self.message(Message::warning("abort requested, stopping\n"));
            return false;
        }

        self.message(Message::bake_state(format!(
            "running command => '{}'\n",
            command
        )));

        let cwd = working_directory.unwrap_or(".");
        let cwd = match std::fs::canonicalize(cwd) {
            Ok(x) => x,
            Err(e) => {
                self.message(Message::error(format!(
                    "{} (working_directory: {})\n",
                    e,
                    working_directory.unwrap_or(".")
                )));
                return false;
            }
        };

        let mut cmd = Command::new(SHELL);
        cmd.arg(SWITCH)
            .arg(command)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // own process group so we can kill the whole tree on abort
        #[cfg(unix)]
        cmd.process_group(0);

        let child = match cmd.spawn() {
            Ok(x) => x,
            Err(e) => {
                self.message(Message::error(format!("can't run '{}': {}\n", command, e)));
                return false;
            }
        };

        let pid = child.id();
        self.set_active_pid(Some(pid));

        // stdout is forwarded here, stderr in a separate thread
        let mut child = child;
        if let Some(stdout) = child.stdout.take() {
            forward_lines(stdout, &self.tx);
        }
        if let Some(stderr) = child.stderr.take() {
            let tx = self.tx.clone();
            thread::spawn(move || forward_lines(stderr, &tx));
        }

        let success = match child.wait() {
            Ok(status) => status.success(),
            Err(_) => false,
        };
        self.set_active_pid(None);
        success
    }

    fn open_link(&self, url: &str) {
        self.message(Message::normal(format!("open link: '{}'\n", url)));
    }

    fn message(&self, input: Message) {
        let mut content = input.content().to_string();
        if !content.ends_with('\n') {
            content.push('\n');
        }
        let _ = self.tx.send(content.into_bytes());
    }

    /// serve mode is non-interactive (inputs come from the request)
    fn input(&self) -> Option<String> {
        None
    }

    fn should_abort(&self) -> bool {
        self.aborted()
    }

    fn set_env(&self, name: &str, value: &str) {
        std::env::set_var(name, value);
    }

    fn get_env(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    fn remove_env(&self, name: &str) {
        std::env::remove_var(name)
    }
}

fn forward_lines<R: Read>(reader: R, tx: &Sender<Vec<u8>>) {
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    while matches!(reader.read_line(&mut line), Ok(n) if n > 0) {
        line.push('\n');
        let _ = tx.send(line.clone().into_bytes());
        line.clear();
    }
}

pub fn start_server(port: u16) {
    let addr = format!("127.0.0.1:{port}");
    let listener = match TcpListener::bind(&addr) {
        Ok(x) => x,
        Err(e) => {
            println!("can't start server on {addr}: {e}");
            std::process::exit(1);
        }
    };
    println!(
        " {} \n web app & api: http://localhost:{} (ctrl+c to stop)\n",
        " ▶ Bake serve ".on_bright_yellow().black(),
        port
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_conn(stream));
            }
            Err(e) => println!("bad connection: {e}"),
        }
    }
}

/// parsed http request (only what bake needs)
struct Req {
    method: String,
    path: String,
    query: Option<String>,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

const MAX_HEAD: usize = 64 * 1024;
const MAX_BODY: usize = 64 * 1024;

fn handle_conn(mut stream: TcpStream) {
    // don't hold threads forever on silent connections
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));

    let req = match read_request(&mut stream) {
        Ok(Some(x)) => x,
        Ok(None) => return,
        Err(e) => {
            let body = format!("bad request: {e}");
            let _ = write_full(&mut stream, 400, "text/plain", &body, "");
            return;
        }
    };

    // BAKE_PASSWORD env protects every route (browser shows a login prompt)
    if !authorized(&req) {
        let _ = write_full(
            &mut stream,
            401,
            "text/plain",
            "password required\n",
            "WWW-Authenticate: Basic realm=\"bake\"\r\n",
        );
        return;
    }

    match route(&req) {
        Resp::Full(status, content_type, body) => {
            let _ = write_full(&mut stream, status, content_type, &body, "");
        }
        Resp::Stream { rx, run_id } => {
            let head = format!(
                "HTTP/1.1 200 OK\r\nX-Run-Id: {run_id}\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
            );
            if stream.write_all(head.as_bytes()).is_ok() && write_stream(&mut stream, rx).is_ok() {}
        }
    }
    let _ = stream.flush();
}

/// reads head (+ body by Content-Length) of a http request
fn read_request(stream: &mut TcpStream) -> std::io::Result<Option<Req>> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        if buf.len() > MAX_HEAD {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "request head too large",
            ));
        }
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Ok(None); // connection closed before full request
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            return parse_request(stream, &buf[..pos], &buf[pos + 4..]);
        }
    }
}

fn parse_request(
    stream: &mut TcpStream,
    head: &[u8],
    body_start: &[u8],
) -> std::io::Result<Option<Req>> {
    let head = String::from_utf8_lossy(head);
    let mut lines = head.split("\r\n");
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_uppercase();
    let target = parts.next().unwrap_or_default().to_string();
    if method.is_empty() || target.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "malformed request line",
        ));
    }

    let mut content_length = 0usize;
    let mut expect_continue = false;
    let mut headers = Vec::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            let name = name.trim().to_string();
            let value = value.trim().to_string();
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse().unwrap_or(0);
            }
            if name.eq_ignore_ascii_case("expect") && value.eq_ignore_ascii_case("100-continue") {
                expect_continue = true;
            }
            headers.push((name, value));
        }
    }

    if content_length > MAX_BODY {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "request body too large",
        ));
    }

    // clients like curl wait for this before sending big bodies
    if expect_continue {
        stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n")?;
        stream.flush()?;
    }

    let mut body = body_start.to_vec();
    while body.len() < content_length {
        let mut chunk = [0u8; 4096];
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(content_length);

    let (path_raw, query) = match target.split_once('?') {
        Some((p, q)) => (p.to_string(), Some(q.to_string())),
        None => (target, None),
    };
    let path = if path_raw == "/" {
        "/".to_string()
    } else {
        percent_decode(path_raw.trim_matches('/'), false)
    };

    Ok(Some(Req {
        method,
        path,
        query,
        headers,
        body,
    }))
}

/// non streaming answer
fn write_full(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
    extra_headers: &str,
) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {status} {}\r\n{extra_headers}Content-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reason(status),
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body.as_bytes())?;
    stream.flush()
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

/// writes task output as chunked transfer encoding, live (flush per chunk)
fn write_stream(stream: &mut TcpStream, rx: Receiver<Vec<u8>>) -> std::io::Result<()> {
    while let Ok(chunk) = rx.recv() {
        if chunk.is_empty() {
            continue;
        }
        stream.write_all(format!("{:x}\r\n", chunk.len()).as_bytes())?;
        stream.write_all(&chunk)?;
        stream.write_all(b"\r\n")?;
        stream.flush()?; // push it out immediately so the user sees live logs
    }
    stream.write_all(b"0\r\n\r\n")?;
    stream.flush()
}

enum Resp {
    Full(u16, &'static str, String),
    /// run output streamed live to the client
    Stream {
        rx: Receiver<Vec<u8>>,
        run_id: u64,
    },
}

fn route(req: &Req) -> Resp {
    let html = wants_html(&req.headers);
    let bake = match BakeViewModel::new(probe_caps()) {
        Ok(x) => x,
        Err(e) => return Resp::Full(500, "text/plain", e),
    };

    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => {
            if html {
                Resp::Full(200, "text/html", index_page(&bake))
            } else {
                tasks_json(&bake)
            }
        }
        ("GET", "running") => running_json(),
        ("DELETE", p) => match p
            .strip_prefix("running/")
            .and_then(|x| x.parse::<u64>().ok())
        {
            Some(id) => {
                if kill_run(id) {
                    Resp::Full(200, "text/plain", "killed".to_string())
                } else {
                    Resp::Full(404, "text/plain", format!("no running task with id {id}"))
                }
            }
            None => Resp::Full(400, "text/plain", "expected /running/{id}".to_string()),
        },
        ("GET", name) => match bake.get_task(name) {
            Some(task) => {
                if html {
                    Resp::Full(200, "text/html", task_page(task))
                } else {
                    task_json_response(task)
                }
            }
            None => Resp::Full(404, "text/plain", format!("task '{name}' not found")),
        },
        ("POST", name) => {
            if bake.get_task(name).is_none() {
                return Resp::Full(404, "text/plain", format!("task '{name}' not found"));
            }
            // params can come from the query string and/or form body
            let mut params = req
                .query
                .as_deref()
                .map(parse_urlencoded)
                .unwrap_or_default();
            params.extend(parse_urlencoded(&String::from_utf8_lossy(&req.body)));
            // this entry makes the run visible in GET /running and killable
            // (created here so its id can be returned in the X-Run-Id header)
            let entry = Arc::new(RunEntry {
                id: next_run_id(),
                name: name.to_string(),
                started_at: Instant::now(),
                abort: AtomicBool::new(false),
                active_child: Mutex::new(None),
            });
            runs().lock().unwrap().push(Arc::clone(&entry));
            let (tx, rx) = mpsc::channel::<Vec<u8>>();
            let name = name.to_string();
            let run_id = entry.id;
            thread::spawn(move || run_task_streaming(name, params, tx, entry));
            Resp::Stream { rx, run_id }
        }
        _ => Resp::Full(405, "text/plain", "method not allowed".to_string()),
    }
}

/// bakefile view model for read only checks (messages are discarded)
fn probe_caps() -> Rc<dyn Capabilities> {
    let (tx, _) = mpsc::channel();
    Rc::new(ServeCapabilities { tx, run: None })
}

/// runs the task and streams everything (bake messages + command output) to the client
fn run_task_streaming(
    task_name: String,
    params: Vec<(String, String)>,
    tx: Sender<Vec<u8>>,
    entry: Arc<RunEntry>,
) {
    let serve_cap = Rc::new(ServeCapabilities {
        tx,
        run: Some(Arc::clone(&entry)),
    });
    let cap: Rc<dyn Capabilities> = serve_cap.clone();

    let bake = match BakeViewModel::new(Rc::clone(&cap)) {
        Ok(x) => x,
        Err(e) => {
            serve_cap.message(Message::error(format!("{e}\n")));
            runs().lock().unwrap().retain(|x| x.id != entry.id);
            return;
        }
    };

    // each request provides its own values so stale envs from previous runs must go
    if let Some(task) = bake.get_task(&task_name) {
        for param in task.params() {
            cap.remove_env(param.name());
        }
    }
    for (name, value) in &params {
        cap.set_env(name, value);
    }

    serve_cap.message(Message::bake_state(format!("run id: {}\n", entry.id)));

    let result = bake.run_task(&task_name);

    runs().lock().unwrap().retain(|x| x.id != entry.id);

    if let Err(e) = result {
        if entry.aborted() {
            serve_cap.message(Message::warning(format!("Task '{task_name}' killed\n")));
        } else {
            serve_cap.message(Message::error(format!("{e}\n")));
            serve_cap.message(Message::error(format!(
                "Task '{task_name}' failed to run\n"
            )));
        }
    }
}

// ---------- json api ----------

/// list of currently running tasks (GET /running)
fn running_json() -> Resp {
    let running: Vec<serde_json::Value> = runs()
        .lock()
        .unwrap()
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "name": r.name,
                "elapsed_ms": r.elapsed_ms(),
            })
        })
        .collect();
    response(
        200,
        "application/json",
        serde_json::to_string_pretty(&json!({ "running": running })).unwrap_or_default(),
    )
}

fn tasks_json(bake: &BakeViewModel) -> Resp {
    let tasks: Vec<serde_json::Value> = bake.tasks().iter().map(task_json).collect();
    response(
        200,
        "application/json",
        serde_json::to_string_pretty(&json!({ "tasks": tasks })).unwrap_or_default(),
    )
}

fn task_json_response(task: &TaskViewModel) -> Resp {
    response(
        200,
        "application/json",
        serde_json::to_string_pretty(&task_json(task)).unwrap_or_default(),
    )
}

fn task_json(task: &TaskViewModel) -> serde_json::Value {
    json!({
        "name": task.name(),
        "help_msg": task.help_msg(),
        "dependencies": task.dependencies(),
        "params": task.params().iter().map(|p| json!({
            "name": p.name(),
            "default": p.default(),
        })).collect::<Vec<_>>(),
        "working_directory": task.working_directory(),
        "keep_alive": task.keep_alive(),
        "commands": match task.commands() {
            Ok(cmds) => json!(cmds.iter().map(command_string).collect::<Vec<_>>()),
            Err(e) => json!({ "error": e }),
        },
    })
}

fn command_string(command: &core::model::command::Command) -> String {
    use core::model::command::Command;
    match command {
        Command::ShellCommand(cmd) => cmd.clone(),
        Command::FunctionCall(fc) => {
            let mut s = format!("@{}.{}", fc.namespace(), fc.function());
            for (k, v) in fc.args() {
                s.push_str(&format!(" --{k} {v}"));
            }
            s
        }
    }
}

// ---------- web app ----------

fn wants_html(headers: &[(String, String)]) -> bool {
    headers.iter().any(|(name, value)| {
        name.eq_ignore_ascii_case("accept") && value.to_lowercase().contains("text/html")
    })
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const STYLE: &str = "
body{font-family:system-ui,sans-serif;max-width:720px;margin:2rem auto;padding:0 1rem;background:#141414;color:#eee}
a{color:#ffd54f;text-decoration:none;font-weight:600;font-size:1.05rem}
ul{padding:0}li{margin:.6rem 0;list-style:none;padding:.7rem 1rem;border:1px solid #333;border-radius:8px}
.help{display:block;color:#9e9e9e;font-size:.85rem;margin-top:.25rem}
label{display:block;margin:.5rem 0;color:#bbb;font-size:.85rem}
input,button{font-size:1rem;padding:.45rem .7rem;border-radius:6px;border:1px solid #444;background:#1c1c1c;color:#eee;width:100%;box-sizing:border-box;margin-top:.2rem}
button{background:#ffd54f;color:#000;font-weight:700;border:none;cursor:pointer;margin-top:1rem}
pre{background:#000;border:1px solid #333;border-radius:8px;padding:1rem;min-height:3rem;white-space:pre-wrap;word-break:break-word;margin-top:1rem;overflow:auto;max-height:70vh}
h2{margin-top:2rem;font-size:1.05rem;color:#ffd54f}
li.running{display:flex;align-items:center;gap:.8rem;border-color:#ffd54f}
li.running .name{flex:0 0 auto}
.elapsed{color:#9e9e9e;font-size:.85rem;flex:1;text-align:right;margin-left:auto}
button.kill{background:#e53935;color:#fff;width:auto;margin:0;padding:.25rem .8rem;font-size:.85rem;cursor:pointer}
";

fn page(title: &str, content: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\">\
<title>{}</title><style>{STYLE}</style></head><body>{}</body></html>",
        html_escape(title),
        content
    )
}

fn index_page(bake: &BakeViewModel) -> String {
    let mut items = String::new();
    for task in bake.tasks() {
        items.push_str(&format!(
            "<li><a href=\"/{}\">▶ {}</a><span class=\"help\">{}</span></li>",
            task.name(),
            html_escape(task.name()),
            task.help_msg().map(html_escape).unwrap_or_default(),
        ));
    }
    page(
        "bake",
        &format!("<h1>▶ Bake</h1>{RUNNING_WIDGET}<ul>{items}</ul>"),
    )
}

/// live list of running tasks with kill buttons
/// polls GET /running every 2s and kills with DELETE /running/{id}
const RUNNING_WIDGET: &str = r#"<div id="running"></div>
<script>
const esc=s=>s.replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));
function killRun(id){fetch('/running/'+id,{method:'DELETE'}).then(refreshRunning);}
async function refreshRunning(){
  try{
    const res=await fetch('/running');
    if(!res.ok)return;
    const {running}=await res.json();
    const el=document.getElementById('running');
    if(!running.length){el.innerHTML='';return;}
    el.innerHTML='<h2>Running now</h2><ul>'+running.map(t=>
      '<li class="running"><a href="/'+esc(t.name)+'">▶ '+esc(t.name)+'</a>'+
      '<span class="help">running for '+Math.floor(t.elapsed_ms/1000)+'s</span>'+
      '<span class="elapsed">#'+t.id+'</span>'+
      '<button class="kill" onclick="killRun('+t.id+')">Kill</button></li>').join('')+'</ul>';
  }catch(e){}
}
refreshRunning();
setInterval(refreshRunning,2000);
</script>"#;

fn task_page(task: &TaskViewModel) -> String {
    let mut inputs = String::new();
    for param in task.params() {
        inputs.push_str(&format!(
            "<label>{}<input name=\"{}\" placeholder=\"{}\" value=\"{}\"></label>",
            html_escape(param.name()),
            html_escape(param.name()),
            html_escape(param.name()),
            param.default().map(html_escape).unwrap_or_default(),
        ));
    }
    let help = task.help_msg().map(html_escape).unwrap_or_default();
    let content = format!(
        "<a href=\"/\">← back</a>\
<h1>{}</h1><p class=\"help\">{help}</p>\
<form id=\"f\">{inputs}<button>Run</button></form>\
<button id=\"kill\" class=\"kill\" hidden>Kill</button>\
<pre id=\"out\" hidden></pre>\
{RUNNING_WIDGET}
<script>
const out=document.getElementById('out');
const killBtn=document.getElementById('kill');
let runId=null;
document.getElementById('f').onsubmit=async e=>{{
  e.preventDefault();
  out.hidden=false; out.textContent='';
  const body=new URLSearchParams([...new FormData(e.target)].filter(([,v])=>v!==''));
  const res=await fetch(location.pathname,{{method:'POST',body}});
  runId=res.headers.get('X-Run-Id');
  killBtn.hidden=false;
  const reader=res.body.getReader(); const dec=new TextDecoder();
  for(;;){{
    const {{done,value}}=await reader.read();
    if(done)break;
    out.textContent+=dec.decode(value,{{stream:true}});
    out.scrollTop=out.scrollHeight;
  }}
  killBtn.hidden=true; runId=null;
}};
killBtn.onclick=()=>{{ if(runId!=null) fetch('/running/'+runId,{{method:'DELETE'}}); }};
</script>",
        html_escape(task.name()),
    );
    page(task.name(), &content)
}

// ---------- http helpers ----------

fn response(status: u16, content_type: &'static str, body: String) -> Resp {
    Resp::Full(status, content_type, body)
}

fn parse_urlencoded(input: &str) -> Vec<(String, String)> {
    input
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let mut kv = pair.splitn(2, '=');
            (
                percent_decode(kv.next().unwrap_or(""), true),
                percent_decode(kv.next().unwrap_or(""), true),
            )
        })
        .collect()
}

fn percent_decode(input: &str, plus_as_space: bool) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => match u8::from_str_radix(&input[i + 1..i + 3], 16) {
                Ok(b) => {
                    out.push(b);
                    i += 3;
                }
                Err(_) => {
                    out.push(b'%');
                    i += 1;
                }
            },
            b'+' if plus_as_space => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decode_test() {
        assert_eq!(percent_decode("%2Fhello%20world", false), "/hello world");
        assert_eq!(percent_decode("a+b", true), "a b");
        assert_eq!(percent_decode("a+b", false), "a+b");
        assert_eq!(percent_decode("plain", true), "plain");
        assert_eq!(percent_decode("bad%zz", false), "bad%zz");
        assert_eq!(percent_decode("truncated%2", false), "truncated%2");
    }

    #[test]
    fn parse_urlencoded_test() {
        assert_eq!(
            parse_urlencoded("PORT=8080&MODE=debug+mode"),
            vec![
                ("PORT".to_string(), "8080".to_string()),
                ("MODE".to_string(), "debug mode".to_string()),
            ]
        );
        assert_eq!(parse_urlencoded(""), Vec::<(String, String)>::new());
        assert_eq!(
            parse_urlencoded("flag"),
            vec![("flag".to_string(), "".to_string())]
        );
    }

    #[test]
    fn html_escape_test() {
        assert_eq!(
            html_escape("<a href=\"x\">&</a>"),
            "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;"
        );
    }

    #[test]
    fn base64_decode_test() {
        assert_eq!(base64_decode("").unwrap(), Vec::<u8>::new());
        assert_eq!(base64_decode("YQ==").unwrap(), b"a".to_vec());
        assert_eq!(base64_decode("YWI=").unwrap(), b"ab".to_vec());
        assert_eq!(base64_decode("YWJj").unwrap(), b"abc".to_vec());
        // 'user:pass'
        assert_eq!(
            base64_decode("dXNlcjpwYXNz").unwrap(),
            b"user:pass".to_vec()
        );
        // whitespace is tolerated
        assert_eq!(
            base64_decode("dXNl\n cjpwYXNz").unwrap(),
            b"user:pass".to_vec()
        );
        assert!(base64_decode("abc").is_none()); // bad length
        assert!(base64_decode("a*bc").is_none()); // bad char
    }

    #[test]
    fn basic_auth_matches_test() {
        let header = format!("Basic {}", encode_std_base64(":s3cret"));
        assert!(
            basic_auth_matches(&header, "s3cret"),
            "header={:?} decoded={:?}",
            header,
            base64_decode(header.trim())
        );
        assert!(!basic_auth_matches(&header, "wrong"));
        assert!(!basic_auth_matches("Basic !!!!", "s3cret"));
        assert!(!basic_auth_matches("", "s3cret"));
        // no colon -> treated as whole value
        let no_user = format!("Basic {}", encode_std_base64("s3cret"));
        assert!(basic_auth_matches(&no_user, "s3cret"));
    }

    /// test helper: standard base64 encoding
    fn encode_std_base64(input: &str) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in input.as_bytes().chunks(3) {
            let mut acc: u32 = 0;
            for &b in chunk {
                acc = (acc << 8) | b as u32;
            }
            let n = chunk.len();
            acc <<= 8 * (3 - n);
            for i in 0..4 {
                if i <= n {
                    out.push(CHARS[(acc >> (18 - 6 * i)) as usize & 63] as char);
                } else {
                    out.push('=');
                }
            }
        }
        out
    }
}
