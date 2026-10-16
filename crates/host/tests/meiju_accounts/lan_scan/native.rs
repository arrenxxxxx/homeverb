use super::*;
use homeverb_host::{ConfirmLanScanRequest, PrepareLanScanRequest};

fn prepare_input(descriptor: &Descriptor) -> PrepareLanScanRequest {
    serde_json::from_value(json!({
        "schema_version":"1.0", "operation_id":"native-preview", "timeout_ms":1000,
        "device_id":descriptor.device.id, "connection_id":descriptor.device.connections[0].id
    }))
    .unwrap()
}

fn confirm_input(id: u64) -> ConfirmLanScanRequest {
    serde_json::from_value(json!({
        "schema_version":"1.0", "operation_id":"native-scan", "timeout_ms":1000,
        "scan_id":id
    }))
    .unwrap()
}

async fn prepare(f: &Fixture, descriptor: &Descriptor) -> Value {
    f.session
        .prepare_native_lan_scan(prepare_input(descriptor), 0xc0a82a02, 24)
        .await
}

fn scan_id(value: &Value) -> u64 {
    assert_eq!(value["error"], Value::Null, "{value}");
    validate_data("preview", &value["data"]);
    value["data"]["scan_id"].as_u64().unwrap()
}

fn validate_data(definition: &str, data: &Value) {
    let schema: Value = serde_json::from_str(include_str!(
        "../../../../../schemas/native-lan-scan.schema.json"
    ))
    .unwrap();
    jsonschema::validator_for(&schema["$defs"][definition])
        .unwrap()
        .validate(data)
        .unwrap();
}

#[tokio::test(start_paused = true)]
async fn bridge_reports_only_selected_scope_without_identifiers_or_mutation() {
    for status in [
        "not_observed",
        "incomplete",
        "ambiguous",
        "unique_candidate",
    ] {
        let (f, discovery, descriptor, id) = setup("AC", "KFR-35G/WF2").await;
        let mut reply = report(vec![hint(&id, ApplianceFamily::AirConditioner, 3)]);
        match status {
            "not_observed" => reply = report(vec![]),
            "incomplete" => reply.limit_reached = true,
            "ambiguous" => {
                reply
                    .candidates
                    .push(hint(&id, ApplianceFamily::AirConditioner, 4));
                reply.received_datagrams = 2;
            }
            _ => {}
        }
        *discovery.reply.lock().unwrap() = Some(reply);
        let calls = f.calls();
        let cards = json!(f.store.devices().unwrap());
        let secrets = f.secrets.values.lock().unwrap().clone();
        let pending = prepare(&f, &descriptor).await;
        let identifier = scan_id(&pending);
        assert_eq!(
            pending["data"],
            json!({
                "scan_id":identifier, "device_id":descriptor.device.id,
                "connection_id":descriptor.device.connections[0].id,
                "alias":descriptor.device.alias,
                "network":{"local_address":"192.168.42.2", "prefix_length":24},
                "expires_in_ms":60000
            })
        );
        assert_eq!(discovery.calls.load(Ordering::Relaxed), 0);
        let other = session(&f, discovery.clone());
        assert_eq!(
            other
                .confirm_native_lan_scan(confirm_input(identifier))
                .await["error"]["code"],
            "LAN_SCAN_INVALID"
        );
        assert!(!other.discard_native_lan_scan(identifier));
        let result = f
            .session
            .confirm_native_lan_scan(confirm_input(identifier))
            .await;
        assert_eq!(result["error"], Value::Null);
        validate_data("outcome", &result["data"]);
        assert_eq!(
            result["data"],
            json!({
                "device_id":descriptor.device.id, "connection_id":descriptor.device.connections[0].id,
                "status":status, "authenticated":false, "route_added":false
            })
        );
        assert!(!result.to_string().contains(id.expose()));
        assert!(!result.to_string().contains("192.168.42.3"));
        assert_eq!(
            f.session
                .confirm_native_lan_scan(confirm_input(identifier))
                .await["error"]["code"],
            "LAN_SCAN_INVALID"
        );
        assert!(!f.session.discard_native_lan_scan(identifier));
        assert_eq!(discovery.calls.load(Ordering::Relaxed), 1);
        assert_eq!(f.calls(), calls);
        assert_eq!(json!(f.store.devices().unwrap()), cards);
        assert_eq!(*f.secrets.values.lock().unwrap(), secrets);
    }
}

