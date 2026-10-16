use super::*;
use homeverb_domain::Descriptor;

async fn setup(fields: Value) -> (Fixture, Descriptor, AccountRecord) {
    let f = Fixture::new();
    *f.wire.product_fields.lock().unwrap() = Some(fields);
    assert_eq!(
        f.connect(None).await["data"]["retrieval"]["status"],
        "complete"
    );
    let descriptor = f
        .store
        .devices()
        .unwrap()
        .into_iter()
        .find(|d| d.device.alias == "Synthetic AC")
        .unwrap();
    let account = f.store.accounts().unwrap().remove(0);
    (f, descriptor, account)
}

fn request(descriptor: &Descriptor) -> Value {
    json!({"method":"inspect_control_options", "device_id":descriptor.device.id,
        "connection_id":descriptor.device.connections[0].id})
}

fn edit_binding(f: &Fixture, account: &AccountRecord, device: &str, edit: impl FnOnce(&mut Value)) {
    let key = account.credential_ref.to_string();
    let mut values = f.secrets.values.lock().unwrap();
    let mut payload: Value = serde_json::from_slice(&values[&key]).unwrap();
    let binding = payload["bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|binding| binding["local"]["device_id"] == device)
        .unwrap();
    edit(binding);
    values.insert(key, serde_json::to_vec(&payload).unwrap());
}

#[tokio::test(start_paused = true)]
async fn read_only_options_enable_permission_selection_without_mutation_or_authority() {
    let (mut f, descriptor, account) = setup(json!({"modelNumber":1,"sn8":"22397091"})).await;
    let cards = json!(f.store.devices().unwrap());
    let secrets = f.secrets.values.lock().unwrap().clone();
    let calls = f.calls();
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    let command = power_command(&descriptor.device.id, false);
    let denied = f.call(json!({"method":"plan","command":command})).await;
    assert!(denied["error"].is_object());
    let result = f.call(request(&descriptor)).await;
    assert_eq!(result["data"]["source"], "saved_inventory");
    assert_eq!(result["data"]["authorization"], "not_checked");
    assert_eq!(result["data"]["device_id"], descriptor.device.id);
    assert_eq!(
        result["data"]["connection_id"],
        descriptor.device.connections[0].id
    );
    assert_eq!(result["data"]["options"].as_array().unwrap().len(), 4);
    let examples: Value = serde_json::from_str(include_str!(
        "../../../../packages/native-contract/account-examples.json"
    ))
    .unwrap();
    let example = examples["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["request"]["operation_id"] == "inspect-options-ac")
        .unwrap();
    assert_eq!(
        result["data"]["options"],
        example["response"]["data"]["options"]
    );
    assert_eq!(
        f.call(json!({"method":"plan","command":command})).await,
        denied
    );
    assert_eq!(json!(f.store.devices().unwrap()), cards);
    assert_eq!(*f.secrets.values.lock().unwrap(), secrets);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 0);
    assert_eq!(f.calls(), calls);
    assert_eq!(
        f.call(json!({"method":"recent_executions"})).await["data"],
        json!([])
    );
    let execution = f.call(json!({"method":"execute","command":command})).await;
    assert_eq!(execution["status"], "failed");
    assert_eq!(execution["attempts"], json!([]));
    assert_eq!(f.calls(), calls);
    for marker in [
        "22397091",
        "880000000000001",
        "SYNTHETIC",
        "product_profile",
        "credential_ref",
    ] {
        assert!(!result.to_string().contains(marker));
    }
    // User-side grant selection remains a separate trusted startup operation.
    let command = power_command(&descriptor.device.id, false);
    f.reopen_with_power_grants(&[MeijuPowerGrant {
        device_id: descriptor.device.id.clone(),
        state: false,
    }]);
    assert_eq!(f.call(request(&descriptor)).await, result);
    f.refresh(&account.id.to_string()).await;
    assert!(f.call(json!({"method":"plan","command":command})).await["error"].is_null());
    assert_eq!(
        f.call(json!({"method":"plan","command":power_command(&descriptor.device.id,true)}))
            .await["error"]["code"],
        "POLICY_DENIED"
    );
    assert!(!f.calls().contains(&CloudEndpoint::ApplianceControl));
}

#[tokio::test(start_paused = true)]
async fn protected_profiles_determine_options_and_match_enabled_descriptors() {
    let cases = [
        (
            json!({"modelNumber":524,"sn8":"21293193"}),
            4,
            Some((16, 1.0)),
        ),
        (
            json!({"modelNumber":1,"sn8":"23096245"}),
            4,
            Some((17, 1.0)),
        ),
        (
            json!({"modelNumber":1,"sn8":"22397091"}),
            4,
            Some((16, 0.5)),
        ),
        (
            json!({"modelNumber":1,"sn8":"22040055"}),
            4,
            Some((17, 0.5)),
        ),
        (
            json!({"modelNumber":1,"sn8":"22012895","category":"wall-air-conditioner"}),
            3,
            None,
        ),
        (json!({"sn8":"22397091"}), 3, None),
        (
            json!({"type":"CA","productModel":"BCD-600WUFIPZM(E)","modelNumber":524}),
            0,
            None,
        ),
        (
            json!({"type":"AC","productModel":"Unknown model","modelNumber":524}),
            0,
            None,
        ),
        (
            json!({"type":"E1","productModel":"KFR-35G/WF2","modelNumber":524}),
            0,
            None,
        ),
    ];
    for (fields, count, temperature) in cases {
        let (f, descriptor, _) = setup(fields).await;
        let result = f.call(request(&descriptor)).await;
        let options = result["data"]["options"].as_array().unwrap();
        assert_eq!(options.len(), count);
        if let Some((minimum, step)) = temperature {
            let option = options
                .iter()
                .find(|o| o["capability"] == "climate")
                .unwrap();
            let parameter = &option["definition"]["parameters"]["properties"]["target_temperature"];
            assert_eq!(parameter["minimum"], minimum);
            assert_eq!(parameter["multipleOf"], step);
        }
        let adapter = MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock))
            .with_ac_power_control()
            .with_ac_fan_control()
            .with_ac_mode_control()
            .with_ac_temperature_control()
            .with_dishwasher_control()
            .with_dishwasher_cycle_control();
        let enabled = adapter
            .discover(&operation())
            .await
            .unwrap()
            .into_iter()
            .find(|d| d.device.id == descriptor.device.id)
            .unwrap();
        assert_eq!(
            enabled
                .capabilities
                .iter()
                .map(|c| c.actions.len())
                .sum::<usize>(),
            options.len()
        );
        for option in options {
            let capability = enabled
                .capabilities
                .iter()
                .find(|c| c.id == option["capability"].as_str().unwrap())
                .unwrap();
            assert_eq!(
                json!(capability.actions[option["action"].as_str().unwrap()]),
                option["definition"]
            );
        }
    }
}

