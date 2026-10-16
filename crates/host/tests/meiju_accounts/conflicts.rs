use super::*;

fn inspect(account: Uuid, device: Uuid) -> Value {
    json!({"method":"inspect_device_conflicts","account_id":account,"device_id":device})
}

async fn selection(f: &Fixture) -> (AccountRecord, Uuid) {
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let inventory = f
        .call(json!({"method":"inspect_account_inventory","account_id":account.id}))
        .await;
    (
        account,
        inventory["data"]["devices"][0]["device_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap(),
    )
}

async fn duplicate(f: &Fixture, subject: &str) -> Uuid {
    *f.wire.subject.lock().unwrap() = subject.into();
    let connected = f.connect(None).await;
    assert_eq!(
        connected["data"]["retrieval"]["error"]["code"],
        "DEVICE_IDENTITY_CONFLICT"
    );
    connected["data"]["account"]["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

#[tokio::test(start_paused = true)]
async fn saved_conflicts_include_unpublished_and_historical_bindings_without_changes() {
    let mut f = Fixture::new();
    let (account, device) = selection(&f).await;
    let initial = f.call(inspect(account.id, device)).await;
    assert_eq!(initial["data"]["matches"], json!([]));
    let second = duplicate(&f, "SECOND-SUBJECT").await;
    let third = duplicate(&f, "THIRD-SUBJECT").await;
    for (id, membership) in [(second, json!(false)), (third, json!(null))] {
        let record = f.store.account(id).unwrap();
        let mut values = f.secrets.values.lock().unwrap();
        let key = record.credential_ref.to_string();
        let mut payload: Value = serde_json::from_slice(&values[&key]).unwrap();
        for binding in payload["bindings"].as_array_mut().unwrap() {
            binding["listed"] = membership.clone();
        }
        values.insert(key, serde_json::to_vec(&payload).unwrap());
    }
    let before = f.secrets.values.lock().unwrap().clone();
    let cards = f.call(json!({"method":"list_device_cards"})).await;
    let calls = f.calls();
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    let result = f.call(inspect(account.id, device)).await;
    assert_eq!(result["data"]["selected"], initial["data"]["selected"]);
    assert_eq!(result["data"]["physical_identity"], "unverified");
    let matches = result["data"]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 2);
    let mut ids = vec![second.to_string(), third.to_string()];
    ids.sort();
    assert_eq!(
        matches
            .iter()
            .map(|entry| entry["account_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ids
    );
    for entry in matches {
        assert_ne!(entry["device_id"], device.to_string());
        assert!(
            !f.store
                .devices()
                .unwrap()
                .iter()
                .any(|d| d.device.id == entry["device_id"].as_str().unwrap())
        );
        assert_eq!(
            entry["membership"],
            if entry["account_id"] == second.to_string() {
                "not_listed"
            } else {
                "unconfirmed"
            }
        );
    }
    // The unpublished account's local binding can itself be selected for diagnosis.
    let reverse = f
        .call(inspect(
            second,
            matches
                .iter()
                .find(|entry| entry["account_id"] == second.to_string())
                .unwrap()["device_id"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
        ))
        .await;
    assert_eq!(reverse["data"]["matches"].as_array().unwrap().len(), 2);
    f.reopen_with_power_grants(&[]);
    assert_eq!(f.call(inspect(account.id, device)).await, result);
    assert_eq!(f.call(json!({"method":"list_device_cards"})).await, cards);
    assert_eq!(*f.secrets.values.lock().unwrap(), before);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 0);
    assert_eq!(f.calls(), calls);
    for marker in [
        "SYNTHETIC",
        "880000000000001",
        "880000000000002",
        "subject",
        "credential_ref",
        "identity_ref",
        "sn8",
        "token",
    ] {
        assert!(!result.to_string().contains(marker), "{marker}");
    }
}

#[tokio::test(start_paused = true)]
async fn conflict_inspection_refuses_any_unavailable_scope_without_partial_results() {
    let f = Fixture::new();
    let (account, device) = selection(&f).await;
    let other_id = duplicate(&f, "SECOND-SUBJECT").await;
    let other = f.store.account(other_id).unwrap();
    let original = f.secrets.values.lock().unwrap().clone();
    let calls = f.calls();
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    for (account_id, device_id, code) in [
        (Uuid::nil(), device, "INVALID_ARGUMENT"),
        (account.id, Uuid::nil(), "INVALID_ARGUMENT"),
        (Uuid::new_v4(), device, "ACCOUNT_NOT_FOUND"),
        (account.id, Uuid::new_v4(), "DEVICE_IDENTITY_CONFLICT"),
    ] {
        let result = f.call(inspect(account_id, device_id)).await;
        assert_eq!(result["error"]["code"], code);
        assert!(result["data"].is_null());
    }
    let busy = f.vault.lease(other_id, &operation()).await.unwrap();
    assert_eq!(
        f.call(inspect(account.id, device)).await["error"]["code"],
        "RESOURCE_BUSY"
    );
    drop(busy);
    let registration = f.vault.lock_registration(&operation()).await.unwrap();
    assert_eq!(
        f.call(inspect(account.id, device)).await["error"]["code"],
        "RESOURCE_BUSY"
    );
    drop(registration);
    f.store
        .set_credential_state(other_id, CredentialState::Pending)
        .unwrap();
    let pending = f.call(inspect(account.id, device)).await;
    assert_eq!(pending["error"]["code"], "ACCOUNT_SESSION_UNCONFIRMED");
    assert!(pending["data"].is_null());
    f.store
        .set_credential_state(other_id, CredentialState::Stored)
        .unwrap();
    f.secrets.unavailable.store(true, Ordering::Relaxed);
    assert_eq!(
        f.call(inspect(account.id, device)).await["error"]["code"],
        "SECRET_STORE_UNAVAILABLE"
    );
    f.secrets.unavailable.store(false, Ordering::Relaxed);
    f.secrets.values.lock().unwrap().insert(
        other.credential_ref.to_string(),
        b"PRIVATE-CORRUPT".to_vec(),
    );
    let corrupt = f.call(inspect(account.id, device)).await;
    assert_eq!(corrupt["error"]["code"], "ACCOUNT_SESSION_UNCONFIRMED");
    assert!(!corrupt.to_string().contains("PRIVATE"));
    *f.secrets.values.lock().unwrap() = original.clone();
    f.store.begin_account_removal(other_id, &[]).unwrap();
    assert_eq!(
        f.call(inspect(account.id, device)).await["error"]["code"],
        "ACCOUNT_REMOVAL_PENDING"
    );
    assert_eq!(f.calls(), calls);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    assert_eq!(*f.secrets.values.lock().unwrap(), original);
    drop(f.vault.lease(account.id, &operation()).await.unwrap());
    drop(f.vault.lock_registration(&operation()).await.unwrap());
}

#[tokio::test(start_paused = true)]
async fn conflict_scope_is_bounded_before_secret_reads_and_excludes_other_platforms() {
    let f = Fixture::new();
    let (account, device) = selection(&f).await;
    for (provider, region) in [("other.provider", "CN"), ("midea.meiju", "US")] {
        let foreign = f
            .vault
            .prepare(provider, region, "Foreign", &operation())
            .await
            .unwrap();
        assert_eq!(
            f.call(inspect(foreign.record().id, device)).await["error"]["code"],
            "ACCOUNT_NOT_FOUND"
        );
        assert_eq!(
            f.call(inspect(account.id, device)).await["data"]["matches"],
            json!([])
        );
    }
    for index in 0..63 {
        drop(
            MeijuAccount::create(
                &f.vault,
                "CN",
                "Empty",
                &secret(&format!("EMPTY-{index}")),
                &secret("SYNTHETIC-SESSION"),
                &operation(),
            )
            .await
            .unwrap(),
        );
    }
    let complete = f.call(inspect(account.id, device)).await;
    assert!(complete["error"].is_null());
    assert_eq!(complete["data"]["matches"], json!([]));
    drop(
        f.vault
            .prepare("midea.meiju", "CN", "Synthetic", &operation())
            .await
            .unwrap(),
    );
    let reads = f.secrets.reads.load(Ordering::Relaxed);
    let calls = f.calls();
    assert_eq!(
        f.call(inspect(account.id, device)).await["error"]["code"],
        "MIDEA_CONFLICT_LIMIT"
    );
    assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
    assert_eq!(f.calls(), calls);
    drop(f.vault.lock_registration(&operation()).await.unwrap());
}

#[tokio::test(start_paused = true)]
async fn conflict_cancellation_releases_registration_and_account_leases() {
    for stop in ["cancel", "deadline", "background", "close", "drop"] {
        let mut f = Fixture::new();
        let (account, device) = selection(&f).await;
        duplicate(&f, "SECOND-SUBJECT").await;
        let original = f.secrets.values.lock().unwrap().clone();
        let calls = f.calls();
        f.secrets.read_delay.store(true, Ordering::Relaxed);
        let mut work = Box::pin(f.call_id("conflicts", inspect(account.id, device)));
        tokio::select! {
            _ = f.secrets.read_started.notified() => (),
            result = &mut work => panic!("Expected pending inspection: {result}"),
        }
        assert_eq!(
            f.vault
                .lock_registration(&operation())
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
                "cancel" => assert!(f.session.cancel("conflicts")),
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
        f.secrets.read_delay.store(false, Ordering::Relaxed);
        for record in f.store.accounts().unwrap() {
            drop(f.vault.lease(record.id, &operation()).await.unwrap());
        }
        drop(f.vault.lock_registration(&operation()).await.unwrap());
        f.reopen_with_power_grants(&[]);
        assert!(f.call(inspect(account.id, device)).await["error"].is_null());
        assert_eq!(*f.secrets.values.lock().unwrap(), original);
        assert_eq!(f.calls(), calls);
    }
}

#[tokio::test(start_paused = true)]
async fn identical_models_do_not_match_without_equal_saved_provider_identity() {
    let f = Fixture::new();
    let (account, device) = selection(&f).await;
    let other_id = duplicate(&f, "SECOND-SUBJECT").await;
    let other = f.store.account(other_id).unwrap();
    {
        let mut values = f.secrets.values.lock().unwrap();
        let key = other.credential_ref.to_string();
        let mut payload: Value = serde_json::from_slice(&values[&key]).unwrap();
        for binding in payload["bindings"].as_array_mut().unwrap() {
            let id = binding["appliance_id"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap();
            binding["appliance_id"] = json!((id + 1000).to_string());
        }
        values.insert(key, serde_json::to_vec(&payload).unwrap());
    }
    let calls = f.calls();
    let inspected = f.call(inspect(account.id, device)).await;
    assert!(inspected["error"].is_null());
    assert_eq!(inspected["data"]["matches"], json!([]));
    assert_eq!(f.calls(), calls);
}
