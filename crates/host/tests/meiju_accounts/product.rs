use super::*;

fn saved(f: &Fixture, account: &AccountRecord) -> Value {
    serde_json::from_slice(&f.secrets.values.lock().unwrap()[&account.credential_ref.to_string()])
        .unwrap()
}

fn assert_read_only_profile_enrichment(f: &Fixture, original: &Value) {
    let current = json!(f.store.devices().unwrap());
    assert_eq!(
        current.as_array().unwrap().len(),
        original.as_array().unwrap().len()
    );
    for before in original.as_array().unwrap() {
        let after = current
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["device"]["id"] == before["device"]["id"])
            .unwrap();
        assert_eq!(after["device"], before["device"]);
        for previous in before["capabilities"].as_array().unwrap() {
            let cap = after["capabilities"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["id"] == previous["id"])
                .unwrap();
            if cap["id"] != "vendor.midea.ac" {
                assert_eq!(cap, previous);
                continue;
            }
            assert!(cap["actions"].as_object().unwrap().is_empty());
            for (name, value) in previous["properties"].as_object().unwrap() {
                assert_eq!(&cap["properties"][name], value);
            }
            for (name, property) in cap["properties"].as_object().unwrap() {
                if previous["properties"].get(name).is_none() {
                    assert_eq!(property["control_requires_observation"], true);
                    assert_eq!(property["writable"], false);
                }
            }
        }
    }
}

#[tokio::test(start_paused = true)]
async fn product_evidence_stays_protected_across_refresh_and_pending_reauthorization() {
    let f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let devices = json!(f.store.devices().unwrap());
    *f.wire.product_fields.lock().unwrap() =
        Some(json!({"sn8":"TEST1234","modelNumber":524,"category":"PRIVATE-PRODUCT-CATEGORY"}));
    let refreshed = f.refresh(&account.id.to_string()).await;
    assert_eq!(refreshed["data"]["retrieval"]["status"], "complete");
    assert_read_only_profile_enrichment(&f, &devices);
    let protected = saved(&f, &account);
    assert_eq!(protected["version"], 3);
    assert_eq!(
        protected["bindings"][0]["product_profile"]["sn8"],
        "TEST1234"
    );
    let mut responses = vec![refreshed];
    for request in [
        json!({"method":"list_device_cards"}),
        json!({"method":"list_accounts"}),
        json!({"method":"get_account","account_id":account.id}),
        json!({"method":"inspect_account_inventory","account_id":account.id}),
        json!({"method":"diagnostics"}),
    ] {
        responses.push(f.call(request).await);
    }
    for device in f.store.devices().unwrap() {
        let result = f
            .call(json!({"method":"describe_device","device":device.device.id}))
            .await;
        assert!(result["error"].is_null());
        assert!(
            device
                .capabilities
                .iter()
                .all(|capability| capability.actions.is_empty())
        );
        responses.push(result);
    }
    f.store
        .set_credential_state(account.id, CredentialState::Pending)
        .unwrap();
    *f.wire.product_fields.lock().unwrap() = None;
    let reauthorized = f.connect(Some(account.id)).await;
    assert_eq!(reauthorized["data"]["retrieval"]["status"], "complete");
    assert_eq!(saved(&f, &account)["bindings"], protected["bindings"]);
    responses.push(reauthorized);
    for marker in [
        "TEST1234",
        "PRIVATE-PRODUCT-CATEGORY",
        "product_profile",
        "subtype",
    ] {
        assert!(!json!(responses).to_string().contains(marker));
        for name in ["homeverb.sqlite3", "homeverb.sqlite3-wal"] {
            if let Ok(bytes) = std::fs::read(f.path.join(name)) {
                assert!(
                    !bytes
                        .windows(marker.len())
                        .any(|window| window == marker.as_bytes())
                );
            }
        }
    }
    assert!(!f.calls().contains(&CloudEndpoint::ApplianceControl));
}

#[tokio::test(start_paused = true)]
async fn product_storage_uncertainty_recovers_without_relogin_or_duplicate_cards() {
    let f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let devices = json!(f.store.devices().unwrap());
    *f.wire.product_fields.lock().unwrap() = Some(json!({"sn8":"TEST1234","modelNumber":524}));
    f.secrets.uncertain.store(true, Ordering::Relaxed);
    let result = f.refresh(&account.id.to_string()).await;
    assert_eq!(result["data"]["account"]["credential_state"], "pending");
    assert_eq!(
        result["data"]["retrieval"]["error"]["code"],
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    let protected = saved(&f, &account);
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    let before = f.calls().len();
    f.secrets.uncertain.store(false, Ordering::Relaxed);
    let recovered = f
        .call(json!({"method":"recover_account","account_id":account.id}))
        .await;
    assert_eq!(recovered["data"]["retrieval"]["status"], "complete");
    assert_eq!(recovered["data"]["account"]["credential_state"], "stored");
    assert_eq!(
        &f.calls()[before..],
        [CloudEndpoint::Homes, CloudEndpoint::HomeAppliances]
    );
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    assert_eq!(saved(&f, &account), protected);
    assert_read_only_profile_enrichment(&f, &devices);
}
