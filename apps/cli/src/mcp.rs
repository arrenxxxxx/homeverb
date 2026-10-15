use homeverb_core::{Core, Operation};
use homeverb_domain::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio_util::sync::CancellationToken;

const PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolCall {
    name: String,
    arguments: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceArguments {
    device: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionArguments {
    request_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StateArguments {
    device: String,
    #[serde(default = "main_component")]
    component: String,
    #[serde(default)]
    cached: bool,
}
fn main_component() -> String {
    "main".into()
}

fn tools() -> Value {
    json!({"tools": [
        {"name": "device_list", "description": "List local fictional device records without discovery.",
            "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false}, "annotations": {"readOnlyHint": true}},
        {"name": "device_describe", "description": "Read locally stored device capabilities.",
            "inputSchema": {"type": "object", "properties": {"device": {"type": "string"}}, "required": ["device"], "additionalProperties": false}, "annotations": {"readOnlyHint": true}},
        {"name": "device_get_state", "description": "Read fictional device state on demand.",
            "inputSchema": {"type": "object", "properties": {"device": {"type": "string"}, "component": {"type": "string", "default": "main"}, "cached": {"type": "boolean", "default": false}},
                "required": ["device"], "additionalProperties": false}, "annotations": {"readOnlyHint": true}},
        {"name": "execution_get", "description": "Read one saved receipt without refreshing state, changing its outcome, or replaying a command.",
            "inputSchema": {"type": "object", "properties": {"request_id": {"type": "string", "minLength": 1}},
                "required": ["request_id"], "additionalProperties": false}, "annotations": {"readOnlyHint": true}}
    ]})
}

async fn call(core: Arc<Core>, params: Value, cancellation: CancellationToken) -> Result<Value> {
    let call: ToolCall = serde_json::from_value(params).map_err(|_| Error::invalid())?;
    let operation = Operation::new(10_000, cancellation)?;
    match call.name.as_str() {
        "device_list" if call.arguments == json!({}) => core.list().map(|v| json!(v)),
        "device_describe" => {
            let args: DeviceArguments =
                serde_json::from_value(call.arguments).map_err(|_| Error::invalid())?;
            core.describe(&args.device).map(|v| json!(v))
        }
        "device_get_state" => {
            let args: StateArguments =
                serde_json::from_value(call.arguments).map_err(|_| Error::invalid())?;
            core.read(
                &args.device,
                &args.component,
                Route::Auto,
                args.cached,
                &operation,
            )
            .await
            .map(|v| json!(v))
        }
        "execution_get" => {
            let args: ExecutionArguments =
                serde_json::from_value(call.arguments).map_err(|_| Error::invalid())?;
            core.execution(&args.request_id, &operation)
                .await
                .map(|v| json!(v))
        }
        _ => Err(Error::new(
            "POLICY_DENIED",
            "This MCP host exposes only the listed read tools.",
        )),
    }
}

pub async fn serve(core: Arc<Core>, cancellation: CancellationToken) -> Result<u8> {
    let mut input = BufReader::new(tokio::io::stdin());
    let mut output = tokio::io::stdout();
    let mut line = Vec::new();
    let mut initialized = false;
    let mut ready = false;
    let mut pending: HashMap<String, CancellationToken> = HashMap::new();
    let mut tasks: tokio::task::JoinSet<(Value, Value)> = tokio::task::JoinSet::new();
    loop {
        // read_until retains partial bytes when a completed tool interrupts input.
        let mut bounded_input = (&mut input).take(1_048_577 - line.len() as u64);
        let response = tokio::select! {
            _ = cancellation.cancelled() => break,
            Some(joined) = tasks.join_next(), if !tasks.is_empty() => {
                match joined {
                    Ok((id, result)) => { pending.remove(&id.to_string()); Some(json!({"jsonrpc": "2.0", "id": id, "result": result})) },
                    Err(_) => None,
                }
            }
            read = bounded_input.read_until(b'\n', &mut line) => {
                if read.map_err(|_| Error::new("MCP_IO_ERROR", "The stdio input failed."))? == 0 { break; }
                if line.len() > 1_048_576 { break; }
                let frame = std::mem::take(&mut line);
                match serde_json::from_slice::<Value>(&frame) {
                    Err(_) => Some(rpc_error(Value::Null, -32700, "Parse error")),
                    Ok(request) => {
                        let id = request.get("id").cloned();
                        let method = request.get("method").and_then(Value::as_str).unwrap_or("");
                        if request.get("jsonrpc") != Some(&json!("2.0")) || method.is_empty() || id.as_ref().is_some_and(|id| !id.is_string() && !id.is_number()) {
                            Some(rpc_error(id.unwrap_or(Value::Null), -32600, "Invalid request"))
                        } else if id.is_none() {
                            if method == "notifications/initialized" && initialized { ready = true; }
                            if method == "notifications/cancelled"
                                && let Some(token) = request.pointer("/params/requestId").and_then(|id| pending.get(&id.to_string())) {
                                token.cancel();
                            }
                            None
                        } else {
                            let id = id.expect("request ID exists");
                            match method {
                                "initialize" if !initialized => {
                                    initialized = true;
                                    Some(json!({"jsonrpc": "2.0", "id": id, "result": {"protocolVersion": PROTOCOL_VERSION,
                                        "capabilities": {"tools": {"listChanged": false}}, "serverInfo": {"name": "homeverb", "version": env!("CARGO_PKG_VERSION")}}}))
                                }
                                "ping" => Some(json!({"jsonrpc": "2.0", "id": id, "result": {}})),
                                _ if !ready => Some(rpc_error(id, -32002, "Initialize the session before calling tools")),
                                "tools/list" => Some(json!({"jsonrpc": "2.0", "id": id, "result": tools()})),
                                "tools/call" if pending.len() < 8 && !pending.contains_key(&id.to_string()) => {
                                    let token = cancellation.child_token();
                                    pending.insert(id.to_string(), token.clone());
                                    let params = request.get("params").cloned().unwrap_or(Value::Null);
                                    let core = core.clone();
                                    tasks.spawn(async move {
                                        let result = call(core, params, token).await;
                                        let is_error = result.is_err();
                                        let envelope = json!(Query::from_result(result));
                                        (id, json!({"content": [{"type": "text", "text": envelope.to_string()}], "structuredContent": envelope, "isError": is_error}))
                                    });
                                    None
                                }
                                "tools/call" => Some(rpc_error(id, -32000, "Duplicate request ID or too many active requests")),
                                _ => Some(rpc_error(id, -32601, "Method not found")),
                            }
                        }
                    }
                }
            }
        };
        if let Some(response) = response {
            output
                .write_all(format!("{response}\n").as_bytes())
                .await
                .map_err(|_| Error::new("MCP_IO_ERROR", "The stdio output failed."))?;
            output
                .flush()
                .await
                .map_err(|_| Error::new("MCP_IO_ERROR", "The stdio output failed."))?;
        }
    }
    for token in pending.values() {
        token.cancel();
    }
    while tasks.join_next().await.is_some() {}
    Ok(0)
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}
