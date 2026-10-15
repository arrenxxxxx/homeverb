use super::*;
use homeverb_domain::{Phase, Status, Transport};

fn command(device: &Device, mode: AcMode) -> Command {
    let mut command = power_command(device, true);
    command.capability = "vendor.midea.ac".into();
    command.action = "activate_mode".into();
    command.arguments = json!({"mode":mode});
    command
}

fn core(f: &Fixture, device: &Device, enabled: bool) -> Core {
    let mut adapter = MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock));
    if enabled {
        adapter = adapter.with_ac_mode_control();
    }
    Core::new(
        f.store.clone(),
        vec![Arc::new(adapter)],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: device.id.clone(),
                component_id: "main".into(),
                capability: "vendor.midea.ac".into(),
                action: "activate_mode".into(),
                transports: vec![Transport::Cloud],
                argument_schema: json!({"type":"object"}),
            }],
        },
    )
}

#[tokio::test]
async fn activation_sends_power_and_mode_together_without_temperature_fan_or_cache_changes() {
    let f = Fixture::new(appliances()).await;
    let devices = f.discover().await;
    for device in devices
        .iter()
        .filter(|device| MODELS[..4].iter().any(|(model, _)| *model == device.model))
    {
        let before = f.read(device, "main", false).await.unwrap();
        let core = core(&f, device, true);
        core.discover(&operation()).await.unwrap();
        let descriptor = core.describe(&device.id).unwrap();
        let capability = descriptor.capability("main", "vendor.midea.ac").unwrap();
        let expected: homeverb_domain::Capability = serde_json::from_str(include_str!(
            "../../../../examples/capability.meiju-mode.json"
        ))
        .unwrap();
        assert_eq!(json!(capability), json!(expected));
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
        for mode in AC_MODES {
            let command = command(device, mode);
            let reads = f.calls();
            let sends = f.wire.controls.lock().unwrap().len();
            let receipt = core
                .execute(command.clone(), CancellationToken::new())
                .await;
            assert_eq!(receipt.status, Status::Accepted);
            assert_eq!(receipt.error.as_ref().unwrap().code, "VERIFICATION_PENDING");
            assert!(receipt.verification.is_none());
            assert!(receipt.items.is_none());
            assert_eq!(f.calls(), reads);
            assert_eq!(f.wire.controls.lock().unwrap().len(), sends + 1);
            assert_eq!(
                f.wire.controls.lock().unwrap().last().unwrap()["command"],
                json!({"control":{"power":"on","mode":mode}})
            );
            assert_eq!(
                json!(f.store.snapshot(&device.id, "main").unwrap().unwrap()),
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
        .filter(|device| MODELS[4..].iter().any(|(model, _)| *model == device.model))
    {
        assert!(
            core(&f, device, true)
                .plan(&command(device, AcMode::Cool))
                .is_err()
        );
    }
}

#[tokio::test]
async fn mode_refuses_weakened_metadata_missing_opt_in_and_invalid_protected_targets() {
    let f = Fixture::new(vec![appliances()[0].clone()]).await;
    let device = f.discover().await.remove(0);
    let enabled = core(&f, &device, true);
    enabled.discover(&operation()).await.unwrap();
    let mut descriptor = enabled.describe(&device.id).unwrap();
    descriptor
        .capabilities
        .iter_mut()
        .find(|c| c.id == "vendor.midea.ac")
        .unwrap()
        .actions
        .get_mut("activate_mode")
        .unwrap()
        .parameters = json!({"type":"object"});
    f.store.save_device(&descriptor).unwrap();
    for args in [
        json!({}),
        json!({"mode":"off"}),
        json!({"mode":"fan_only"}),
        json!({"mode":"COOL"}),
        json!({"mode":true}),
        json!({"mode":"cool","power":"off"}),
        json!({"mode":"cool","temperature":26}),
    ] {
        let mut value = command(&device, AcMode::Cool);
        value.arguments = args;
        let result = enabled.execute(value, CancellationToken::new()).await;
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.attempts[0].phase, Phase::NotSent);
    }
    let refused = core(&f, &device, false)
        .execute(command(&device, AcMode::Cool), CancellationToken::new())
        .await;
    assert_eq!(refused.error.unwrap().code, "CAPABILITY_UNSUPPORTED");
    let baseline = f.secrets.values.lock().unwrap().clone();
    for (field, value) in [
        ("model", json!("Unknown AC")),
        ("family", json!(0xca)),
        ("listed", json!(false)),
        ("listed", Value::Null),
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
        let result = enabled
            .execute(command(&device, AcMode::Cool), CancellationToken::new())
            .await;
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.attempts[0].phase, Phase::NotSent);
    }
    assert!(f.wire.controls.lock().unwrap().is_empty());
}
