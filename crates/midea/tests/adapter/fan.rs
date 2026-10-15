use super::*;
use homeverb_domain::{Phase, Status, Transport};

fn fan_command(device: &Device, speed: u8) -> Command {
    let mut command = power_command(device, false);
    command.capability = "vendor.midea.ac".into();
    command.action = "set_fan_speed".into();
    command.arguments = json!({"wind_speed":speed});
    command
}

fn fan_core(f: &Fixture, device: &Device, arguments: Value) -> Core {
    Core::new(
        f.store.clone(),
        vec![Arc::new(
            MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock))
                .with_ac_fan_control(),
        )],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: device.id.clone(),
                component_id: "main".into(),
                capability: "vendor.midea.ac".into(),
                action: "set_fan_speed".into(),
                transports: vec![Transport::Cloud],
                argument_schema: arguments,
            }],
        },
    )
}

#[tokio::test]
async fn fan_refuses_absent_or_unconfirmed_membership_before_transport() {
    let f = Fixture::new(vec![appliances()[0].clone()]).await;
    let device = f.discover().await.remove(0);
    let core = fan_core(&f, &device, json!({"const":{"wind_speed":20}}));
    core.discover(&operation()).await.unwrap();
    f.wire
        .inventories
        .lock()
        .unwrap()
        .insert("FIRST".into(), vec![]);
    core.discover(&operation()).await.unwrap();
    for (listed, code) in [
        (json!(false), "MIDEA_DEVICE_NOT_LISTED"),
        (Value::Null, "MIDEA_INVENTORY_REQUIRED"),
    ] {
        for bytes in f.secrets.values.lock().unwrap().values_mut() {
            let mut payload: Value = serde_json::from_slice(bytes).unwrap();
            payload["bindings"][0]["listed"] = listed.clone();
            *bytes = serde_json::to_vec(&payload).unwrap();
        }
        let calls = f.calls();
        let refused = core
            .execute(fan_command(&device, 20), CancellationToken::new())
            .await;
        assert_eq!(refused.status, Status::Failed);
        assert_eq!(refused.error.unwrap().code, code);
        assert_eq!(refused.attempts[0].phase, Phase::NotSent);
        assert_eq!(f.calls(), calls);
        assert!(f.wire.controls.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn reviewed_models_and_codes_send_only_fan_fields_with_unverified_receipts() {
    let f = Fixture::new(appliances()).await;
    let devices = f.discover().await;
    for device in devices
        .iter()
        .filter(|d| MODELS[..4].iter().any(|(model, _)| *model == d.model))
    {
        let before = f.read(device, "main", false).await.unwrap();
        for speed in AC_FAN_SPEEDS {
            let core = fan_core(&f, device, json!({"const":{"wind_speed":speed}}));
            core.discover(&operation()).await.unwrap();
            let descriptor = core.describe(&device.id).unwrap();
            assert!(
                descriptor
                    .capability("main", "power")
                    .unwrap()
                    .actions
                    .is_empty()
            );
            assert!(
                descriptor
                    .capability("main", "climate")
                    .unwrap()
                    .actions
                    .is_empty()
            );
            let command = fan_command(device, speed);
            let reads = f.calls();
            let sends = f.wire.controls.lock().unwrap().len();
            let receipt = core
                .execute(command.clone(), CancellationToken::new())
                .await;
            assert_eq!(receipt.status, Status::Accepted);
            assert_eq!(receipt.error.as_ref().unwrap().code, "VERIFICATION_PENDING");
            assert!(receipt.verification.is_none());
            assert_eq!(f.calls(), reads);
            assert_eq!(f.wire.controls.lock().unwrap().len(), sends + 1);
            assert_eq!(
                f.wire.controls.lock().unwrap().last().unwrap()["command"],
                json!({"control":{"wind_speed":speed}})
            );
            assert_eq!(
                serde_json::to_value(f.store.snapshot(&device.id, "main").unwrap().unwrap())
                    .unwrap(),
                json!(before)
            );
            assert_eq!(
                json!(core.execute(command, CancellationToken::new()).await),
                json!(receipt)
            );
            assert_eq!(f.wire.controls.lock().unwrap().len(), sends + 1);
        }
    }
    for device in devices
        .iter()
        .filter(|d| MODELS[4..].iter().any(|(model, _)| *model == d.model))
    {
        let core = fan_core(&f, device, json!({"type":"object"}));
        assert_eq!(
            core.plan(&fan_command(device, 20)).unwrap_err().code,
            "CAPABILITY_UNSUPPORTED"
        );
    }
    // Cached writable metadata grants no authority to the default read-only host.
    assert_eq!(
        f.core
            .plan(&fan_command(
                devices.iter().find(|d| d.model == MODELS[0].0).unwrap(),
                20
            ))
            .unwrap_err()
            .code,
        "POLICY_DENIED"
    );
}

#[tokio::test]
async fn fan_contract_is_rechecked_even_after_public_metadata_is_weakened() {
    let f = Fixture::new(appliances()).await;
    let device = f
        .discover()
        .await
        .into_iter()
        .find(|d| d.model == MODELS[0].0)
        .unwrap();
    let core = fan_core(&f, &device, json!({"type":"object"}));
    core.discover(&operation()).await.unwrap();
    let mut descriptor = core.describe(&device.id).unwrap();
    descriptor
        .capabilities
        .iter_mut()
        .find(|c| c.id == "vendor.midea.ac")
        .unwrap()
        .actions
        .get_mut("set_fan_speed")
        .unwrap()
        .parameters = json!({"type":"object"});
    f.store.save_device(&descriptor).unwrap();
    for args in [
        json!({}),
        json!({"wind_speed":0}),
        json!({"wind_speed":101}),
        json!({"wind_speed":20.5}),
        json!({"wind_speed":"20"}),
        json!({"wind_speed":20,"power":"on"}),
    ] {
        let mut command = fan_command(&device, 20);
        command.arguments = args;
        let receipt = core.execute(command, CancellationToken::new()).await;
        assert_eq!(receipt.status, Status::Failed);
        assert_eq!(receipt.attempts[0].phase, Phase::NotSent);
    }
    assert!(f.wire.controls.lock().unwrap().is_empty());
    let without_opt_in = Core::new(
        f.store.clone(),
        vec![f.adapter.clone()],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: device.id.clone(),
                component_id: "main".into(),
                capability: "vendor.midea.ac".into(),
                action: "set_fan_speed".into(),
                transports: vec![Transport::Cloud],
                argument_schema: json!({"type":"object"}),
            }],
        },
    );
    let refused = without_opt_in
        .execute(fan_command(&device, 20), CancellationToken::new())
        .await;
    assert_eq!(refused.error.unwrap().code, "CAPABILITY_UNSUPPORTED");
    assert_eq!(refused.attempts[0].phase, Phase::NotSent);
    assert!(f.wire.controls.lock().unwrap().is_empty());
    let power_only = f.control_core(&device, false);
    assert_eq!(
        power_only.plan(&fan_command(&device, 20)).unwrap_err().code,
        "POLICY_DENIED"
    );
    let mut numeric = fan_command(&device, 20);
    numeric.arguments = json!({"wind_speed":20.0});
    assert_eq!(
        core.execute(numeric, CancellationToken::new()).await.status,
        Status::Accepted
    );
    assert_eq!(
        f.wire.controls.lock().unwrap()[0]["command"],
        json!({"control":{"wind_speed":20}})
    );
}

