use super::*;
use homeverb_domain::{Phase, Status, Transport};

async fn fixture() -> (Fixture, Device) {
    let mut entry = appliances()[0].clone();
    entry["sn8"] = json!("TEST1234");
    entry["category"] = json!("wall_air_conditioner");
    let f = Fixture::new(vec![entry]).await;
    let device = f.discover().await.remove(0);
    (f, device)
}
fn core(f: &Fixture, device: &Device, enabled: bool) -> Core {
    let adapter = MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock));
    Core::new(
        f.store.clone(),
        vec![Arc::new(if enabled {
            adapter.with_ac_feature_control()
        } else {
            adapter
        })],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: device.id.clone(),
                component_id: "main".into(),
                capability: "vendor.midea.ac".into(),
                action: "*".into(),
                transports: vec![Transport::Cloud],
                argument_schema: json!({"type":"object"}),
            }],
        },
    )
}
// Explicit independent action grants; wildcard actions are deliberately not supported.
fn granted_core(f: &Fixture, device: &Device, enabled: bool) -> Core {
    let adapter = MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock));
    let names = [
        "eco",
        "breezeless",
        "horizontal_swing",
        "horizontal_direction",
        "display",
        "self_clean",
    ];
    Core::new(
        f.store.clone(),
        vec![Arc::new(if enabled {
            adapter.with_ac_feature_control()
        } else {
            adapter
        })],
        Arc::new(SystemClock),
        Policy {
            grants: names
                .into_iter()
                .map(|name| Grant {
                    device_id: device.id.clone(),
                    component_id: "main".into(),
                    capability: "vendor.midea.ac".into(),
                    action: format!("set_{name}"),
                    transports: vec![Transport::Cloud],
                    argument_schema: json!({"type":"object"}),
                })
                .collect(),
        },
    )
}
fn command(device: &Device, name: &str, value: Value) -> Command {
    let mut c = power_command(device, false);
    c.capability = "vendor.midea.ac".into();
    c.action = format!("set_{name}");
    c.arguments = json!({name:value});
    c
}

