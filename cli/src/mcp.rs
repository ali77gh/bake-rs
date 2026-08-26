//! model context protocol endpoint for `bake serve`
//!
//! implements the streamable http transport in its simplest stateless form:
//! agents POST one json-rpc message to /mcp and get one json object back
//! (no sessions, no sse streams). auth is the same BAKE_PASSWORD basic auth
//! that protects every other route.
//!
//! tools: list_tasks, run_task, get_output, list_running, kill_run

use std::sync::OnceLock;

use serde_json::json;
use serde_json::Value;

use core::viewmodel::BakeViewModel;

use crate::serve::{find_run, probe_caps, run_report, start_run};

/// newest protocol version this server speaks
const LATEST: &str = "2025-06-18";
/// older versions are answered with themselves so old clients keep working
const SUPPORTED: &[&str] = &["2024-11-05", "2025-03-26", "2025-06-18"];

pub(crate) fn handle(req: &crate::serve::Req) -> crate::serve::Resp {
    let parsed = std::str::from_utf8(&req.body)
        .ok()
        .and_then(|x| serde_json::from_str::<Value>(x).ok());

    let Some(msg) = parsed else {
        return reply_error(&Value::Null, -32700, "parse error");
    };
    if !msg.is_object() {
        return reply_error(&Value::Null, -32600, "invalid request");
    }

    // requests carry an id and need an answer, notifications don't
    let id = msg.get("id").cloned();
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = msg.get("params");

    match (id.is_some(), method) {
        (_, "initialize") => reply_result(id.as_ref().unwrap_or(&Value::Null), initialize(params)),
        // notifications expect no response, 202 accepted is enough
        (false, _) => crate::serve::Resp::Full(202, "text/plain", String::new()),
        (_, "ping") => reply_result(&id.unwrap_or(Value::Null), json!({})),
        (_, "tools/list") => reply_result(&id.unwrap_or(Value::Null), json!({ "tools": tools() })),
        (_, "tools/call") => call_tool(&id.unwrap_or(Value::Null), params),
        (true, other) => reply_error(
            &id.unwrap_or(Value::Null),
            -32601,
            &format!("method not found: {other}"),
        ),
    }
}

/// version negotiation + server identity/capabilities
fn initialize(params: Option<&Value>) -> Value {
    let requested = params
        .and_then(|p| p.get("protocolVersion"))
        .and_then(|v| v.as_str())
        .unwrap_or(LATEST);
    let version = if SUPPORTED.contains(&requested) {
        requested
    } else {
        LATEST
    };
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": "bake",
            "version": core::util::version::VERSION,
        },
    })
}

fn tools() -> &'static [Value] {
    static TOOLS: OnceLock<Vec<Value>> = OnceLock::new();
    TOOLS.get_or_init(|| {
        vec![
            json!({
                "name": "list_tasks",
                "description": "List all tasks defined in this project's bakefile with their help text, params and commands.",
                "inputSchema": { "type": "object", "properties": {} },
            }),
            json!({
                "name": "run_task",
                "description": "Run a task from the project's bakefile. By default waits for completion and returns the full output. Pass background=true for long-running tasks to return immediately with a run id.",
                "inputSchema": {
                    "type": "object",
                    "required": ["task"],
                    "properties": {
                        "task": { "type": "string", "description": "task name from list_tasks" },
                        "params": {
                            "type": "object",
                            "additionalProperties": { "type": "string" },
                            "description": "values for the task's declared params",
                        },
                        "background": { "type": "boolean", "description": "return immediately instead of waiting" },
                    },
                },
            }),
            json!({
                "name": "get_output",
                "description": "Get the output collected so far for a run. Works while the task runs and after it finishes.",
                "inputSchema": {
                    "type": "object",
                    "required": ["run_id"],
                    "properties": {
                        "run_id": { "type": "integer", "description": "run id from run_task or list_running" },
                    },
                },
            }),
            json!({
                "name": "list_running",
                "description": "List currently running tasks with their run ids and elapsed time.",
                "inputSchema": { "type": "object", "properties": {} },
            }),
            json!({
                "name": "kill_run",
                "description": "Kill a running task and its whole process tree.",
                "inputSchema": {
                    "type": "object",
                    "required": ["run_id"],
                    "properties": {
                        "run_id": { "type": "integer", "description": "run id from list_running" },
                    },
                },
            }),
        ]
    })
}

