use homeverb_core::{Core, Operation};
use homeverb_domain::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
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
struct CommandArguments {
    command: Command,
}

enum ToolOutput {
    Query(Value),
    Execution(Box<Execution>, WaitFor),
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

fn tools(allow_controls: bool) -> Value {
    let mut tools = json!([
        {"name": "device_list", "description": "List local device records without discovery or account refresh.",
            "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false}, "annotations": {"readOnlyHint": true}},
        {"name": "device_describe", "description": "Read locally stored device capabilities.",
            "inputSchema": {"type": "object", "properties": {"device": {"type": "string"}}, "required": ["device"], "additionalProperties": false}, "annotations": {"readOnlyHint": true}},
        {"name": "device_get_state", "description": "Read device state on demand. Cached reads are local; a live cloud response may still be stale and does not verify a control.",
            "inputSchema": {"type": "object", "properties": {"device": {"type": "string"}, "component": {"type": "string", "default": "main"}, "cached": {"type": "boolean", "default": false}},
                "required": ["device"], "additionalProperties": false}, "annotations": {"readOnlyHint": true}},
        {"name": "execution_get", "description": "Read one saved receipt without refreshing state, changing its outcome, or replaying a command.",
            "inputSchema": {"type": "object", "properties": {"request_id": {"type": "string", "minLength": 1}},
                "required": ["request_id"], "additionalProperties": false}, "annotations": {"readOnlyHint": true}}
    ]);
    if allow_controls {
        let schema: Value =
            serde_json::from_str(include_str!("../../../schemas/command.schema.json"))
                .expect("the compiled Command schema is valid JSON");
        let input = json!({"type":"object", "properties":{"command":schema},
            "required":["command"], "additionalProperties":false});
        tools.as_array_mut().expect("tool list").extend([
            json!({"name":"command_plan", "description":"Validate one complete command against local capabilities and trusted startup grants without sending or journaling. This cannot prove reachability or physical success.",
                "inputSchema":input, "annotations":{"readOnlyHint":true, "openWorldHint":false}}),
            json!({"name":"command_execute", "description":"Execute one explicitly requested command within trusted startup grants. Preserve its request_id across retries and inspect execution_get after cancellation. Never generate a new ID to retry an accepted or unknown outcome; cancellation is not physical rollback.",
                "inputSchema":input, "annotations":{"readOnlyHint":false, "destructiveHint":true, "idempotentHint":false, "openWorldHint":true}}),
        ]);
    }
    json!({"tools":tools})
}

async fn call(
    core: Arc<Core>,
    params: Value,
    allow_controls: bool,
    cancellation: CancellationToken,
) -> Result<ToolOutput> {
    let call: ToolCall = serde_json::from_value(params).map_err(|_| Error::invalid())?;
    if allow_controls && matches!(call.name.as_str(), "command_plan" | "command_execute") {
        let args: CommandArguments =
            serde_json::from_value(call.arguments).map_err(|_| Error::invalid())?;
        args.command.validate()?;
        let operation = Operation::new(args.command.timeout_ms, cancellation)?;
        if call.name == "command_plan" {
            return operation
                .run(async { core.plan(&args.command) })
                .await
                .map(ToolOutput::Query);
        }
        let wait = args.command.wait_for;
        let execution = core.execute_with_operation(args.command, &operation).await;
        return Ok(ToolOutput::Execution(Box::new(execution), wait));
    }
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
            "This MCP session exposes only its listed tools.",
        )),
    }
    .map(ToolOutput::Query)
}

fn tool_result(result: Result<ToolOutput>) -> Value {
    let (envelope, is_error) = match result {
        Ok(ToolOutput::Execution(execution, wait)) => {
            let is_error = super::execution_exit(&execution, wait) != 0;
            (json!(execution), is_error)
        }
        Ok(ToolOutput::Query(data)) => (json!(Query::from_result(Ok(data))), false),
        Err(error) => (json!(Query::<Value>::from_result(Err(error))), true),
    };
    json!({"content":[{"type":"text", "text":envelope.to_string()}],
        "structuredContent":envelope, "isError":is_error})
}

pub async fn serve(
    core: Arc<Core>,
    allow_controls: bool,
    cancellation: CancellationToken,
) -> Result<u8> {
    serve_io(
        core,
        allow_controls,
        cancellation,
        tokio::io::stdin(),
        tokio::io::stdout(),
    )
    .await
}

async fn serve_io(
    core: Arc<Core>,
    allow_controls: bool,
    cancellation: CancellationToken,
    input: impl AsyncRead + Unpin,
    mut output: impl AsyncWrite + Unpin,
) -> Result<u8> {
    let mut input = BufReader::new(input);
    let mut line = Vec::new();
    let mut initialized = false;
    let mut ready = false;
    let mut pending: HashMap<String, CancellationToken> = HashMap::new();
    let mut tasks: tokio::task::JoinSet<(Value, Value)> = tokio::task::JoinSet::new();
    let result: Result<()> = async {
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
                                    "tools/list" => Some(json!({"jsonrpc": "2.0", "id": id, "result": tools(allow_controls)})),
                                    "tools/call" if pending.len() < 8 && !pending.contains_key(&id.to_string()) => {
                                        let token = cancellation.child_token();
                                        pending.insert(id.to_string(), token.clone());
                                        let params = request.get("params").cloned().unwrap_or(Value::Null);
                                        let core = core.clone();
                                        tasks.spawn(async move {
                                            let result = call(core, params, allow_controls, token).await;
                                            (id, tool_result(result))
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
                tokio::select! {
                    _ = cancellation.cancelled() => break,
                    written = tokio::time::timeout(std::time::Duration::from_secs(2), async {
                        output.write_all(format!("{response}\n").as_bytes()).await?;
                        output.flush().await
                    }) => {
                        written.map_err(|_| Error::new("MCP_IO_ERROR", "The stdio output timed out."))?
                            .map_err(|_| Error::new("MCP_IO_ERROR", "The stdio output failed."))?;
                    }
                }
            }
        }
        Ok(())
    }.await;
    for token in pending.values() {
        token.cancel();
    }
    while tasks.join_next().await.is_some() {}
    result?;
    Ok(0)
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

#[cfg(test)]
mod tests;
