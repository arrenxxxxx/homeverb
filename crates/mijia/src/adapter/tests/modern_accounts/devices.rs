//! Modern device adoption uses fictional accounts and actual Core/SQLite plumbing.
use super::*;
use homeverb_core::Grant;
use homeverb_domain::{Command, Phase, Status, WaitFor};

fn composed(f: &Fixture, peer: &Arc<Peer>) -> Arc<MijiaAdapter> {
    Arc::new(
        adapter(f, peer)
            .with_fan_controls()
            .with_reference_controls()
            .with_outlet_controls(),
    )
}
fn core(f: &Fixture, adapter: Arc<MijiaAdapter>, policy: Policy) -> Core {
    Core::new(
        f.store.clone(),
        vec![adapter],
        Arc::new(SystemClock),
        policy,
    )
}
async fn adopt(f: &Fixture, adapter: &MijiaAdapter, id: Uuid) -> Device {
    let report = adapter
        .refresh_modern_devices(id, &operation())
        .await
        .unwrap();
    assert!(!report.complete);
    f.publish(&report).await;
    report.descriptors.unwrap().remove(0).device
}
fn wire(id: &str, kind: u32, parent: Option<&str>) -> Value {
    json!({"did":id,"pid":kind,"parent_id":parent,"name":"Fictional grouped fan","model":"jdjz.fan.2072fc"})
}

#[tokio::test]
async fn modern_root_publishes_and_reopens_core_reads_without_legacy_or_identity_leaks() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    let record = connect(&a).await.account;
    let device = adopt(&f, &a, record.id).await;
    let stored = payload(&f, &record);
    assert_eq!(stored["version"], 3);
    assert_eq!(
        stored["bindings"][0]["read_protocol"],
        "modern_reference_v1"
    );
    assert_eq!(stored["bindings"][0]["modern"]["kind"], 0);
    let c = core(&f, composed(&f, &p), Policy::default());
    let state = c
        .read(&device.id, "fan-2", Route::Cloud, false, &operation())
        .await
        .unwrap();
    assert_eq!(state.observations["power.state"].value, false);
    assert!(
        state
            .observations
            .values()
            .all(|v| v.stale && v.observed_at.is_none())
    );
    assert!(
        c.describe(&device.id)
            .unwrap()
            .capabilities
            .iter()
            .all(|v| v.actions.is_empty())
    );
    assert!(f.cloud.calls.lock().unwrap().is_empty());
    let count = puts(&f);
    let again = adopt(&f, &a, record.id).await;
    assert_eq!(again.id, device.id);
    assert_eq!(again.connections, device.connections);
    assert_eq!(puts(&f), count);
    assert!(
        p.property_requests
            .lock()
            .unwrap()
            .iter()
            .all(|q| q["did"] == "PRIVATE-MODERN-DEVICE")
    );
    let public = serde_json::to_string(&c.list().unwrap()).unwrap()
        + &serde_json::to_string(&state).unwrap();
    for marker in [
        "PRIVATE-MODERN-DEVICE",
        "PRIVATE-MODERN-TOKEN",
        "PRIVATE-MODERN-USER",
    ] {
        assert!(!public.contains(marker));
        for file in std::fs::read_dir(&f.directory.0).unwrap() {
            let path = file.unwrap().path();
            if path.is_file() {
                assert!(
                    !std::fs::read(path)
                        .unwrap()
                        .windows(marker.len())
                        .any(|s| s == marker.as_bytes())
                );
            }
        }
    }
}