/// executes a tools/call request, failures become isError results so agents
/// can read them (protocol errors like unknown methods use json-rpc errors instead)
fn call_tool(id: &Value, params: Option<&Value>) -> crate::serve::Resp {
    let args = params
        .and_then(|p| p.get("arguments"))
        .cloned()
        .unwrap_or(json!({}));
    let name = params
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or_default();

    match dispatch_tool(name, &args) {
        Ok(text) => {
            let result = json!({ "content": [{ "type": "text", "text": text }] });
            reply_result(id, result)
        }
        Err(e) => {
            let result = json!({
                "content": [{ "type": "text", "text": e }],
                "isError": true,
            });
            reply_result(id, result)
        }
    }
}

fn dispatch_tool(name: &str, args: &Value) -> Result<String, String> {
    match name {
        "list_tasks" => tool_list_tasks(),
        "run_task" => tool_run_task(args),
        "get_output" => tool_get_output(args),
        "list_running" => Ok(serde_json::to_string_pretty(
            &json!({ "running": crate::serve::running_value() }),
        )
        .unwrap_or_default()),
        "kill_run" => {
            let id = run_id_arg(args)?;
            if crate::serve::kill_run(id) {
                Ok(format!("killed run #{id}"))
            } else {
                Ok(format!("no running task with id {id}"))
            }
        }
        other => Err(format!("unknown tool '{other}'")),
    }
}

fn run_id_arg(args: &Value) -> Result<u64, String> {
    args.get("run_id")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| "expected integer 'run_id'".to_string())
}

fn view_model() -> Result<BakeViewModel, String> {
    BakeViewModel::new(probe_caps())
}

fn tool_list_tasks() -> Result<String, String> {
    let bake = view_model()?;
    let tasks: Vec<Value> = bake.tasks().iter().map(task_summary).collect();
    Ok(serde_json::to_string_pretty(&json!({ "tasks": tasks })).unwrap_or_default())
}

fn task_summary(task: &core::viewmodel::task_viewmodel::TaskViewModel) -> Value {
    json!({
        "name": task.name(),
        "help_msg": task.help_msg(),
        "params": task.params().iter().map(|p| json!({
            "name": p.name(),
            "default": p.default(),
        })).collect::<Vec<_>>(),
    })
}

fn tool_run_task(args: &Value) -> Result<String, String> {
    let task = args
        .get("task")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "expected string 'task'".to_string())?
        .to_string();
    let background = args
        .get("background")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);

    let mut raw_params = Vec::new();
    if let Some(map) = args.get("params").and_then(|x| x.as_object()) {
        for (key, value) in map {
            let value = value
                .as_str()
                .ok_or_else(|| format!("param '{key}' must be a string"))?;
            raw_params.push((key.clone(), value.to_string()));
        }
    }

    let bake = view_model()?;
    let (entry, rx) = start_run(&bake, &task, raw_params)?;

    if background {
        return Ok(format!(
            "started task '{task}' as run #{}, poll get_output(run_id={}) or kill it with kill_run",
            entry.id(),
            entry.id()
        ));
    }

    // drain the stream channel to wait for completion, it closes when the
    // worker thread finishes, the output itself is collected inside the entry
    for _ in rx {}
    Ok(run_report(&entry))
}

fn tool_get_output(args: &Value) -> Result<String, String> {
    let id = run_id_arg(args)?;
    match find_run(id) {
        Some(entry) => Ok(run_report(&entry)),
        None => Err(format!("no run with id {id}")),
    }
}

// ---------- json-rpc answers ----------

fn reply_result(id: &Value, result: Value) -> crate::serve::Resp {
    let body = json!({ "jsonrpc": "2.0", "id": id, "result": result });
    crate::serve::Resp::Full(200, "application/json", body.to_string())
}

