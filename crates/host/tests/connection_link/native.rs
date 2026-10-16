use super::*;
use homeverb_host::{ConfirmConnectionLinkRequest, PrepareConnectionLinkRequest};
use serde_json::Value;

fn prepare(f: &Fixture) -> PrepareConnectionLinkRequest {
    serde_json::from_value(json!({
        "schema_version":"1.0", "operation_id":"native-prepare", "timeout_ms":1000,
        "device_id":DEVICE, "anchor_id":ANCHOR, "connection":f.connection()
    }))
    .unwrap()
}

fn confirm(link_id: u64) -> ConfirmConnectionLinkRequest {
    serde_json::from_value(json!({
        "schema_version":"1.0", "operation_id":"native-confirm", "timeout_ms":1000, "link_id":link_id
    })).unwrap()
}

fn validate(definition: &str, value: &Value) {
    let mut options = jsonschema::options();
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".schema.json"))
        else {
            continue;
        };
        let document = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        options = options.with_resource(
            format!("urn:homeverb:schema:{name}"),
            jsonschema::Resource::from_contents(document),
        );
    }
    options.build(&json!({"$ref":format!("urn:homeverb:schema:native-connection-link#/$defs/{definition}")}))
        .unwrap().validate(value).unwrap();
}

fn id(value: &Value) -> u64 {
    assert_eq!(value["error"], Value::Null, "{value}");
    validate("prepare_result", value);
    value["data"]["link_id"].as_u64().unwrap()
}

#[tokio::test(start_paused = true)]
async fn native_bridge_preserves_exact_scope_and_publishes_only_after_confirmation() {
    let f = Fixture::new(vec![Transport::Lan]);
    f.store
        .set_preferences(DEVICE, &preferences(Some("Local alias")))
        .unwrap();
    let session = Session::new(Core::new(
        f.store.clone(),
        vec![f.adapter.clone()],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: DEVICE.into(),
                component_id: "main".into(),
                capability: "power".into(),
                action: "set".into(),
                transports: vec![Transport::Lan],
                argument_schema: json!({"const":{"state":false}}),
            }],
        },
    ));
    let initial = json!(f.core.describe(DEVICE).unwrap());
    let pending = session.prepare_native_connection_link(prepare(&f)).await;
    let identifier = id(&pending);
    assert_eq!(pending["data"]["descriptor"], initial);
    assert_eq!(
        pending["data"]["anchor"],
        initial["device"]["connections"][0]
    );
    assert_eq!(pending["data"]["connection"], json!(f.connection()));
    assert_eq!(
        pending["data"]["transport_grants"],
        json!([{
            "device_id":DEVICE,"component_id":"main","capability":"power","action":"set",
            "transports":["lan"],"argument_schema":{"const":{"state":false}}
        }])
    );
    f.assert_unlinked();
    let other = Session::new(f.other_core());
    assert_eq!(
        other
            .confirm_native_connection_link(confirm(identifier))
            .await["error"]["code"],
        "CONNECTION_LINK_INVALID"
    );
    assert!(!other.discard_native_connection_link(identifier));
    assert!(!session.discard_native_lan_scan(identifier));
    assert!(!session.discard_native_approval(identifier));
    let result = session
        .confirm_native_connection_link(confirm(identifier))
        .await;
    validate("confirm_result", &result);
    let mut expected = initial;
    expected["device"]["connections"]
        .as_array_mut()
        .unwrap()
        .push(pending["data"]["connection"].clone());
    assert_eq!(result["data"], expected);
    assert_eq!(json!(f.core.describe(DEVICE).unwrap()), expected);
    assert_eq!(
        f.store.preferences().unwrap()[0].room_id.as_deref(),
        Some("room-test")
    );
    assert!(!session.discard_native_connection_link(identifier));
    assert_eq!(
        session
            .confirm_native_connection_link(confirm(identifier))
            .await["error"]["code"],
        "CONNECTION_LINK_INVALID"
    );
    f.core.discover(&operation()).await.unwrap();
    assert_eq!(
        json!(SqliteStore::open(&f.directory).unwrap().devices().unwrap()[0]),
        expected
    );
}

#[tokio::test(start_paused = true)]
async fn native_bridge_bounds_capacity_prunes_expiry_and_dismisses_without_mutation() {
    let f = Fixture::new(vec![]);
    let session = Session::new(f.other_core());
    let mut ids = Vec::new();
    for _ in 0..8 {
        ids.push(id(&session
            .prepare_native_connection_link(prepare(&f))
            .await));
    }
    assert_eq!(
        session.prepare_native_connection_link(prepare(&f)).await["error"]["code"],
        "RESOURCE_BUSY"
    );
    assert!(session.discard_native_connection_link(ids.pop().unwrap()));
    ids.push(id(&session
        .prepare_native_connection_link(prepare(&f))
        .await));
    tokio::time::advance(std::time::Duration::from_secs(60)).await;
    assert_eq!(
        session
            .confirm_native_connection_link(confirm(ids[0]))
            .await["error"]["code"],
        "CONNECTION_LINK_EXPIRED"
    );
    let fresh = id(&session.prepare_native_connection_link(prepare(&f)).await);
    for old in ids {
        assert!(!session.discard_native_connection_link(old));
    }
    assert!(session.discard_native_connection_link(fresh));
    assert!(!session.discard_native_connection_link(fresh));
    f.assert_unlinked();
}