#[tokio::test]
async fn logical_children_never_get_cards_and_missing_or_unknown_entries_remain_partial() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    *p.devices.lock().unwrap() = vec![
        wire("parent", 18, Some("gateway")),
        wire("left", 21, Some("parent")),
        wire("right", 21, Some("parent")),
        wire("orphan", 21, None),
    ];
    let record = connect(&a).await.account;
    let report = a
        .refresh_modern_devices(record.id, &operation())
        .await
        .unwrap();
    assert_eq!(report.descriptors.as_ref().unwrap().len(), 1);
    assert!(!report.issues.is_empty());
    f.publish(&report).await;
    assert_eq!(f.core.list().unwrap().len(), 1);
    let before = payload(&f, &record);
    assert_eq!(
        before["bindings"][0]["modern"]["logical_children"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    p.devices.lock().unwrap().clear();
    let report = a
        .refresh_modern_devices(record.id, &operation())
        .await
        .unwrap();
    assert!(report.descriptors.as_ref().unwrap().is_empty());
    f.publish(&report).await;
    assert_eq!(payload(&f, &record), before);
    assert_eq!(f.core.list().unwrap().len(), 1);
    let mut unknown = wire("unknown", 0, None);
    unknown["model"] = json!("fiction.unknown.v1");
    *p.devices.lock().unwrap() = vec![unknown];
    let report = a
        .refresh_modern_devices(record.id, &operation())
        .await
        .unwrap();
    assert_eq!(
        report.descriptors.unwrap()[0].device.support_status,
        "unsupported"
    );
}

#[tokio::test]
async fn explicit_protocol_selection_preserves_local_identity_and_preferences() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    *f.cloud.devices.lock().unwrap() = p.devices.lock().unwrap().clone();
    let legacy = a
        .connect_devices(credentials(), None, "Home", &operation())
        .await
        .unwrap();
    f.publish(&legacy).await;
    let old = f.core.list().unwrap().remove(0);
    f.store.create_room("study", "Study").unwrap();
    f.store
        .set_preferences(
            &old.id,
            &PreferencesInput {
                alias: Some("My fan".into()),
                room_id: Some("study".into()),
                favorite: true,
            },
        )
        .unwrap();
    let record = connect(&a).await.account;
    assert_eq!(record.id, legacy.account.id);
    let modern = adopt(&f, &a, record.id).await;
    assert_eq!(old.id, modern.id);
    assert_eq!(old.connections, modern.connections);
    let c = core(&f, a.clone(), Policy::default());
    let before = f.cloud.calls.lock().unwrap().len();
    c.read(&old.id, "fan-2", Route::Cloud, false, &operation())
        .await
        .unwrap();
    assert_eq!(f.cloud.calls.lock().unwrap().len(), before);
    let report = a.refresh_devices(record.id, &operation()).await.unwrap();
    f.publish(&report).await;
    let count = p.reads.lock().unwrap().len();
    c.read(&old.id, "fan-2", Route::Cloud, false, &operation())
        .await
        .unwrap();
    assert_eq!(p.reads.lock().unwrap().len(), count);
    let saved = payload(&f, &record);
    assert!(saved["bindings"][0].get("read_protocol").is_none());
    assert!(saved["bindings"][0]["modern"].is_object());
    assert_eq!(c.list().unwrap()[0].id, old.id);
    assert_eq!(
        f.store
            .preferences()
            .unwrap()
            .iter()
            .find(|p| p.device_id == old.id)
            .unwrap()
            .alias
            .as_deref(),
        Some("My fan")
    );
    let preferences = f.store.preferences().unwrap();
    let saved = preferences.iter().find(|p| p.device_id == old.id).unwrap();
    assert_eq!(saved.room_id.as_deref(), Some("study"));
    assert!(saved.favorite);
}

