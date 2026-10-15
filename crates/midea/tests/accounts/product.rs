use super::*;

fn with_profile(id: &str) -> Value {
    let mut value = appliance(id, "AC");
    value["sn8"] = json!("TEST1234");
    value["modelNumber"] = json!("000524");
    value["category"] = json!("PRIVATE-PRODUCT-CATEGORY");
    value
}

fn payload(f: &Fixture, reference: CredentialRef) -> Value {
    serde_json::from_slice(&f.bytes(reference)).unwrap()
}

#[tokio::test]
async fn product_profiles_survive_legacy_upgrade_missing_fields_reopen_and_reauthorization() {
    for version in [1, 2] {
        let f = Fixture::new();
        let mut account = f.account().await;
        let record = account.record().clone();
        let bare = inventory(record.credential_ref, vec![appliance("11", "AC")]).await;
        let bindings = account.bind_inventory(&bare, &operation()).await.unwrap();
        f.store
            .save_device(&descriptor(bindings[0], record.id))
            .unwrap();
        let preferences = PreferencesInput {
            alias: Some("Kitchen AC".into()),
            room_id: None,
            favorite: true,
        };
        f.store
            .set_preferences(&bindings[0].device_id.to_string(), &preferences)
            .unwrap();
        let old_preferences = json!(f.store.preferences().unwrap());
        let mut old = payload(&f, record.credential_ref);
        old["version"] = json!(version);
        if version == 1 {
            old["bindings"][0].as_object_mut().unwrap().remove("listed");
        }
        f.set_bytes(record.credential_ref, serde_json::to_vec(&old).unwrap());
        let original = f.bytes(record.credential_ref);
        let puts = f.puts();
        drop(account);
        let mut account = f.reopen(record.id).await.unwrap();
        assert_eq!(f.bytes(record.credential_ref), original);
        assert_eq!(f.puts(), puts);
        let listing = inventory(record.credential_ref, vec![with_profile("11")]).await;
        assert_eq!(
            account
                .bind_inventory(&listing, &operation())
                .await
                .unwrap(),
            bindings
        );
        let upgraded = payload(&f, record.credential_ref);
        assert_eq!(upgraded["version"], 3);
        assert_eq!(
            upgraded["bindings"][0]["product_profile"],
            json!({"sn8":"TEST1234","subtype":"524","category":"PRIVATE-PRODUCT-CATEGORY"})
        );
        assert_eq!(upgraded["bindings"][0]["listed"], true);
        let saved = f.bytes(record.credential_ref);
        assert_eq!(
            account.bind_inventory(&bare, &operation()).await.unwrap(),
            bindings
        );
        assert_eq!(f.bytes(record.credential_ref), saved);
        account
            .replace_session(
                &secret("PRIVATE-SUBJECT"),
                &secret("ROTATED-SESSION"),
                &operation(),
            )
            .await
            .unwrap();
        assert_eq!(
            payload(&f, record.credential_ref)["bindings"],
            upgraded["bindings"]
        );
        assert_eq!(json!(f.store.preferences().unwrap()), old_preferences);
        for entry in std::fs::read_dir(&f.directory.0).unwrap().flatten() {
            if entry.path().is_file() {
                let bytes = std::fs::read(entry.path()).unwrap();
                for marker in ["TEST1234", "PRIVATE-PRODUCT-CATEGORY"] {
                    assert!(
                        !bytes
                            .windows(marker.len())
                            .any(|window| window == marker.as_bytes())
                    );
                }
            }
        }
    }
}

#[tokio::test]
async fn product_conflicts_and_corrupt_profiles_preserve_the_entire_saved_binding_set() {
    let f = Fixture::new();
    let mut account = f.account().await;
    let record = account.record().clone();
    let listing = inventory(record.credential_ref, vec![with_profile("11")]).await;
    account
        .bind_inventory(&listing, &operation())
        .await
        .unwrap();
    let original = f.bytes(record.credential_ref);
    let puts = f.puts();
    for (key, value) in [
        ("sn8", json!("TEST5678")),
        ("modelNumber", json!(525)),
        ("category", json!("different-category")),
    ] {
        let mut entry = with_profile("11");
        entry[key] = value;
        let listing = inventory(record.credential_ref, vec![with_profile("22"), entry]).await;
        assert_eq!(
            account
                .bind_inventory(&listing, &operation())
                .await
                .unwrap_err()
                .code,
            "DEVICE_IDENTITY_CONFLICT"
        );
        assert_eq!(f.bytes(record.credential_ref), original);
        assert_eq!(f.puts(), puts);
    }
    drop(account);
    for (key, value) in [
        ("sn8", json!("00000000")),
        ("subtype", json!("00524")),
        ("subtype", json!("0")),
        ("category", json!("")),
        ("category", json!("x".repeat(129))),
        ("extra", json!(true)),
    ] {
        let mut changed: Value = serde_json::from_slice(&original).unwrap();
        changed["bindings"][0]["product_profile"][key] = value;
        f.set_bytes(record.credential_ref, serde_json::to_vec(&changed).unwrap());
        assert_eq!(
            f.reopen(record.id).await.unwrap_err().code,
            "ACCOUNT_SESSION_UNCONFIRMED"
        );
    }
    for version in [1, 2, 4] {
        let mut changed: Value = serde_json::from_slice(&original).unwrap();
        changed["version"] = json!(version);
        f.set_bytes(record.credential_ref, serde_json::to_vec(&changed).unwrap());
        assert_eq!(
            f.reopen(record.id).await.unwrap_err().code,
            "ACCOUNT_SESSION_UNCONFIRMED"
        );
    }
    assert_eq!(f.puts(), puts);
}

#[tokio::test]
async fn uncertain_product_updates_return_no_bindings_and_keep_the_account_pending() {
    for mode in [1, 2, 3] {
        let f = Fixture::new();
        let mut account = f.account().await;
        let record = account.record().clone();
        let bare = inventory(record.credential_ref, vec![appliance("11", "AC")]).await;
        let original_bindings = account.bind_inventory(&bare, &operation()).await.unwrap();
        let original = f.bytes(record.credential_ref);
        f.secrets.mode.store(mode, Ordering::Relaxed);
        let listing = inventory(record.credential_ref, vec![with_profile("11")]).await;
        assert!(
            account
                .bind_inventory(&listing, &operation())
                .await
                .is_err()
        );
        assert_eq!(
            f.store.account(record.id).unwrap().credential_state,
            CredentialState::Pending
        );
        assert_eq!(
            account.session(&operation()).await.unwrap_err().code,
            "ACCOUNT_SESSION_UNCONFIRMED"
        );
        drop(account);
        if mode == 1 {
            assert_eq!(f.bytes(record.credential_ref), original);
        } else {
            let saved = payload(&f, record.credential_ref);
            let persisted: DeviceBinding =
                serde_json::from_value(saved["bindings"][0]["local"].clone()).unwrap();
            assert_eq!(persisted, original_bindings[0]);
            assert_eq!(saved["bindings"][0]["product_profile"]["sn8"], "TEST1234");
            let puts = f.puts();
            let mut lease = f.vault.lease(record.id, &operation()).await.unwrap();
            let exact = lease.inspect_session(&operation()).await.unwrap().unwrap();
            lease.confirm_session(&exact, &operation()).await.unwrap();
            let mut recovered = MeijuAccount::open(lease, &operation()).await.unwrap();
            assert_eq!(
                recovered
                    .bind_inventory(&listing, &operation())
                    .await
                    .unwrap(),
                original_bindings
            );
            assert_eq!(f.puts(), puts);
        }
    }
}