#[tokio::test(start_paused = true)]
async fn native_confirmation_consumes_on_invalid_metadata_or_withdrawn_evidence() {
    for case in [
        "version",
        "operation",
        "timeout",
        "metadata",
        "evidence",
        "account",
        "background",
        "close",
    ] {
        let f = Fixture::new(vec![]);
        let session = Session::new(f.other_core());
        let identifier = id(&session.prepare_native_connection_link(prepare(&f)).await);
        let mut input = confirm(identifier);
        match case {
            "version" => input.schema_version = "2.0".into(),
            "operation" => input.operation_id.clear(),
            "timeout" => input.timeout_ms = 0,
            "metadata" => {
                f.store
                    .set_preferences(DEVICE, &preferences(Some("Changed")))
                    .unwrap();
            }
            "evidence" => f.adapter.fault.store(1, Ordering::SeqCst),
            "account" => f.adapter.fault.store(4, Ordering::SeqCst),
            "background" => {
                session.set_foreground(false);
                session.set_foreground(true);
            }
            _ => session.close(),
        }
        let result = session.confirm_native_connection_link(input).await;
        validate("confirm_result", &result);
        assert!(result["error"].is_object(), "{case}: {result}");
        assert!(!session.discard_native_connection_link(identifier));
        assert_eq!(
            session
                .confirm_native_connection_link(confirm(identifier))
                .await["error"]["code"],
            "CONNECTION_LINK_INVALID"
        );
        f.assert_unlinked();
    }
}

#[tokio::test(start_paused = true)]
async fn native_link_concurrency_cancellation_and_lifecycle_release_without_reusing_consent() {
    for stage in ["prepare", "confirm"] {
        for stop in ["cancel", "background", "close", "deadline", "drop"] {
            let f = Fixture::new(vec![]);
            let session = Session::new(f.other_core());
            let identifier = id(&session.prepare_native_connection_link(prepare(&f)).await);
            f.adapter.fault.store(6, Ordering::SeqCst);
            let mut work = Box::pin(async {
                if stage == "prepare" {
                    session.prepare_native_connection_link(prepare(&f)).await
                } else {
                    session
                        .confirm_native_connection_link(confirm(identifier))
                        .await
                }
            });
            tokio::select! {
                _ = f.adapter.started.notified() => (),
                value = &mut work => panic!("Expected pending evidence: {value}"),
            }
            if stage == "confirm" {
                assert!(!session.discard_native_connection_link(identifier));
                assert_eq!(
                    session
                        .confirm_native_connection_link(confirm(identifier))
                        .await["error"]["code"],
                    "CONNECTION_LINK_INVALID"
                );
            }
            if stop == "drop" {
                drop(work);
            } else {
                match stop {
                    "cancel" => assert!(session.cancel(if stage == "prepare" {
                        "native-prepare"
                    } else {
                        "native-confirm"
                    })),
                    "background" => {
                        session.set_foreground(false);
                        session.set_foreground(true);
                    }
                    "close" => session.close(),
                    _ => tokio::time::advance(std::time::Duration::from_secs(1)).await,
                }
                assert_eq!(
                    work.await["error"]["code"],
                    if stop == "deadline" {
                        "DEADLINE_EXCEEDED"
                    } else {
                        "CANCELLED"
                    }
                );
            }
            f.assert_unlinked();
            assert!(!session.cancel("native-prepare"));
            assert!(!session.cancel("native-confirm"));
            drop(f.store.lock_device(DEVICE).unwrap());
            f.adapter.fault.store(0, Ordering::SeqCst);
            if stage == "prepare" {
                session.discard_native_connection_link(identifier);
            }
            if stop != "close" {
                let fresh = id(&session.prepare_native_connection_link(prepare(&f)).await);
                assert!(session.discard_native_connection_link(fresh));
            }
        }
    }
}

#[test]
fn native_link_metadata_and_ordinary_dispatch_reject_forged_authority() {
    let f = Fixture::new(vec![]);
    let examples: Value = serde_json::from_str(include_str!(
        "../../../../packages/native-contract/connection-link-examples.json"
    ))
    .unwrap();
    for name in ["prepare", "confirm", "prepare_result", "confirm_result"] {
        validate(name, &examples[name]);
    }
    for field in [
        "confirmed",
        "principal",
        "endpoint",
        "token",
        "appliance_id",
    ] {
        let mut value = examples["prepare"].clone();
        value[field] = json!("FORBIDDEN");
        assert!(serde_json::from_value::<PrepareConnectionLinkRequest>(value).is_err());
        let mut value = examples["confirm"].clone();
        value[field] = json!("FORBIDDEN");
        assert!(serde_json::from_value::<ConfirmConnectionLinkRequest>(value).is_err());
        let mut value = examples["prepare"].clone();
        value["connection"][field] = json!("FORBIDDEN");
        assert!(serde_json::from_value::<PrepareConnectionLinkRequest>(value).is_err());
    }
    for method in [
        "prepare_connection_link",
        "confirm_connection_link",
        "discard_connection_link",
    ] {
        assert!(serde_json::from_value::<HostRequest>(json!({"schema_version":"1.0","operation_id":"ordinary","timeout_ms":1000,"request":{"method":method}})).is_err());
    }
    f.assert_unlinked();
}