#[tokio::test]
async fn fan_refuses_changed_private_model_or_binding_without_sending() {
    let f = Fixture::new(appliances()).await;
    let device = f
        .discover()
        .await
        .into_iter()
        .find(|d| d.model == MODELS[0].0)
        .unwrap();
    let core = fan_core(&f, &device, json!({"type":"object"}));
    core.discover(&operation()).await.unwrap();
    let baseline = f.secrets.values.lock().unwrap().clone();
    for (field, value) in [
        ("model", json!("Unreviewed AC")),
        ("family", json!(0xca)),
        ("connection_id", json!(Uuid::new_v4())),
        ("identity_ref", json!(Uuid::new_v4())),
    ] {
        let mut changed = baseline.clone();
        for bytes in changed.values_mut() {
            let mut payload: Value = serde_json::from_slice(bytes).unwrap();
            let binding = &mut payload["bindings"][0];
            if field.ends_with("_id") || field.ends_with("_ref") {
                binding["local"][field] = value.clone();
            } else {
                binding[field] = value.clone();
            }
            *bytes = serde_json::to_vec(&payload).unwrap();
        }
        *f.secrets.values.lock().unwrap() = changed;
        let receipt = core
            .execute(fan_command(&device, 20), CancellationToken::new())
            .await;
        assert_eq!(receipt.status, Status::Failed);
        assert_eq!(receipt.attempts[0].phase, Phase::NotSent);
        assert!(f.wire.controls.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn fan_provider_errors_keep_uncertainty_and_never_replay() {
    let f = Fixture::new(appliances()).await;
    let device = f
        .discover()
        .await
        .into_iter()
        .find(|d| d.model == MODELS[0].0)
        .unwrap();
    let core = fan_core(&f, &device, json!({"type":"object"}));
    core.discover(&operation()).await.unwrap();
    for (body, status) in [
        (r#"{"code":0}"#, Status::Accepted),
        (r#"{"code":1306}"#, Status::Unknown),
        (r#"{"code":7400}"#, Status::Failed),
        (r#"{"code":0,"code":7400}"#, Status::Unknown),
    ] {
        *f.wire.control_reply.lock().unwrap() = Some(body.as_bytes().to_vec());
        let command = fan_command(&device, 40);
        let before = f.wire.controls.lock().unwrap().len();
        let result = core
            .execute(command.clone(), CancellationToken::new())
            .await;
        assert_eq!(result.status, status);
        assert_eq!(
            json!(core.execute(command, CancellationToken::new()).await),
            json!(result)
        );
        assert_eq!(f.wire.controls.lock().unwrap().len(), before + 1);
    }
}