#[tokio::test]
async fn role_model_parent_and_cross_account_conflicts_preserve_saved_bytes() {
    for change in ["model", "kind", "parent", "child-root", "child-move"] {
        let f = Fixture::new();
        let p = Peer::new();
        let a = composed(&f, &p);
        *p.devices.lock().unwrap() = vec![
            wire("root", 18, Some("gateway")),
            wire("child", 21, Some("root")),
        ];
        let record = connect(&a).await.account;
        adopt(&f, &a, record.id).await;
        let before = bytes(&f, &record);
        let mut next = vec![
            wire("root", 18, Some("gateway")),
            wire("child", 21, Some("root")),
        ];
        match change {
            "model" => {
                next[0]["model"] = json!("fiction.other.v1");
                next.pop();
            }
            "kind" => next[0]["pid"] = json!(0),
            "parent" => next[0]["parent_id"] = json!("new-gateway"),
            "child-root" => {
                next[1]["pid"] = json!(0);
                next[1]["parent_id"] = Value::Null;
            }
            _ => {
                next.push(wire("other", 0, None));
                next[1]["parent_id"] = json!("other");
            }
        }
        *p.devices.lock().unwrap() = next;
        let report = a
            .refresh_modern_devices(record.id, &operation())
            .await
            .unwrap();
        assert_eq!(
            report.descriptors.unwrap_err().code,
            "DEVICE_IDENTITY_CONFLICT",
            "{change}"
        );
        assert_eq!(bytes(&f, &record), before);
    }
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    let record = connect(&a).await.account;
    adopt(&f, &a, record.id).await;
    *p.subject.lock().unwrap() = "999002".into();
    let other = connect(&a).await.account;
    let report = a
        .refresh_modern_devices(other.id, &operation())
        .await
        .unwrap();
    assert_eq!(
        report.descriptors.unwrap_err().code,
        "DEVICE_IDENTITY_CONFLICT"
    );
}

#[tokio::test]
async fn uncertain_binding_save_recovers_exact_bytes_without_authentication_or_rewrite() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    let record = connect(&a).await.account;
    f.secrets.mode.store(2, Ordering::SeqCst);
    let report = a
        .refresh_modern_devices(record.id, &operation())
        .await
        .unwrap();
    assert_eq!(
        report.descriptors.unwrap_err().code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert_eq!(report.account.credential_state, CredentialState::Pending);
    assert!(f.core.list().unwrap().is_empty());
    let before = bytes(&f, &record);
    let put_count = puts(&f);
    let auth = p.auth.lock().unwrap().len();
    f.secrets.mode.store(0, Ordering::SeqCst);
    let recovered = a
        .recover_modern_devices(record.id, &operation())
        .await
        .unwrap();
    assert_eq!(recovered.account.credential_state, CredentialState::Stored);
    assert_eq!(bytes(&f, &record), before);
    assert_eq!(puts(&f), put_count);
    assert_eq!(p.auth.lock().unwrap().len(), auth);
    f.publish(&recovered).await;
    assert_eq!(f.core.list().unwrap().len(), 1);
}

#[tokio::test]
async fn modern_read_failure_never_falls_back_and_dropped_refresh_releases_leases() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    let record = connect(&a).await.account;
    let device = adopt(&f, &a, record.id).await;
    let c = core(&f, a.clone(), Policy::default());
    p.fail_read.store(1, Ordering::SeqCst);
    assert!(
        c.read(&device.id, "fan-2", Route::Cloud, false, &operation())
            .await
            .is_err()
    );
    assert!(f.cloud.calls.lock().unwrap().is_empty());
    p.fail_read.store(0, Ordering::SeqCst);
    p.pause.store(2, Ordering::SeqCst);
    let op = operation();
    let mut work = Box::pin(a.refresh_modern_devices(record.id, &op));
    tokio::select! { _=p.started.notified()=>{}, _=&mut work=>panic!("Expected blocked directory") }
    drop(work);
    p.pause.store(0, Ordering::SeqCst);
    assert!(
        a.refresh_modern_devices(record.id, &operation())
            .await
            .unwrap()
            .descriptors
            .is_ok()
    );
}

