use super::*;
use homeverb_mock::Scenario;
use std::time::Duration;

async fn fixture(scenario: Scenario) -> (tempfile::TempDir, Arc<Core>) {
    let directory = tempfile::tempdir().unwrap();
    let core = Arc::new(homeverb_host::open_mock(directory.path(), scenario, true).unwrap());
    core.discover(&Operation::new(10_000, CancellationToken::new()).unwrap())
        .await
        .unwrap();
    (directory, core)
}

fn command() -> Value {
    json!({"schema_version":"1.0", "request_id":"fictional-command",
        "device_id":homeverb_mock::DEVICE_ID, "component_id":"main", "capability":"power",
        "action":"set", "arguments":{"state":true}, "connection":"auto",
        "timeout_ms":10_000, "wait_for":"verified"})
}

async fn invoke(core: &Arc<Core>, allow: bool, name: &str, arguments: Value) -> Value {
    let result = tool_result(
        call(
            core.clone(),
            json!({"name":name, "arguments":arguments}),
            allow,
            CancellationToken::new(),
        )
        .await,
    );
    let text: Value = serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(text, result["structuredContent"]);
    result
}

#[tokio::test]
async fn controls_need_host_admission_and_core_authority_without_forged_fields() {
    let (directory, core) = fixture(Scenario::Normal).await;
    assert_eq!(tools(false)["tools"].as_array().unwrap().len(), 4);
    let catalog = tools(true);
    assert_eq!(catalog["tools"].as_array().unwrap().len(), 6);
    let schema: Value =
        serde_json::from_str(include_str!("../../../../schemas/command.schema.json")).unwrap();
    for tool in &catalog["tools"].as_array().unwrap()[4..] {
        assert_eq!(tool["inputSchema"]["properties"]["command"], schema);
    }
    assert_eq!(catalog["tools"][5]["annotations"]["readOnlyHint"], false);
    for name in ["command_plan", "command_execute"] {
        let args = json!({"command":command()});
        let denied = invoke(&core, false, name, args.clone()).await;
        assert_eq!(
            denied["structuredContent"]["error"]["code"],
            "POLICY_DENIED"
        );
        let no_authority =
            Arc::new(homeverb_host::open_mock(directory.path(), Scenario::Normal, false).unwrap());
        let denied = invoke(&no_authority, true, name, args).await;
        assert_eq!(denied["isError"], true);
        assert_eq!(
            denied["structuredContent"]["error"]["code"],
            "POLICY_DENIED"
        );
        for field in ["principal", "confirmed", "meiju_power_grants", "profile"] {
            let mut args = json!({"command":command()});
            args[field] = json!("forged");
            assert_eq!(
                invoke(&core, true, name, args).await["structuredContent"]["error"]["code"],
                "INVALID_ARGUMENT"
            );
            let mut value = command();
            value[field] = json!("forged");
            assert_eq!(
                invoke(&core, true, name, json!({"command":value})).await["structuredContent"]["error"]
                    ["code"],
                "INVALID_ARGUMENT"
            );
        }
        for (field, value) in [
            ("request_id", json!(" ")),
            ("timeout_ms", json!(0)),
            ("timeout_ms", json!(180001)),
        ] {
            let mut invalid = command();
            invalid[field] = value;
            assert_eq!(
                invoke(&core, true, name, json!({"command":invalid})).await["structuredContent"]["error"]
                    ["code"],
                "INVALID_ARGUMENT"
            );
        }
    }
    let planned = invoke(&core, true, "command_plan", json!({"command":command()})).await;
    assert_eq!(planned["isError"], false);
    assert_eq!(planned["structuredContent"]["data"]["command"], command());
    assert!(core.recent_executions().unwrap().is_empty());
}