fn reply_error(id: &Value, code: i64, message: &str) -> crate::serve::Resp {
    let body = json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    });
    crate::serve::Resp::Full(200, "application/json", body.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serve::Resp;

    fn req(body: &str) -> crate::serve::Req {
        crate::serve::Req {
            method: "POST".to_string(),
            path: "mcp".to_string(),
            query: None,
            headers: Vec::new(),
            body: body.as_bytes().to_vec(),
        }
    }

    fn full_body(resp: Resp) -> Value {
        match resp {
            Resp::Full(_, _, body) => serde_json::from_str(&body).unwrap(),
            _ => panic!("expected full response"),
        }
    }

    #[test]
    fn parse_error_test() {
        let resp = handle(&req("{not json"));
        assert!(matches!(resp, Resp::Full(200, "application/json", _)));
        let v = full_body(resp);
        assert_eq!(v["error"]["code"], -32700);
    }

    #[test]
    fn invalid_request_test() {
        let resp = handle(&req("[1, 2, 3]"));
        assert_eq!(full_body(resp)["error"]["code"], -32600);
    }

    #[test]
    fn initialize_test() {
        let resp = handle(&req(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}"#,
        ));
        let v = full_body(resp);
        assert_eq!(v["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(v["result"]["serverInfo"]["name"], "bake");
        assert!(v["result"]["capabilities"]["tools"].is_object());
    }

    /// old client versions get their own version echoed back
    #[test]
    fn initialize_old_version_test() {
        let resp = handle(&req(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#,
        ));
        assert_eq!(full_body(resp)["result"]["protocolVersion"], "2024-11-05");
    }

    /// unknown versions get the server's latest
    #[test]
    fn initialize_future_version_test() {
        let resp = handle(&req(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2999-01-01"}}"#,
        ));
        assert_eq!(full_body(resp)["result"]["protocolVersion"], LATEST);
    }

    #[test]
    fn notification_returns_202_test() {
        let resp = handle(&req(
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        ));
        assert!(matches!(resp, Resp::Full(202, _, _)));
    }

    #[test]
    fn ping_test() {
        let resp = handle(&req(r#"{"jsonrpc":"2.0","id":7,"method":"ping"}"#));
        let v = full_body(resp);
        assert_eq!(v["id"], 7);
        assert!(v["result"].is_object());
    }

    #[test]
    fn method_not_found_test() {
        let resp = handle(&req(
            r#"{"jsonrpc":"2.0","id":1,"method":"resources/read"}"#,
        ));
        assert_eq!(full_body(resp)["error"]["code"], -32601);
    }

    #[test]
    fn tools_list_test() {
        let resp = handle(&req(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#));
        let v = full_body(resp);
        let tools = v["result"]["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            vec![
                "list_tasks",
                "run_task",
                "get_output",
                "list_running",
                "kill_run"
            ]
        );
        for tool in tools {
            assert!(tool["description"].is_string());
            assert_eq!(tool["inputSchema"]["type"], "object");
        }
    }

    #[test]
    fn unknown_tool_is_error_result_test() {
        let resp = handle(&req(
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nope","arguments":{}}}"#,
        ));
        let v = full_body(resp);
        assert_eq!(v["result"]["isError"], true);
        assert!(v["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("unknown tool"));
    }

    #[test]
    fn missing_args_are_error_results_test() {
        for (tool, args) in [("run_task", "{}"), ("get_output", "{}"), ("kill_run", "{}")] {
            let body = format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"{tool}","arguments":{args}}}}}"#
            );
            let v = full_body(handle(&req(&body)));
            assert_eq!(v["result"]["isError"], true, "tool: {tool}");
        }
    }

    #[test]
    fn list_running_tool_test() {
        let resp = handle(&req(
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_running","arguments":{}}}"#,
        ));
        let v = full_body(resp);
        assert!(v["result"]["isError"].is_null());
        assert!(v["result"]["content"][0]["text"].is_string());
    }
}