#[tokio::test]
async fn stale_control_descriptors_and_grants_cannot_write_a_modern_selected_binding() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    *f.cloud.devices.lock().unwrap() = p.devices.lock().unwrap().clone();
    let old = a
        .connect_devices(credentials(), None, "Home", &operation())
        .await
        .unwrap();
    f.publish(&old).await;
    let device = f.core.list().unwrap().remove(0);
    connect(&a).await;
    a.refresh_modern_devices(old.account.id, &operation())
        .await
        .unwrap()
        .descriptors
        .unwrap();
    assert!(
        a.inspect_control_options(&device, &device.connections[0], &operation())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        !a.history_options(&device, &device.connections[0], &operation())
            .await
            .unwrap()
            .is_empty()
    );
    let c = core(
        &f,
        a.clone(),
        Policy {
            grants: vec![Grant {
                device_id: device.id.clone(),
                component_id: "fan-2".into(),
                capability: "power".into(),
                action: "set".into(),
                transports: vec![Transport::Cloud],
                argument_schema: json!({"const":{"state":false}}),
            }],
        },
    );
    let before = f.cloud.calls.lock().unwrap().len();
    let receipt = c
        .execute(
            Command {
                schema_version: "1.0".into(),
                request_id: "fictional-stale-write".into(),
                device_id: device.id,
                component_id: "fan-2".into(),
                capability: "power".into(),
                action: "set".into(),
                arguments: json!({"state":false}),
                connection: Route::Cloud,
                timeout_ms: 10000,
                wait_for: WaitFor::Accepted,
            },
            CancellationToken::new(),
        )
        .await;
    assert_eq!(receipt.status, Status::Failed);
    assert_eq!(receipt.error.unwrap().code, "CAPABILITY_UNSUPPORTED");
    assert_eq!(receipt.attempts[0].phase, Phase::NotSent);
    assert_eq!(f.cloud.calls.lock().unwrap().len(), before);
}

#[tokio::test]
async fn protected_v3_tampering_and_downgrades_refuse_before_any_provider_request() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    let record = connect(&a).await.account;
    let device = adopt(&f, &a, record.id).await;
    let baseline_bytes = bytes(&f, &record);
    let baseline = payload(&f, &record);
    let c = core(&f, a.clone(), Policy::default());
    let count = p.reads.lock().unwrap().len();
    for (pointer, value) in [
        ("/version", json!(1)),
        ("/version", json!(2)),
        ("/bindings/0/read_protocol", json!("arbitrary")),
        ("/bindings/0/read_protocol", json!("legacy")),
        ("/bindings/0/modern", Value::Null),
        ("/bindings/0/modern/kind", json!(21)),
        ("/bindings/0/modern/parent", json!("PRIVATE-MODERN-DEVICE")),
        (
            "/bindings/0/modern/logical_children",
            json!(["PRIVATE-MODERN-DEVICE"]),
        ),
        (
            "/bindings/0/modern/logical_children",
            json!(["duplicate", "duplicate"]),
        ),
        ("/bindings/0/mapping/properties/0/piid", json!(999)),
        ("/modern", Value::Null),
    ] {
        let mut bad = baseline.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        f.secrets.values.lock().unwrap().insert(
            record.credential_ref.to_string(),
            serde_json::to_vec(&bad).unwrap(),
        );
        assert!(
            c.read(&device.id, "fan-2", Route::Cloud, false, &operation())
                .await
                .is_err(),
            "{pointer}"
        );
        assert_eq!(p.reads.lock().unwrap().len(), count);
    }
    f.secrets
        .values
        .lock()
        .unwrap()
        .insert(record.credential_ref.to_string(), baseline_bytes);
    let before = bytes(&f, &record);
    let decoded = Payload::decode(&SecretBytes::new(before.clone()), &record).unwrap();
    assert!(decoded.encode(&record).unwrap().expose() == before);
    a.renew_modern_account(record.id, &operation())
        .await
        .unwrap();
    assert_eq!(payload(&f, &record)["version"], 3);
    assert_eq!(payload(&f, &record)["bindings"], baseline["bindings"]);
}