#[tokio::test(start_paused = true)]
async fn bridge_bounds_and_consumes_previews_before_failed_admission() {
    let (f, discovery, descriptor, _) = setup("AC", "KFR-35G/WF2").await;
    let mut ids = Vec::new();
    for _ in 0..8 {
        ids.push(scan_id(&prepare(&f, &descriptor).await));
    }
    assert_eq!(
        prepare(&f, &descriptor).await["error"]["code"],
        "RESOURCE_BUSY"
    );
    let dismissed = ids.pop().unwrap();
    assert!(f.session.discard_native_lan_scan(dismissed));
    assert!(!f.session.discard_native_lan_scan(dismissed));
    ids.push(scan_id(&prepare(&f, &descriptor).await));
    tokio::time::advance(Duration::from_secs(60)).await;
    assert_eq!(
        f.session
            .confirm_native_lan_scan(confirm_input(ids[0]))
            .await["error"]["code"],
        "LAN_SCAN_EXPIRED"
    );
    let fresh = scan_id(&prepare(&f, &descriptor).await);
    for id in ids {
        assert!(!f.session.discard_native_lan_scan(id));
    }
    assert!(f.session.discard_native_lan_scan(fresh));
    for mode in [
        "version",
        "operation",
        "timeout",
        "changed",
        "background",
        "close",
    ] {
        let identifier = scan_id(&prepare(&f, &descriptor).await);
        let mut input = confirm_input(identifier);
        match mode {
            "version" => input.schema_version = "2.0".into(),
            "operation" => input.operation_id.clear(),
            "timeout" => input.timeout_ms = 0,
            "changed" => {
                edit_binding(&f, &descriptor.device.id, |b| b["listed"] = json!(false));
            }
            "background" => {
                f.session.set_foreground(false);
                f.session.set_foreground(true);
            }
            _ => f.session.close(),
        }
        let result = f.session.confirm_native_lan_scan(input).await;
        assert_ne!(result["error"], Value::Null, "{mode}");
        assert!(!f.session.discard_native_lan_scan(identifier));
        if mode == "changed" {
            edit_binding(&f, &descriptor.device.id, |b| b["listed"] = json!(true));
        }
    }
    assert_eq!(discovery.calls.load(Ordering::Relaxed), 0);
}

#[tokio::test(start_paused = true)]
async fn bridge_active_scan_cancels_without_restoring_consumed_consent() {
    for mode in ["cancel", "background", "close", "deadline", "drop"] {
        let (f, discovery, descriptor, _) = setup("AC", "KFR-35G/WF2").await;
        discovery.stall.store(true, Ordering::Relaxed);
        let identifier = scan_id(&prepare(&f, &descriptor).await);
        let session = f.session.clone();
        let task = tokio::spawn(async move {
            session
                .confirm_native_lan_scan(confirm_input(identifier))
                .await
        });
        discovery.started.notified().await;
        assert!(!f.session.discard_native_lan_scan(identifier));
        assert_eq!(
            f.session
                .confirm_native_lan_scan(confirm_input(identifier))
                .await["error"]["code"],
            "LAN_SCAN_INVALID"
        );
        match mode {
            "cancel" => assert!(f.session.cancel("native-scan")),
            "background" => f.session.set_foreground(false),
            "close" => f.session.close(),
            "deadline" => tokio::time::advance(Duration::from_secs(1)).await,
            _ => task.abort(),
        }
        if mode == "drop" {
            assert!(task.await.unwrap_err().is_cancelled());
        } else {
            let result = task.await.unwrap();
            assert_eq!(
                result["error"]["code"],
                if mode == "deadline" {
                    "DEADLINE_EXCEEDED"
                } else {
                    "CANCELLED"
                }
            );
        }
        assert!(!f.session.cancel("native-scan"));
        assert_eq!(discovery.calls.load(Ordering::Relaxed), 1);
        let account = f.store.accounts().unwrap()[0].id;
        assert!(f.vault.lease(account, &operation()).await.is_ok());
        if mode == "background" {
            f.session.set_foreground(true);
        }
        if mode != "close" {
            assert_eq!(
                f.session
                    .confirm_native_lan_scan(confirm_input(identifier))
                    .await["error"]["code"],
                "LAN_SCAN_INVALID"
            );
        }
    }
}

