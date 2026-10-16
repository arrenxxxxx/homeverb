use super::*;

fn inspect(id: Uuid) -> Value {
    json!({"method":"inspect_account_inventory","account_id":id})
}

#[tokio::test(start_paused = true)]
async fn inspection_reports_saved_membership_without_network_mutation_or_private_identifiers() {
    let mut f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let initial = f.call(inspect(account.id)).await;
    assert_eq!(initial["data"]["account"]["authorization"], "not_checked");
    assert_eq!(initial["data"]["devices"].as_array().unwrap().len(), 2);
    let mut expected = f.store.devices().unwrap().into_iter().map(|d| {
        json!({"device_id":d.device.id,"connection_id":d.device.connections[0].id,"membership":"listed"})
    }).collect::<Vec<_>>();
    expected.sort_by(|a, b| a["device_id"].as_str().cmp(&b["device_id"].as_str()));
    assert_eq!(initial["data"]["devices"], json!(expected));
    let original = f.secrets.values.lock().unwrap().clone();
    let cards = json!(f.store.devices().unwrap());
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    let calls = f.calls();
    f.reopen_with_power_grants(&[]);
    assert_eq!(f.call(inspect(account.id)).await, initial);
    assert_eq!(*f.secrets.values.lock().unwrap(), original);
    assert_eq!(json!(f.store.devices().unwrap()), cards);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 0);
    assert_eq!(f.calls(), calls);
    for marker in [
        "SYNTHETIC",
        "880000000000001",
        "880000000000002",
        "identity_ref",
        "credential_ref",
        "subject",
        "token",
    ] {
        assert!(!initial.to_string().contains(marker), "{marker}");
    }
    f.wire.empty_inventory.store(true, Ordering::Relaxed);
    f.refresh(&account.id.to_string()).await;
    let calls = f.calls();
    let absent = f.call(inspect(account.id)).await;
    assert!(
        absent["data"]["devices"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["membership"] == "not_listed")
    );
    assert_eq!(f.calls(), calls);
    f.wire.inventory_code.store(1306, Ordering::Relaxed);
    f.refresh(&account.id.to_string()).await;
    assert_eq!(f.call(inspect(account.id)).await, absent);
    f.wire.empty_inventory.store(false, Ordering::Relaxed);
    f.wire.inventory_code.store(0, Ordering::Relaxed);
    f.refresh(&account.id.to_string()).await;
    let mut expected = initial;
    expected["data"]["account"]["last_complete_inventory_at"] = json!(
        f.store
            .account(account.id)
            .unwrap()
            .last_complete_inventory_at
            .unwrap()
            .to_rfc3339()
    );
    assert_eq!(f.call(inspect(account.id)).await, expected);
}

#[tokio::test(start_paused = true)]
async fn inspection_preserves_legacy_and_unpublished_bindings_without_inventing_cards() {
    let f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let key = account.credential_ref.to_string();
    {
        let mut values = f.secrets.values.lock().unwrap();
        let mut payload: Value = serde_json::from_slice(&values[&key]).unwrap();
        payload["version"] = json!(1);
        for binding in payload["bindings"].as_array_mut().unwrap() {
            binding.as_object_mut().unwrap().remove("listed");
        }
        values.insert(key.clone(), serde_json::to_vec(&payload).unwrap());
    }
    let before = f.secrets.values.lock().unwrap().clone();
    let calls = f.calls();
    let legacy = f.call(inspect(account.id)).await;
    assert!(
        legacy["data"]["devices"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["membership"] == "unconfirmed")
    );
    assert_eq!(*f.secrets.values.lock().unwrap(), before);
    assert_eq!(f.calls(), calls);

    // A saved identity may exist before successful Core publication.
    f.wire.extra_device.store(true, Ordering::Relaxed);
    let mut leased = MeijuAccount::open(
        f.vault.lease(account.id, &operation()).await.unwrap(),
        &operation(),
    )
    .await
    .unwrap();
    leased
        .retrieve(f.wire.as_ref(), &operation())
        .await
        .unwrap();
    drop(leased);
    let result = f.call(inspect(account.id)).await;
    assert_eq!(result["data"]["devices"].as_array().unwrap().len(), 3);
    assert_eq!(f.store.devices().unwrap().len(), 2);
}

