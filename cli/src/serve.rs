use std::io::{BufRead, BufReader, Cursor, Read};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use colored::Colorize;
use core::viewmodel::capabilities::Capabilities;
use core::viewmodel::message::Message;
use core::viewmodel::task_viewmodel::TaskViewModel;
use core::viewmodel::BakeViewModel;
use serde_json::json;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

use crate::capabilities::{SHELL, SWITCH};

/// [Capabilities] implementation for `bake serve`
/// instead of printing to stdout it streams every message/command output
/// into a channel which is consumed by the http response body
struct ServeCapabilities {
    tx: Sender<Vec<u8>>,
}

impl Capabilities for ServeCapabilities {
    fn read_file(&self, file_name: &str) -> Option<String> {
        std::fs::read_to_string(file_name).ok()
    }

    fn execute(&self, command: &str, working_directory: Option<&str>) -> bool {
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

        let child = Command::new(SHELL)
            .arg(SWITCH)
            .arg(command)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match child {
            Ok(x) => x,
            Err(e) => {
                self.message(Message::error(format!("can't run '{}': {}\n", command, e)));
                return false;
            }
        };

        // stdout is forwarded here, stderr in a separate thread
        if let Some(stdout) = child.stdout.take() {
            forward_lines(stdout, &self.tx);
        }
        if let Some(stderr) = child.stderr.take() {
            let tx = self.tx.clone();
            thread::spawn(move || forward_lines(stderr, &tx));
        }

        match child.wait() {
            Ok(status) => status.success(),
            Err(_) => false,
        }
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

/// reads streamed chunks and turns them into an http response body
struct ChannelReader {
    rx: Receiver<Vec<u8>>,
    buf: Vec<u8>,
}

impl Read for ChannelReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        while self.buf.is_empty() {
            match self.rx.recv() {
                Ok(chunk) => self.buf = chunk,
                Err(_) => return Ok(0), // sender dropped -> end of stream
            }
        }
        let n = out.len().min(self.buf.len());
        out[..n].copy_from_slice(&self.buf[..n]);
        self.buf.drain(..n);
        Ok(n)
    }
}

pub fn start_server(port: u16) {
    let addr = format!("127.0.0.1:{port}");
    let server = match Server::http(&addr) {
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

    for request in server.incoming_requests() {
        thread::spawn(move || handle_request(request));
    }
}

type Resp = Response<Box<dyn Read>>;

fn handle_request(mut request: Request) {
    let method = request.method().clone();
    let url = request.url().to_string();
    let (raw_path, query) = match url.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (url.as_str(), None),
    };
    let path = if raw_path == "/" {
        "/".to_string()
    } else {
        percent_decode(raw_path.trim_matches('/'), false)
    };

    let mut params = query.map(parse_urlencoded).unwrap_or_default();
    if method == Method::Post {
        let mut body = String::new();
        if request
            .as_reader()
            .take(64 * 1024)
            .read_to_string(&mut body)
            .is_ok()
        {
            params.extend(parse_urlencoded(&body));
        }
    }

    let response = route(&request, &method, &path, params);
    let _ = request.respond(response);
}

fn route(request: &Request, method: &Method, path: &str, params: Vec<(String, String)>) -> Resp {
    let html = wants_html(request);
    let bake = match BakeViewModel::new(probe_caps()) {
        Ok(x) => x,
        Err(e) => return response(500, "text/plain", e),
    };

    match (method, path) {
        (&Method::Get, "/") => {
            if html {
                response(200, "text/html", index_page(&bake))
            } else {
                tasks_json(&bake)
            }
        }
        (&Method::Get, name) => match bake.get_task(name) {
            Some(task) => {
                if html {
                    response(200, "text/html", task_page(task))
                } else {
                    task_json_response(task)
                }
            }
            None => response(404, "text/plain", format!("task '{name}' not found")),
        },
        (&Method::Post, name) => {
            if bake.get_task(name).is_none() {
                return response(404, "text/plain", format!("task '{name}' not found"));
            }
            let (tx, rx) = mpsc::channel::<Vec<u8>>();
            let name = name.to_string();
            thread::spawn(move || run_task_streaming(name, params, tx));
            Response::new(
                StatusCode(200),
                vec![],
                Box::new(ChannelReader {
                    rx,
                    buf: Vec::new(),
                }) as Box<dyn Read>,
                None,
                None,
            )
        }
        _ => response(405, "text/plain", "method not allowed".to_string()),
    }
}

/// bakefile view model for read only checks (messages are discarded)
fn probe_caps() -> Rc<dyn Capabilities> {
    let (tx, _) = mpsc::channel();
    Rc::new(ServeCapabilities { tx })
}

/// runs the task and streams everything (bake messages + command output) to the client
fn run_task_streaming(task_name: String, params: Vec<(String, String)>, tx: Sender<Vec<u8>>) {
    let cap: Rc<dyn Capabilities> = Rc::new(ServeCapabilities { tx });

    let bake = match BakeViewModel::new(Rc::clone(&cap)) {
        Ok(x) => x,
        Err(e) => {
            cap.message(Message::error(format!("{e}\n")));
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

    if let Err(e) = bake.run_task(&task_name) {
        cap.message(Message::error(format!("{e}\n")));
        cap.message(Message::error(format!(
            "Task '{task_name}' failed to run\n"
        )));
    }
}

// ---------- json api ----------

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

fn wants_html(request: &Request) -> bool {
    request
        .headers()
        .iter()
        .any(|h| h.field.equiv("Accept") && h.value.as_str().to_lowercase().contains("text/html"))
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
    page("bake", &format!("<h1>▶ Bake</h1><ul>{items}</ul>"))
}

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
<pre id=\"out\" hidden></pre>\
<script>
const out=document.getElementById('out');
document.getElementById('f').onsubmit=async e=>{{
  e.preventDefault();
  out.hidden=false; out.textContent='';
  const body=new URLSearchParams([...new FormData(e.target)].filter(([,v])=>v!==''));
  const res=await fetch(location.pathname,{{method:'POST',body}});
  const reader=res.body.getReader(); const dec=new TextDecoder();
  for(;;){{
    const {{done,value}}=await reader.read();
    if(done)break;
    out.textContent+=dec.decode(value,{{stream:true}});
    out.scrollTop=out.scrollHeight;
  }}
}};
</script>",
        html_escape(task.name()),
    );
    page(task.name(), &content)
}

// ---------- http helpers ----------

fn response(status: u16, content_type: &str, body: String) -> Resp {
    let bytes = body.into_bytes();
    let len = bytes.len();
    let mut resp = Response::new(
        StatusCode(status),
        vec![],
        Box::new(Cursor::new(bytes)) as Box<dyn Read>,
        Some(len),
        None,
    );
    if let Ok(h) = Header::from_bytes(b"Content-Type".as_ref(), content_type.as_bytes()) {
        resp.add_header(h);
    }
    resp
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
}