#[tokio::test]
async fn all_pinned_scalar_mappings_read_through_modern_core_routes() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    let profiles: Value =
        serde_json::from_str(include_str!("../../../reference_profiles.json")).unwrap();
    *p.devices.lock().unwrap()=profiles.as_array().unwrap().iter().enumerate().map(|(i,v)|json!({"did":format!("FICTIONAL-MODERN-{i}"),"pid":0,"name":"Fictional reference device","model":v["model"],"spec_type":v["spec_type"]})).collect();
    let record = connect(&a).await.account;
    let report = a
        .refresh_modern_devices(record.id, &operation())
        .await
        .unwrap();
    f.publish(&report).await;
    let c = core(&f, a.clone(), Policy::default());
    let mut observations = 0;
    for descriptor in report.descriptors.unwrap() {
        assert!(
            descriptor
                .capabilities
                .iter()
                .all(|cap| cap.actions.is_empty())
        );
        if descriptor.device.support_status == "unsupported" {
            continue;
        }
        for component in &descriptor.device.components {
            let snapshot = c
                .read(
                    &descriptor.device.id,
                    &component.id,
                    Route::Cloud,
                    false,
                    &operation(),
                )
                .await
                .unwrap();
            observations += snapshot.observations.len();
        }
    }
    assert_eq!(observations, 264);
    assert!(f.cloud.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn cancellation_during_binding_save_keeps_pending_bytes_and_no_cards() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    let record = connect(&a).await.account;
    let op = operation();
    let vault = Arc::new(AccountVault::new(
        f.store.clone(),
        Arc::new(CancelAfterPut {
            inner: f.secrets.clone(),
            cancellation: op.cancellation.clone(),
        }),
    ));
    let cancelling = MijiaAdapter::new(vault, f.cloud.clone(), Arc::new(SystemClock))
        .with_modern_protocol(p.clone(), p.clone());
    let report = cancelling
        .refresh_modern_devices(record.id, &op)
        .await
        .unwrap();
    assert_eq!(report.descriptors.unwrap_err().code, "CANCELLED");
    assert_eq!(report.account.credential_state, CredentialState::Pending);
    assert!(f.core.list().unwrap().is_empty());
    let before = bytes(&f, &record);
    let count = puts(&f);
    let recovered = a
        .recover_modern_devices(record.id, &operation())
        .await
        .unwrap();
    assert!(recovered.descriptors.is_ok());
    assert_eq!(bytes(&f, &record), before);
    assert_eq!(puts(&f), count);
}

#[tokio::test]
async fn legacy_refresh_cannot_recreate_a_modern_logical_child_as_another_device() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    *p.devices.lock().unwrap() = vec![wire("root", 18, None), wire("child", 21, Some("root"))];
    let record = connect(&a).await.account;
    adopt(&f, &a, record.id).await;
    *f.cloud.devices.lock().unwrap() = vec![wire("child", 0, None)];
    let report = a
        .connect_devices(credentials(), Some(record.id), "Unused", &operation())
        .await
        .unwrap();
    assert_eq!(
        report.descriptors.unwrap_err().code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    assert_eq!(f.core.list().unwrap().len(), 1);
    assert_eq!(
        payload(&f, &record)["bindings"].as_array().unwrap().len(),
        1
    );
}

#[tokio::test]
async fn modern_composition_is_required_after_restart_and_never_falls_back_to_legacy() {
    let f = Fixture::new();
    let p = Peer::new();
    let a = composed(&f, &p);
    let record = connect(&a).await.account;
    let device = adopt(&f, &a, record.id).await;
    let count = p.reads.lock().unwrap().len();
    let result = f
        .core
        .read(&device.id, "fan-2", Route::Cloud, false, &operation())
        .await
        .unwrap_err();
    assert_eq!(result.code, "CAPABILITY_UNSUPPORTED");
    assert_eq!(p.reads.lock().unwrap().len(), count);
    assert!(f.cloud.calls.lock().unwrap().is_empty());
}