#[tokio::test(start_paused = true)]
async fn bridge_rejects_networks_and_ordinary_forged_requests() {
    let (f, discovery, descriptor, _) = setup("AC", "KFR-35G/WF2").await;
    for (address, prefix) in [(0x7f000001, 24), (0xc0a82a00, 24), (0xc0a82a02, 31)] {
        assert_eq!(
            f.session
                .prepare_native_lan_scan(prepare_input(&descriptor), address, prefix)
                .await["error"]["code"],
            "INVALID_ARGUMENT"
        );
    }
    let mut input = prepare_input(&descriptor);
    input.schema_version = "2.0".into();
    assert_eq!(
        f.session
            .prepare_native_lan_scan(input, 0xc0a82a02, 24)
            .await["error"]["code"],
        "INVALID_ARGUMENT"
    );
    for method in ["prepare_lan_scan", "confirm_lan_scan", "discard_lan_scan"] {
        assert!(serde_json::from_value::<HostRequest>(json!({"schema_version":"1.0", "operation_id":"forged", "timeout_ms":1000, "request":{"method":method}})).is_err());
    }
    assert_eq!(discovery.calls.load(Ordering::Relaxed), 0);
}

#[tokio::test(start_paused = true)]
async fn bridge_preparation_cannot_publish_after_background_foreground_race() {
    let (f, discovery, descriptor, _) = setup("AC", "KFR-35G/WF2").await;
    f.secrets.read_delay.store(true, Ordering::Relaxed);
    let pending = prepare(&f, &descriptor);
    tokio::pin!(pending);
    tokio::select! {
        _ = f.secrets.read_started.notified() => (),
        result = &mut pending => panic!("Expected secret read: {result:?}"),
    }
    f.session.set_foreground(false);
    f.session.set_foreground(true);
    assert_eq!(pending.await["error"]["code"], "CANCELLED");
    f.secrets.read_delay.store(false, Ordering::Relaxed);
    assert!(!f.session.cancel("native-preview"));
    let id = scan_id(&prepare(&f, &descriptor).await);
    assert!(f.session.discard_native_lan_scan(id));
    assert_eq!(discovery.calls.load(Ordering::Relaxed), 0);
}

#[tokio::test(start_paused = true)]
async fn scan_ids_and_appliance_approval_ids_do_not_alias() {
    let (f, _, descriptor, _) = setup("AC", "KFR-35G/WF2").await;
    let scan = scan_id(&prepare(&f, &descriptor).await);
    let directory = f.path.join("mock-preview");
    let core =
        homeverb_host::open_mock(&directory, homeverb_mock::Scenario::ApprovalRequired, true)
            .unwrap();
    core.discover(&operation()).await.unwrap();
    let session = Session::new(core);
    let mut command: Value = serde_json::from_str(include_str!(
        "../../../../../examples/command.approved-cycle.json"
    ))
    .unwrap();
    command["device_id"] = json!("dev-mock-ac");
    let prepared = session
        .prepare_native_approval(
            serde_json::from_value(json!({
                "schema_version":"1.0", "operation_id":"approval", "command":command
            }))
            .unwrap(),
        )
        .await;
    assert_eq!(prepared["error"], Value::Null, "{prepared}");
    let approval = prepared["data"]["approval_id"].as_u64().unwrap();
    assert_ne!(scan, approval);
    assert!(!session.discard_native_approval(scan));
    assert!(!f.session.discard_native_lan_scan(approval));
    assert!(session.discard_native_approval(approval));
    assert!(f.session.discard_native_lan_scan(scan));
}
