use super::*;
use homeverb_domain::{Phase, Status, Transport};

fn command(device: &Device, value: f64) -> Command {
    let mut command = power_command(device, false);
    command.capability = "climate".into();
    command.action = "set".into();
    command.arguments = json!({"target_temperature":value});
    command
}
fn core(f: &Fixture, device: &Device, enabled: bool) -> Core {
    let mut adapter = MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock));
    if enabled {
        adapter = adapter.with_ac_temperature_control();
    }
    Core::new(
        f.store.clone(),
        vec![Arc::new(adapter)],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: device.id.clone(),
                component_id: "main".into(),
                capability: "climate".into(),
                action: "set".into(),
                transports: vec![Transport::Cloud],
                argument_schema: json!({"type":"object"}),
            }],
        },
    )
}

#[tokio::test]
async fn temperature_controls_use_each_protected_encoding_without_other_fields_or_cache_changes() {
    for (profile, minimum, step, fractional) in [
        (json!({"modelNumber":524}), 16.0, 1.0, true),
        (json!({"modelNumber":1,"sn8":"23096245"}), 17.0, 1.0, false),
        (json!({"modelNumber":1,"sn8":"22397091"}), 16.0, 0.5, true),
        (
            json!({"modelNumber":1,"sn8":"TEST1234","category":"wall-air-conditioner"}),
            17.0,
            0.5,
            true,
        ),
    ] {
        let entries = appliances()
            .into_iter()
            .take(4)
            .map(|mut entry| {
                entry
                    .as_object_mut()
                    .unwrap()
                    .extend(profile.as_object().unwrap().clone());
                entry
            })
            .collect();
        let f = Fixture::new(entries).await;
        let devices = f.discover().await;
        assert_eq!(devices.len(), 4);
        for device in devices {
            *f.wire.state.lock().unwrap() = Some(if fractional {
                json!({"temperature":15,"small_temperature":0.5})
            } else {
                json!({"temperature":15.5})
            });
            let before = f.read(&device, "main", false).await.unwrap();
            let core = core(&f, &device, true);
            core.discover(&operation()).await.unwrap();
            let descriptor = core.describe(&device.id).unwrap();
            let capability = descriptor.capability("main", "climate").unwrap();
            assert_eq!(
                capability.actions["set"].parameters["properties"]["target_temperature"],
                json!({"type":"number","minimum":minimum as u8,"maximum":30,"multipleOf":step})
            );
            if minimum == 17.0 && step == 0.5 {
                let fixture: homeverb_domain::Capability = serde_json::from_str(include_str!(
                    "../../../../examples/capability.meiju-temperature.json"
                ))
                .unwrap();
                assert_eq!(json!(capability), json!(fixture));
            }
            // An action's write bounds must not suppress a historical reading outside them.
            f.read(&device, "main", true).await.unwrap();
            for value in [minimum, minimum + step, 30.0] {
                let command = command(&device, value);
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
                let mut expected = json!({"temperature":value as u8});
                if fractional {
                    expected["small_temperature"] = json!(value.fract());
                }
                assert_eq!(
                    f.wire.controls.lock().unwrap().last().unwrap()["command"],
                    json!({"control":expected})
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
        // Omitted subsequent fields retain the protected mapping for descriptors.
        *f.wire.inventories.lock().unwrap().get_mut("FIRST").unwrap() =
            appliances().into_iter().take(4).collect();
        let device = f.store.devices().unwrap().remove(0).device;
        let enabled = core(&f, &device, true);
        enabled.discover(&operation()).await.unwrap();
        assert!(
            enabled
                .describe(&device.id)
                .unwrap()
                .capability("main", "climate")
                .unwrap()
                .actions
                .contains_key("set")
        );
    }
}

#[tokio::test]
async fn weakened_descriptors_cannot_bypass_profile_arguments_opt_in_or_membership() {
    let mut entry = appliances()[0].clone();
    entry["modelNumber"] = json!(524);
    let f = Fixture::new(vec![entry]).await;
    let device = f.discover().await.remove(0);
    let enabled = core(&f, &device, true);
    enabled.discover(&operation()).await.unwrap();
    let mut descriptor = enabled.describe(&device.id).unwrap();
    descriptor
        .capabilities
        .iter_mut()
        .find(|cap| cap.id == "climate")
        .unwrap()
        .actions
        .get_mut("set")
        .unwrap()
        .parameters = json!({"type":"object"});
    f.store.save_device(&descriptor).unwrap();
    for arguments in [
        json!({}),
        json!({"target_temperature":16.5}),
        json!({"target_temperature":15}),
        json!({"target_temperature":31}),
        json!({"target_temperature":"26"}),
        json!({"target_temperature":true}),
        json!({"target_temperature":26,"mode":"cool"}),
    ] {
        let mut value = command(&device, 26.0);
        value.arguments = arguments;
        let result = enabled.execute(value, CancellationToken::new()).await;
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.attempts[0].phase, Phase::NotSent);
    }
    let refused = core(&f, &device, false)
        .execute(command(&device, 26.0), CancellationToken::new())
        .await;
    assert_eq!(refused.error.unwrap().code, "CAPABILITY_UNSUPPORTED");
    let baseline = f.secrets.values.lock().unwrap().clone();
    for (field, value) in [
        ("product_profile", Value::Null),
        (
            "product_profile",
            json!({"sn8":"22012895","subtype":"1","category":"wall-air-conditioner"}),
        ),
        (
            "product_profile",
            json!({"sn8":"26096947","subtype":"1","category":"wall_air_conditioner"}),
        ),
        ("model", json!("Unknown model")),
        ("family", json!(0xca)),
        ("listed", json!(false)),
        ("listed", Value::Null),
    ] {
        let mut changed = baseline.clone();
        for bytes in changed.values_mut() {
            let mut payload: Value = serde_json::from_slice(bytes).unwrap();
            payload["bindings"][0][field] = value.clone();
            *bytes = serde_json::to_vec(&payload).unwrap();
        }
        *f.secrets.values.lock().unwrap() = changed;
        let result = enabled
            .execute(command(&device, 26.0), CancellationToken::new())
            .await;
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.attempts[0].phase, Phase::NotSent);
    }
    assert!(f.wire.controls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn read_only_scalar_targets_use_retained_profiles_and_preserve_cache_on_invalid_responses() {
    let entries = appliances()
        .into_iter()
        .take(4)
        .map(|mut entry| {
            entry["modelNumber"] = json!(1);
            entry["sn8"] = json!("23096245");
            entry
        })
        .collect();
    let f = Fixture::new(entries).await;
    let devices = f.discover().await;
    // Later sparse inventory must not erase the protected read mapping.
    *f.wire.inventories.lock().unwrap().get_mut("FIRST").unwrap() =
        appliances().into_iter().take(4).collect();
    f.discover().await;
    for device in devices {
        let descriptor = f.core.describe(&device.id).unwrap();
        let climate = descriptor.capability("main", "climate").unwrap();
        assert!(climate.actions.is_empty());
        assert_eq!(
            climate.properties["target_temperature"]["writable"],
            json!(false)
        );
        *f.wire.state.lock().unwrap() = Some(json!({"temperature":"15.5","indoor_temperature":27}));
        let calls = f.calls();
        let live = f.read(&device, "main", false).await.unwrap();
        assert_eq!(f.calls(), calls + 1);
        assert_eq!(
            live.observations["climate.target_temperature"].value,
            json!(15.5)
        );
        assert!(live.observations["climate.target_temperature"].stale);
        assert!(
            live.observations["climate.target_temperature"]
                .observed_at
                .is_none()
        );
        let cached = f.read(&device, "main", true).await.unwrap();
        assert_eq!(f.calls(), calls + 1);
        assert_eq!(
            cached.observations["climate.target_temperature"].value,
            json!(15.5)
        );
        let saved = f.store.snapshot(&device.id, "main").unwrap().unwrap();
        *f.wire.state.lock().unwrap() = Some(json!({"temperature":true}));
        assert_eq!(
            f.read(&device, "main", false).await.unwrap_err().code,
            "MIDEA_RESPONSE_INVALID"
        );
        assert_eq!(f.calls(), calls + 2);
        assert_eq!(
            json!(f.store.snapshot(&device.id, "main").unwrap().unwrap()),
            json!(saved)
        );
    }
    assert!(f.wire.controls.lock().unwrap().is_empty());
}