#[tokio::test(start_paused = true)]
async fn dishwasher_options_preserve_profile_domains_and_never_supply_approval() {
    for (code, count) in [
        ("7600020L", 4),
        ("76006481", 4),
        ("7600649Q", 4),
        ("7600V1E7", 4),
        ("760Y0026", 4),
        ("7600016L", 0),
        ("TEST1234", 0),
    ] {
        let (f, descriptor, _) = setup(json!({"type":"E1","productModel":"GX1200Max \u{5c0a}\u{4eab}\u{7248}","modelNumber":1,"sn8":code})).await;
        let calls = f.calls();
        let result = f.call(request(&descriptor)).await;
        let options = result["data"]["options"].as_array().unwrap();
        assert_eq!(options.len(), count);
        for option in options {
            assert_eq!(option["definition"]["risk"], "elevated");
            assert_eq!(option["definition"]["idempotency"], "non_idempotent");
            let mut arguments = if option["action"] == "start_cycle" {
                json!({"program":"eco_wash"})
            } else {
                json!({})
            };
            if code == "760Y0026" {
                arguments["additional"] = json!(0);
                assert_eq!(
                    option["definition"]["parameters"]["properties"]["additional"]["enum"],
                    json!([0, 9, 10, 13, 14])
                );
            }
            let mut command = power_command(&descriptor.device.id, false);
            command["capability"] = option["capability"].clone();
            command["action"] = option["action"].clone();
            command["arguments"] = arguments;
            assert!(f.call(json!({"method":"plan","command":command})).await["error"].is_object());
        }
        assert_eq!(f.calls(), calls);
    }
}

#[tokio::test(start_paused = true)]
async fn options_refuse_protected_identity_membership_and_secret_failures() {
    for (change, code) in [
        ("model", "DEVICE_IDENTITY_CONFLICT"),
        ("identity", "DEVICE_IDENTITY_CONFLICT"),
        ("connection", "DEVICE_IDENTITY_CONFLICT"),
        ("absent", "MIDEA_DEVICE_NOT_LISTED"),
        ("unconfirmed", "MIDEA_INVENTORY_REQUIRED"),
        ("pending", "ACCOUNT_SESSION_UNCONFIRMED"),
        ("corrupt", "ACCOUNT_SESSION_UNCONFIRMED"),
        ("unavailable", "SECRET_STORE_UNAVAILABLE"),
        ("lease", "RESOURCE_BUSY"),
    ] {
        let (f, descriptor, account) = setup(json!({"modelNumber":524})).await;
        let calls = f.calls();
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        let mut lease = None;
        match change {
            "model" => edit_binding(&f, &account, &descriptor.device.id, |b| {
                b["model"] = json!("KFR-26G/N8VJC3")
            }),
            "identity" => edit_binding(&f, &account, &descriptor.device.id, |b| {
                b["local"]["identity_ref"] = json!(Uuid::new_v4())
            }),
            "connection" => edit_binding(&f, &account, &descriptor.device.id, |b| {
                b["local"]["connection_id"] = json!(Uuid::new_v4())
            }),
            "absent" => edit_binding(&f, &account, &descriptor.device.id, |b| {
                b["listed"] = json!(false)
            }),
            "unconfirmed" => edit_binding(&f, &account, &descriptor.device.id, |b| {
                b["listed"] = Value::Null
            }),
            "pending" => f
                .store
                .set_credential_state(account.id, CredentialState::Pending)
                .unwrap(),
            "corrupt" => {
                f.secrets.values.lock().unwrap().insert(
                    account.credential_ref.to_string(),
                    b"PRIVATE-CORRUPT".to_vec(),
                );
            }
            "unavailable" => f.secrets.unavailable.store(true, Ordering::Relaxed),
            _ => lease = Some(f.vault.lease(account.id, &operation()).await.unwrap()),
        }
        let result = f.call(request(&descriptor)).await;
        assert!(result["data"].is_null());
        assert_eq!(result["error"]["code"], code, "{change}");
        assert_eq!(f.calls(), calls);
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        drop(lease);
    }
}