#[tokio::test]
async fn exact_reference_settings_follow_core_grants_prerequisites_and_uncertain_write_rules() {
    let mut entry = appliances()[0].clone();
    entry["sn8"] = json!("22040089");
    entry["category"] = json!("wall_air_conditioner");
    let f = Fixture::new(vec![entry]).await;
    let device = f.discover().await.remove(0);
    let names = ["cool_energy_saving", "strong_cooling", "light_sensing"];
    let c = Core::new(
        f.store.clone(),
        vec![Arc::new(
            MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock))
                .with_ac_feature_control(),
        )],
        Arc::new(SystemClock),
        Policy {
            grants: names
                .iter()
                .map(|name| Grant {
                    device_id: device.id.clone(),
                    component_id: "main".into(),
                    capability: "vendor.midea.ac".into(),
                    action: format!("set_{name}"),
                    transports: vec![Transport::Cloud],
                    argument_schema: json!({"const":{*name:true}}),
                })
                .collect(),
        },
    );
    c.discover(&operation()).await.unwrap();
    for (name, wire, value) in [
        ("cool_energy_saving", "cool_power_saving", 1),
        ("strong_cooling", "strong_cool", 1),
        ("light_sensing", "light_sensitive", 3),
    ] {
        *f.wire.state.lock().unwrap() = Some(json!({wire:0,"buzzer":"off"}));
        let cmd = command(&device, name, json!(true));
        let result = c.execute(cmd.clone(), CancellationToken::new()).await;
        assert_eq!(
            result.status,
            Status::Accepted,
            "{name}: {:?}",
            result.error
        );
        assert!(result.verification.is_none());
        assert_eq!(
            f.wire.controls.lock().unwrap().last().unwrap()["command"],
            json!({"control":{wire:value,"buzzer":"off"}})
        );
        let sends = f.wire.controls.lock().unwrap().len();
        let calls = f.calls();
        assert_eq!(
            json!(c.execute(cmd, CancellationToken::new()).await),
            json!(result)
        );
        assert_eq!(f.calls(), calls);
        assert_eq!(f.wire.controls.lock().unwrap().len(), sends);

        *f.wire.state.lock().unwrap() = Some(json!({wire:2}));
        let result = c
            .execute(
                command(&device, name, json!(true)),
                CancellationToken::new(),
            )
            .await;
        assert_eq!(result.status, Status::Failed);
        assert!(result.attempts.iter().all(|a| a.phase == Phase::NotSent));
        assert_eq!(f.wire.controls.lock().unwrap().len(), sends);

        *f.wire.state.lock().unwrap() = Some(json!({wire:0}));
        f.wire.control_fault.store(2, Ordering::Relaxed);
        let cmd = command(&device, name, json!(true));
        let result = c.execute(cmd.clone(), CancellationToken::new()).await;
        assert_eq!(result.status, Status::Unknown);
        let calls = f.calls();
        assert_eq!(
            json!(c.execute(cmd, CancellationToken::new()).await),
            json!(result)
        );
        assert_eq!(f.calls(), calls);
        assert_eq!(f.wire.controls.lock().unwrap().len(), sends + 1);
        f.wire.control_fault.store(0, Ordering::Relaxed);
    }
    assert!(c.plan(&command(&device, "eco", json!(true))).is_err());
}
#[tokio::test]
async fn optional_controls_use_prerequisite_reads_one_exact_send_and_no_optimistic_cache() {
    let (f, device) = fixture().await;
    *f.wire.state.lock().unwrap() = Some(
        json!({"power":"on","eco":"off","buzzer":"off","wind_swing_lr":"off","wind_swing_ud":"on","screen_display_now":100,"screen_display":"on"}),
    );
    f.wire
        .query_states
        .lock()
        .unwrap()
        .insert("no_wind_sense".into(), json!({"no_wind_sense":0}));
    let c = granted_core(&f, &device, true);
    c.discover(&operation()).await.unwrap();
    let descriptor = c.describe(&device.id).unwrap();
    let expected: homeverb_domain::Capability = serde_json::from_str(include_str!(
        "../../../../examples/capability.meiju-ac-features.json"
    ))
    .unwrap();
    assert_eq!(
        json!(descriptor.capability("main", "vendor.midea.ac").unwrap()),
        json!(expected)
    );
    let before = f.read(&device, "main", false).await.unwrap();
    for (name, value, expected, reads) in [
        (
            "eco",
            json!(true),
            json!({"eco":"on","cool_power_saving":1,"buzzer":"off"}),
            1,
        ),
        (
            "breezeless",
            json!(2),
            json!({"no_wind_sense":2,"buzzer":"off"}),
            2,
        ),
        (
            "horizontal_swing",
            json!(true),
            json!({"wind_swing_lr":"on","wind_swing_ud":"on","buzzer":"off"}),
            1,
        ),
        (
            "display",
            json!(false),
            json!({"screen_display":"off","buzzer":"off"}),
            1,
        ),
    ] {
        let cmd = command(&device, name, value);
        let calls = f.calls();
        let sends = f.wire.controls.lock().unwrap().len();
        let receipt = c.execute(cmd.clone(), CancellationToken::new()).await;
        assert_eq!(
            receipt.status,
            Status::Accepted,
            "{name}: {:?}",
            receipt.error
        );
        assert_eq!(receipt.error.as_ref().unwrap().code, "VERIFICATION_PENDING");
        assert!(receipt.verification.is_none());
        assert_eq!(f.calls(), calls + reads);
        assert_eq!(
            f.wire.controls.lock().unwrap().last().unwrap()["command"],
            json!({"control":expected})
        );
        assert_eq!(
            json!(f.store.snapshot(&device.id, "main").unwrap().unwrap()),
            json!(before)
        );
        assert_eq!(
            json!(c.execute(cmd, CancellationToken::new()).await),
            json!(receipt)
        );
        assert_eq!(f.calls(), calls + reads);
        assert_eq!(f.wire.controls.lock().unwrap().len(), sends + 1);
    }
}
#[tokio::test]
async fn missing_optional_state_does_not_invent_values_or_fail_basic_read() {
    let (f, device) = fixture().await;
    *f.wire.state.lock().unwrap() = Some(json!({"power":"on","eco":"unsupported","buzzer":"off"}));
    let c = granted_core(&f, &device, true);
    c.discover(&operation()).await.unwrap();
    let snapshot = c
        .read(&device.id, "main", Route::Cloud, false, &operation())
        .await
        .unwrap();
    assert_eq!(snapshot.observations["power.state"].value, json!(true));
    assert!(!snapshot.observations.contains_key("vendor.midea.ac.eco"));
    let receipt = c
        .execute(
            command(&device, "eco", json!(true)),
            CancellationToken::new(),
        )
        .await;
    assert_eq!(receipt.status, Status::Failed);
    assert_eq!(receipt.attempts[0].phase, Phase::NotSent);
    assert!(f.wire.controls.lock().unwrap().is_empty());
    assert!(
        !f.wire
            .queries
            .lock()
            .unwrap()
            .iter()
            .any(|q| q["query_type"] == "group_data_four")
    );
}
#[tokio::test]
async fn complete_default_state_avoids_every_supplemental_query() {
    let (f, device) = fixture().await;
    *f.wire.state.lock().unwrap() = Some(
        json!({"eco":"off","comfort_power_save":"off","strong_wind":"off","wind_swing_lr":"off","wind_swing_ud":"off","fresh_air_remove_odor":0,"buzzer":"off","screen_display_now":100,"prevent_straight_wind":1,"prevent_super_cool":"off","dry":"off","ptc":"off","follow_body_sense":"off","wind_swing_ud_angle":50,"wind_swing_lr_angle":50,"no_wind_sense":0}),
    );
    let c = granted_core(&f, &device, true);
    c.discover(&operation()).await.unwrap();
    let calls = f.calls();
    let snapshot = c
        .read(&device.id, "main", Route::Cloud, false, &operation())
        .await
        .unwrap();
    assert_eq!(f.calls(), calls + 1);
    assert_eq!(snapshot.observations.len(), 16);
    assert!(
        snapshot
            .observations
            .values()
            .all(|o| o.stale && o.observed_at.is_none())
    );
}
#[tokio::test]
async fn private_feature_contract_and_protected_membership_survive_descriptor_tampering() {
    let (f, device) = fixture().await;
    let c = granted_core(&f, &device, true);
    c.discover(&operation()).await.unwrap();
    let mut descriptor = c.describe(&device.id).unwrap();
    let vendor = descriptor
        .capabilities
        .iter_mut()
        .find(|c| c.id == "vendor.midea.ac")
        .unwrap();
    let weakened = vendor.actions.get_mut("set_eco").unwrap();
    weakened.parameters = json!({"type":"object"});
    let weakened = weakened.clone();
    vendor.actions.insert("set_self_clean".into(), weakened);
    f.store.save_device(&descriptor).unwrap();
    for (name, args) in [
        ("eco", json!({})),
        ("eco", json!({"eco":1})),
        ("eco", json!({"eco":true,"power":"on"})),
        ("self_clean", json!({"self_clean":true})),
    ] {
        let mut cmd = command(&device, name, json!(true));
        cmd.arguments = args;
        let receipt = c.execute(cmd, CancellationToken::new()).await;
        assert_eq!(receipt.status, Status::Failed);
        assert_eq!(receipt.attempts[0].phase, Phase::NotSent);
    }
    assert_eq!(
        granted_core(&f, &device, false)
            .execute(
                command(&device, "eco", json!(true)),
                CancellationToken::new()
            )
            .await
            .error
            .unwrap()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
    // A wildcard policy is not an exact feature grant.
    assert_eq!(
        core(&f, &device, true)
            .execute(
                command(&device, "eco", json!(true)),
                CancellationToken::new()
            )
            .await
            .error
            .unwrap()
            .code,
        "POLICY_DENIED"
    );
    let baseline = f.secrets.values.lock().unwrap().clone();
    for (field, value) in [
        ("product_profile", Value::Null),
        ("model", json!("Unknown")),
        ("listed", json!(false)),
    ] {
        let mut changed = baseline.clone();
        for bytes in changed.values_mut() {
            let mut payload: Value = serde_json::from_slice(bytes).unwrap();
            payload["bindings"][0][field] = value.clone();
            *bytes = serde_json::to_vec(&payload).unwrap();
        }
        *f.secrets.values.lock().unwrap() = changed;
        let result = c
            .execute(
                command(&device, "eco", json!(true)),
                CancellationToken::new(),
            )
            .await;
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.attempts[0].phase, Phase::NotSent);
    }
    assert!(f.wire.controls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn optional_feature_lost_ack_and_rejection_keep_receipts_without_resending() {
    for (fault, reply, expected) in [
        (1, None, Status::Failed),
        (2, None, Status::Unknown),
        (0, Some(b"{\"code\":7400}".to_vec()), Status::Failed),
    ] {
        let (f, device) = fixture().await;
        *f.wire.state.lock().unwrap() = Some(json!({"eco":"off","buzzer":"on"}));
        let c = granted_core(&f, &device, true);
        c.discover(&operation()).await.unwrap();
        f.wire.control_fault.store(fault, Ordering::Relaxed);
        *f.wire.control_reply.lock().unwrap() = reply;
        let cmd = command(&device, "eco", json!(true));
        let receipt = c.execute(cmd.clone(), CancellationToken::new()).await;
        assert_eq!(receipt.status, expected);
        let sends = f.wire.controls.lock().unwrap().len();
        let reads = f.calls();
        assert_eq!(
            json!(c.execute(cmd, CancellationToken::new()).await),
            json!(receipt)
        );
        assert_eq!(f.wire.controls.lock().unwrap().len(), sends);
        assert_eq!(f.calls(), reads);
    }
}
#[tokio::test]
async fn prerequisite_cancellation_proves_not_sent() {
    let (f, device) = fixture().await;
    let c = Arc::new(granted_core(&f, &device, true));
    c.discover(&operation()).await.unwrap();
    f.wire.delay.store(true, Ordering::Relaxed);
    let cancel = CancellationToken::new();
    let run_cancel = cancel.clone();
    let cmd = command(&device, "eco", json!(true));
    let task = tokio::spawn(async move { c.execute(cmd, run_cancel).await });
    f.wire.started.notified().await;
    cancel.cancel();
    let receipt = task.await.unwrap();
    assert_eq!(receipt.status, Status::Failed);
    assert_eq!(receipt.attempts[0].phase, Phase::NotSent);
    assert!(f.wire.controls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn optional_query_failure_keeps_basic_observations_and_fixed_warning() {
    let (f, device) = fixture().await;
    *f.wire.state.lock().unwrap() = Some(json!({"power":"on","eco":"off","buzzer":"off"}));
    *f.wire.query_fault.lock().unwrap() = Some("no_wind_sense".into());
    let c = granted_core(&f, &device, true);
    c.discover(&operation()).await.unwrap();
    let result = c
        .read(&device.id, "main", Route::Cloud, false, &operation())
        .await
        .unwrap();
    assert_eq!(result.observations["power.state"].value, json!(true));
    assert_eq!(
        result.observations["vendor.midea.ac.eco"].value,
        json!(false)
    );
    assert!(
        !result
            .observations
            .contains_key("vendor.midea.ac.breezeless")
    );
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w == "An optional AC state query is unavailable.")
    );
    assert!(f.wire.controls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn supplemental_reads_are_limited_to_the_hosts_exact_device_scope() {
    let (f, device) = fixture().await;
    let adapter = MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock))
        .with_ac_feature_control()
        .with_ac_feature_devices([]);
    let c = Core::new(
        f.store.clone(),
        vec![Arc::new(adapter)],
        Arc::new(SystemClock),
        Policy::default(),
    );
    c.discover(&operation()).await.unwrap();
    let before = f.calls();
    let result = c
        .read(&device.id, "main", Route::Cloud, false, &operation())
        .await
        .unwrap();
    assert_eq!(result.observations["power.state"].value, json!(true));
    assert_eq!(f.calls(), before + 1);
    assert!(f.wire.controls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn legacy_basic_descriptors_keep_foreground_reads_until_explicit_feature_setup() {
    let (f, device) = fixture().await;
    let mut descriptor = f.core.describe(&device.id).unwrap();
    let cap = descriptor
        .capabilities
        .iter_mut()
        .find(|c| c.id == "vendor.midea.ac")
        .unwrap();
    cap.revision = 1;
    cap.properties
        .retain(|name, _| matches!(name.as_str(), "mode" | "wind_speed"));
    f.store.save_device(&descriptor).unwrap();
    *f.wire.state.lock().unwrap() =
        Some(json!({"power":"on","eco":"off","no_wind_sense":0,"buzzer":"off"}));
    let snapshot = f.read(&device, "main", false).await.unwrap();
    assert_eq!(snapshot.observations["power.state"].value, json!(true));
    assert!(!snapshot.observations.contains_key("vendor.midea.ac.eco"));
}

#[tokio::test]
async fn missing_client_buzzer_preference_does_not_block_supported_feature_or_invent_sound_state() {
    let (f, device) = fixture().await;
    *f.wire.state.lock().unwrap() = Some(json!({"eco":"off"}));
    let c = granted_core(&f, &device, true);
    c.discover(&operation()).await.unwrap();
    let result = c
        .execute(
            command(&device, "eco", json!(true)),
            CancellationToken::new(),
        )
        .await;
    assert_eq!(result.status, Status::Accepted);
    assert_eq!(
        f.wire.controls.lock().unwrap().last().unwrap()["command"],
        json!({"control":{"eco":"on","cool_power_saving":1}})
    );
    *f.wire.state.lock().unwrap() = Some(json!({"eco":"off","buzzer":"invalid"}));
    let result = c
        .execute(
            command(&device, "eco", json!(true)),
            CancellationToken::new(),
        )
        .await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.attempts[0].phase, Phase::NotSent);
    assert_eq!(f.wire.controls.lock().unwrap().len(), 1);
}