#[tokio::test(start_paused = true)]
async fn inspection_errors_preserve_data_and_never_fall_back_to_refresh() {
    let f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let original = f.secrets.values.lock().unwrap().clone();
    let calls = f.calls();
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    assert_eq!(
        f.call(inspect(Uuid::new_v4())).await["error"]["code"],
        "ACCOUNT_NOT_FOUND"
    );
    f.store
        .set_credential_state(account.id, CredentialState::Pending)
        .unwrap();
    let result = f.call(inspect(account.id)).await;
    assert!(result["data"].is_null());
    assert_eq!(result["error"]["code"], "ACCOUNT_SESSION_UNCONFIRMED");
    f.store
        .set_credential_state(account.id, CredentialState::Stored)
        .unwrap();
    let lease = f.vault.lease(account.id, &operation()).await.unwrap();
    assert_eq!(
        f.call(inspect(account.id)).await["error"]["code"],
        "RESOURCE_BUSY"
    );
    drop(lease);
    f.secrets.unavailable.store(true, Ordering::Relaxed);
    assert_eq!(
        f.call(inspect(account.id)).await["error"]["code"],
        "SECRET_STORE_UNAVAILABLE"
    );
    assert!(
        f.call(json!({"method":"get_account","account_id":account.id}))
            .await["error"]
            .is_null()
    );
    f.secrets.unavailable.store(false, Ordering::Relaxed);
    f.secrets.values.lock().unwrap().insert(
        account.credential_ref.to_string(),
        b"PRIVATE-CORRUPT".to_vec(),
    );
    let corrupt = f.call(inspect(account.id)).await;
    assert_eq!(corrupt["error"]["code"], "ACCOUNT_SESSION_UNCONFIRMED");
    assert!(!corrupt.to_string().contains("PRIVATE"));
    *f.secrets.values.lock().unwrap() = original;
    let ids: Vec<_> = f
        .store
        .devices()
        .unwrap()
        .into_iter()
        .map(|d| d.device.id)
        .collect();
    f.store.begin_account_removal(account.id, &ids).unwrap();
    let reads = f.secrets.reads.load(Ordering::Relaxed);
    assert_eq!(
        f.call(inspect(account.id)).await["error"]["code"],
        "ACCOUNT_REMOVAL_PENDING"
    );
    assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
    assert_eq!(f.calls(), calls);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 0);
}

#[tokio::test(start_paused = true)]
async fn inspection_isolates_account_and_region_without_touching_other_secrets() {
    let f = Fixture::new();
    f.connect(None).await;
    let primary = f.store.accounts().unwrap().remove(0);
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
    let busy = f.vault.lease(primary.id, &operation()).await.unwrap();
    let inspected = f.call(inspect(other_id)).await;
    assert_eq!(inspected["data"]["account"]["id"], other_id.to_string());
    assert_eq!(inspected["data"]["devices"], json!([]));
    drop(busy);
    for (provider, region) in [("other.provider", "CN"), ("midea.meiju", "US")] {
        let lease = f
            .vault
            .prepare(provider, region, "Foreign", &operation())
            .await
            .unwrap();
        let id = lease.record().id;
        drop(lease);
        let reads = f.secrets.reads.load(Ordering::Relaxed);
        assert_eq!(
            f.call(inspect(id)).await["error"]["code"],
            "ACCOUNT_NOT_FOUND"
        );
        assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
    }
}

#[tokio::test(start_paused = true)]
async fn cancelled_inspection_releases_lease_without_network_or_mutation() {
    for stop in ["cancel", "deadline", "background", "close", "drop"] {
        let mut f = Fixture::new();
        f.connect(None).await;
        let account = f.store.accounts().unwrap().remove(0);
        let calls = f.calls();
        let original = f.secrets.values.lock().unwrap().clone();
        f.secrets.read_delay.store(true, Ordering::Relaxed);
        let mut work = Box::pin(f.call_id("inspect", inspect(account.id)));
        tokio::select! {
            _ = f.secrets.read_started.notified() => (),
            value = &mut work => panic!("Expected pending inspection: {value}"),
        }
        assert_eq!(
            f.vault
                .lease(account.id, &operation())
                .await
                .err()
                .unwrap()
                .code,
            "RESOURCE_BUSY"
        );
        if stop == "drop" {
            drop(work);
        } else {
            match stop {
                "cancel" => assert!(f.session.cancel("inspect")),
                "background" => f.session.set_foreground(false),
                "close" => f.session.close(),
                _ => tokio::time::advance(std::time::Duration::from_secs(2)).await,
            }
            let result = work.await;
            assert_eq!(
                result["error"]["code"],
                if stop == "deadline" {
                    "DEADLINE_EXCEEDED"
                } else {
                    "CANCELLED"
                }
            );
        }
        drop(f.vault.lease(account.id, &operation()).await.unwrap());
        assert_eq!(f.calls(), calls);
        assert_eq!(*f.secrets.values.lock().unwrap(), original);
        f.secrets.read_delay.store(false, Ordering::Relaxed);
        f.reopen_with_power_grants(&[]);
        assert!(f.call(inspect(account.id)).await["error"].is_null());
    }
}