#[tokio::test(start_paused = true)]
async fn cancelled_option_inspection_releases_lease_without_network_or_mutation() {
    for stop in ["cancel", "deadline", "background", "close", "drop"] {
        let (f, descriptor, account) = setup(json!({"modelNumber":524})).await;
        let calls = f.calls();
        let secrets = f.secrets.values.lock().unwrap().clone();
        f.secrets.read_delay.store(true, Ordering::Relaxed);
        let mut work = Box::pin(f.call_id("options", request(&descriptor)));
        tokio::select! {
            _=f.secrets.read_started.notified()=>(),
            value=&mut work=>panic!("Expected pending inspection: {value}"),
        }
        assert!(f.vault.lease(account.id, &operation()).await.is_err());
        if stop == "drop" {
            drop(work);
        } else {
            match stop {
                "cancel" => assert!(f.session.cancel("options")),
                "background" => f.session.set_foreground(false),
                "close" => f.session.close(),
                _ => tokio::time::advance(std::time::Duration::from_secs(2)).await,
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
        drop(f.vault.lease(account.id, &operation()).await.unwrap());
        assert_eq!(*f.secrets.values.lock().unwrap(), secrets);
        assert_eq!(f.calls(), calls);
    }
}

#[tokio::test(start_paused = true)]
async fn option_inspection_rechecks_metadata_after_awaiting_protected_storage() {
    let (f, descriptor, _) = setup(json!({"modelNumber":524})).await;
    f.secrets.read_delay.store(true, Ordering::Relaxed);
    let mut work = Box::pin(f.call_id("options", request(&descriptor)));
    tokio::select! {
        _=f.secrets.read_started.notified()=>(),
        value=&mut work=>panic!("Expected pending inspection: {value}"),
    }
    let changed = f
        .call(
            json!({"method":"set_device_preferences","device_id":descriptor.device.id,
        "preferences":{"alias":"Renamed during inspection","room_id":null,"favorite":false}}),
        )
        .await;
    assert!(changed["error"].is_null());
    f.secrets.read_delay.store(false, Ordering::Relaxed);
    f.secrets.read_resume.notify_one();
    assert_eq!(work.await["error"]["code"], "DEVICE_IDENTITY_CONFLICT");
    let current = f.call(request(&descriptor)).await;
    assert!(current["error"].is_null());
}

#[tokio::test(start_paused = true)]
async fn option_selection_requires_exact_ids_and_never_reads_unrelated_secrets() {
    let (f, descriptor, _) = setup(json!({"modelNumber":524})).await;
    let other = MeijuAccount::create(
        &f.vault,
        "CN",
        "Other",
        &secret("OTHER-SUBJECT"),
        &secret("OTHER-SESSION"),
        &operation(),
    )
    .await
    .unwrap();
    let other_id = other.record().id;
    drop(other);
    let lease = f.vault.lease(other_id, &operation()).await.unwrap();
    assert!(f.call(request(&descriptor)).await["error"].is_null());
    drop(lease);
    f.secrets.unavailable.store(true, Ordering::Relaxed);
    let reads = f.secrets.reads.load(Ordering::Relaxed);
    let mut alias = request(&descriptor);
    alias["device_id"] = json!(descriptor.device.alias);
    assert_eq!(f.call(alias).await["error"]["code"], "INVALID_ARGUMENT");
    let mut missing = request(&descriptor);
    missing["device_id"] = json!(Uuid::new_v4());
    assert_eq!(f.call(missing).await["error"]["code"], "DEVICE_NOT_FOUND");
    let mut foreign = request(&descriptor);
    foreign["connection_id"] = json!(Uuid::new_v4());
    assert_eq!(f.call(foreign).await["error"]["code"], "INVALID_ARGUMENT");
    assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
}