#[tokio::test]
async fn results_preserve_outcomes_and_replay_identity_separately_from_tool_errors() {
    for (scenario, wait, status, is_error) in [
        (Scenario::Normal, "verified", "verified", false),
        (Scenario::AcceptOnly, "accepted", "accepted", false),
        (Scenario::AcceptOnly, "verified", "accepted", true),
        (Scenario::LostAck, "verified", "unknown", true),
        (Scenario::Reject, "verified", "failed", true),
        (Scenario::Partial, "verified", "partial", true),
        (Scenario::SlowBeforeSend, "verified", "failed", true),
        (Scenario::SlowAfterSend, "verified", "unknown", true),
    ] {
        let (_directory, core) = fixture(scenario).await;
        let mut command = command();
        command["wait_for"] = json!(wait);
        if scenario == Scenario::Partial {
            command["capability"] = json!("climate");
            command["arguments"] = json!({"mode":"heat", "target_temperature":25});
        }
        if matches!(scenario, Scenario::SlowBeforeSend | Scenario::SlowAfterSend) {
            command["timeout_ms"] = json!(25);
        }
        let result = invoke(&core, true, "command_execute", json!({"command":command})).await;
        assert_eq!(result["isError"], is_error);
        assert_eq!(result["structuredContent"]["status"], status);
        assert_eq!(
            invoke(&core, true, "command_execute", json!({"command":command})).await,
            result
        );
        let lookup = invoke(
            &core,
            false,
            "execution_get",
            json!({"request_id":"fictional-command"}),
        )
        .await;
        assert_eq!(lookup["isError"], false);
        assert_eq!(
            lookup["structuredContent"]["data"],
            result["structuredContent"]
        );
        if scenario == Scenario::Partial {
            command["arguments"]["target_temperature"] = json!(24);
        } else {
            command["arguments"]["state"] = json!(false);
        }
        let conflict = invoke(&core, true, "command_execute", json!({"command":command})).await;
        assert_eq!(conflict["isError"], true);
        assert_eq!(
            conflict["structuredContent"]["error"]["code"],
            "REQUEST_CONFLICT"
        );
    }
}

async fn send(stream: &mut tokio::io::DuplexStream, value: Value) {
    stream
        .write_all(format!("{value}\n").as_bytes())
        .await
        .unwrap();
}

async fn response(stream: &mut BufReader<tokio::io::DuplexStream>) -> Value {
    let mut line = String::new();
    tokio::time::timeout(Duration::from_secs(3), stream.read_line(&mut line))
        .await
        .unwrap()
        .unwrap();
    serde_json::from_str(&line).unwrap()
}

#[tokio::test]
async fn notification_eof_signal_and_output_failure_preserve_sent_receipts() {
    for stop in [
        "notification",
        "eof",
        "signal",
        "output_failure",
        "output_stalled",
    ] {
        let (directory, core) = fixture(Scenario::SlowAfterSend).await;
        let cancellation = CancellationToken::new();
        let (mut input, server_input) = tokio::io::duplex(4096);
        let (server_output, output) = tokio::io::duplex(128);
        let mut output = BufReader::new(output);
        let server = tokio::spawn(serve_io(
            core.clone(),
            true,
            cancellation.clone(),
            server_input,
            server_output,
        ));
        send(
            &mut input,
            json!({"jsonrpc":"2.0", "id":1, "method":"initialize"}),
        )
        .await;
        response(&mut output).await;
        send(
            &mut input,
            json!({"jsonrpc":"2.0", "method":"notifications/initialized"}),
        )
        .await;
        send(
            &mut input,
            json!({"jsonrpc":"2.0", "id":"rpc-control", "method":"tools/call",
            "params":{"name":"command_execute", "arguments":{"command":command()}}}),
        )
        .await;
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let state = core
                    .read(
                        homeverb_mock::DEVICE_ID,
                        "main",
                        Route::Auto,
                        false,
                        &Operation::new(1000, CancellationToken::new()).unwrap(),
                    )
                    .await
                    .unwrap();
                if state.observations["power.state"].value == json!(true) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap();
        match stop {
            "notification" => {
                send(&mut input, json!({"jsonrpc":"2.0", "method":"notifications/cancelled", "params":{"requestId":"rpc-control"}})).await;
                let result = response(&mut output).await;
                assert_eq!(result["id"], "rpc-control");
                assert_eq!(result["result"]["isError"], true);
                assert_eq!(result["result"]["structuredContent"]["status"], "unknown");
                input.shutdown().await.unwrap();
            }
            "eof" => input.shutdown().await.unwrap(),
            "signal" => cancellation.cancel(),
            "output_failure" => {
                drop(output);
                send(
                    &mut input,
                    json!({"jsonrpc":"2.0", "id":2, "method":"ping"}),
                )
                .await;
            }
            "output_stalled" => {
                send(
                    &mut input,
                    json!({"jsonrpc":"2.0", "id":2, "method":"tools/list"}),
                )
                .await;
            }
            _ => unreachable!(),
        }
        let result = tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .unwrap()
            .unwrap();
        if matches!(stop, "output_failure" | "output_stalled") {
            assert_eq!(result.unwrap_err().code, "MCP_IO_ERROR");
        } else {
            assert_eq!(result.unwrap(), 0);
        }
        let reopened =
            Arc::new(homeverb_host::open_mock(directory.path(), Scenario::Normal, false).unwrap());
        let receipt = invoke(
            &reopened,
            false,
            "execution_get",
            json!({"request_id":"fictional-command"}),
        )
        .await;
        assert_eq!(receipt["isError"], false);
        assert_eq!(receipt["structuredContent"]["data"]["status"], "unknown");
        assert_eq!(
            receipt["structuredContent"]["data"]["attempts"][0]["phase"],
            "possibly_sent"
        );
    }
}
