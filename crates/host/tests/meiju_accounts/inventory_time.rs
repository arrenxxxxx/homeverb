use super::*;

fn historical() -> chrono::DateTime<chrono::Utc> {
    "2020-01-01T00:00:00Z".parse().unwrap()
}

#[tokio::test(start_paused = true)]
async fn incomplete_initial_connection_and_uncertain_bindings_do_not_invent_complete_time() {
    for code in [1306, 7400] {
        let f = Fixture::new();
        f.wire.inventory_code.store(code, Ordering::Relaxed);
        let result = f.connect(None).await;
        assert_eq!(result["data"]["authentication"], "succeeded");
        assert!(result["data"]["account"]["last_complete_inventory_at"].is_null());
        assert!(
            f.store.accounts().unwrap()[0]
                .last_complete_inventory_at
                .is_none()
        );
    }
    let f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    f.store
        .record_complete_inventory(account.id, historical())
        .unwrap();
    f.wire.extra_device.store(true, Ordering::Relaxed);
    f.secrets.uncertain.store(true, Ordering::Relaxed);
    let result = f.refresh(&account.id.to_string()).await;
    assert_eq!(
        result["data"]["retrieval"]["error"]["code"],
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    let updated = f.store.account(account.id).unwrap();
    assert_eq!(updated.last_complete_inventory_at, Some(historical()));
    assert_eq!(updated.credential_state, CredentialState::Pending);
}

#[tokio::test(start_paused = true)]
async fn timestamp_failure_preserves_completed_binding_storage_without_publishing_new_cards() {
    let f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    f.store
        .record_complete_inventory(account.id, historical())
        .unwrap();
    f.wire.extra_device.store(true, Ordering::Relaxed);
    let database = rusqlite::Connection::open(f.path.join("homeverb.sqlite3")).unwrap();
    database.execute_batch("CREATE TRIGGER reject_inventory_time BEFORE UPDATE OF last_complete_inventory_at ON accounts BEGIN SELECT RAISE(ABORT, 'synthetic'); END").unwrap();
    let result = f.refresh(&account.id.to_string()).await;
    assert_eq!(
        result["data"]["retrieval"]["error"]["code"],
        "STORAGE_ERROR"
    );
    assert_eq!(result["data"]["account"]["credential_state"], "stored");
    assert_eq!(
        result["data"]["account"]["last_complete_inventory_at"],
        historical().to_rfc3339()
    );
    let inventory = f
        .call(json!({"method":"inspect_account_inventory","account_id":account.id}))
        .await;
    assert_eq!(inventory["data"]["devices"].as_array().unwrap().len(), 3);
    assert_eq!(f.store.devices().unwrap().len(), 2);
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    database
        .execute_batch("DROP TRIGGER reject_inventory_time")
        .unwrap();
    assert_eq!(
        f.refresh(&account.id.to_string()).await["data"]["retrieval"]["status"],
        "complete"
    );
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    assert_eq!(f.store.devices().unwrap().len(), 3);
}

#[tokio::test(start_paused = true)]
async fn complete_and_empty_retrieval_record_time_without_rewriting_unchanged_credentials() {
    let mut f = Fixture::new();
    let before = chrono::Utc::now();
    let connected = f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let recorded = account.last_complete_inventory_at.unwrap();
    assert!(recorded >= before && recorded <= chrono::Utc::now());
    assert_eq!(
        connected["data"]["account"]["last_complete_inventory_at"],
        recorded.to_rfc3339()
    );
    f.store
        .record_complete_inventory(account.id, historical())
        .unwrap();
    let secrets = f.secrets.values.lock().unwrap().clone();
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    let result = f.refresh(&account.id.to_string()).await;
    assert_eq!(result["data"]["retrieval"]["status"], "complete");
    assert_ne!(
        f.store
            .account(account.id)
            .unwrap()
            .last_complete_inventory_at,
        Some(historical())
    );
    assert_eq!(*f.secrets.values.lock().unwrap(), secrets);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);

    f.wire.empty_inventory.store(true, Ordering::Relaxed);
    f.store
        .record_complete_inventory(account.id, historical())
        .unwrap();
    assert_eq!(
        f.refresh(&account.id.to_string()).await["data"]["retrieval"]["device_ids"],
        json!([])
    );
    let updated = f
        .store
        .account(account.id)
        .unwrap()
        .last_complete_inventory_at
        .unwrap();
    assert_ne!(updated, historical());
    let calls = f.calls();
    f.reopen_with_power_grants(&[]);
    f.secrets.unavailable.store(true, Ordering::Relaxed);
    assert_eq!(
        f.call(json!({"method":"list_accounts"})).await["data"][0]["last_complete_inventory_at"],
        updated.to_rfc3339()
    );
    assert_eq!(
        f.call(json!({"method":"get_account","account_id":account.id}))
            .await["data"]["account"]["last_complete_inventory_at"],
        updated.to_rfc3339()
    );
    assert_eq!(
        f.call(json!({"method":"rename_account","account_id":account.id,"label":"Renamed"}))
            .await["data"]["last_complete_inventory_at"],
        updated.to_rfc3339()
    );
    assert_eq!(f.calls(), calls);
    assert_eq!(
        SqliteStore::open(&f.path)
            .unwrap()
            .account(account.id)
            .unwrap()
            .last_complete_inventory_at,
        Some(updated)
    );
}

#[tokio::test(start_paused = true)]
async fn failures_partial_results_and_rejected_login_preserve_historical_time() {
    for fault in ["homes", "partial", "expired", "login", "storage"] {
        let f = Fixture::new();
        f.connect(None).await;
        let account = f.store.accounts().unwrap().remove(0);
        f.store
            .record_complete_inventory(account.id, historical())
            .unwrap();
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        let original = f.secrets.values.lock().unwrap().clone();
        let database = rusqlite::Connection::open(f.path.join("homeverb.sqlite3")).unwrap();
        match fault {
            "homes" => f.wire.homes_code.store(1306, Ordering::Relaxed),
            "partial" => f.wire.inventory_code.store(1306, Ordering::Relaxed),
            "expired" => f.wire.inventory_code.store(7400, Ordering::Relaxed),
            "login" => f.wire.login_code.store(7400, Ordering::Relaxed),
            _ => database.execute_batch("CREATE TRIGGER reject_inventory_time BEFORE UPDATE OF last_complete_inventory_at ON accounts BEGIN SELECT RAISE(ABORT, 'synthetic'); END").unwrap(),
        }
        let result = if fault == "login" {
            f.connect(Some(account.id)).await
        } else {
            f.refresh(&account.id.to_string()).await
        };
        let error = if fault == "login" {
            &result["error"]
        } else {
            &result["data"]["retrieval"]["error"]
        };
        assert_eq!(
            error["code"],
            match fault {
                "homes" => "MIDEA_REQUEST_REJECTED",
                "partial" => "MIDEA_INVENTORY_PARTIAL",
                "storage" => "STORAGE_ERROR",
                _ => "AUTH_REQUIRED",
            }
        );
        assert_eq!(
            f.store
                .account(account.id)
                .unwrap()
                .last_complete_inventory_at,
            Some(historical())
        );
        assert_eq!(
            f.store.account(account.id).unwrap().credential_state,
            CredentialState::Stored
        );
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        assert_eq!(*f.secrets.values.lock().unwrap(), original);
        drop(f.vault.lease(account.id, &operation()).await.unwrap());
    }
}

#[tokio::test(start_paused = true)]
async fn pending_recovery_records_complete_time_and_lifecycle_interruptions_preserve_it() {
    let f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    f.store
        .record_complete_inventory(account.id, historical())
        .unwrap();
    f.store
        .set_credential_state(account.id, CredentialState::Pending)
        .unwrap();
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    let result = f
        .call(json!({"method":"recover_account","account_id":account.id}))
        .await;
    assert_eq!(result["data"]["retrieval"]["status"], "complete");
    assert_ne!(
        f.store
            .account(account.id)
            .unwrap()
            .last_complete_inventory_at,
        Some(historical())
    );
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    for stop in ["cancel", "deadline", "background", "close", "drop"] {
        let mut f = Fixture::new();
        f.connect(None).await;
        let account = f.store.accounts().unwrap().remove(0);
        f.store
            .record_complete_inventory(account.id, historical())
            .unwrap();
        *f.wire.delay.lock().unwrap() = Some(CloudEndpoint::HomeAppliances);
        let id = account.id.to_string();
        let mut work = Box::pin(f.refresh(&id));
        tokio::select! {
            _ = f.wire.started.notified() => (),
            value = &mut work => panic!("Expected a pending retrieval: {value}"),
        }
        if stop == "drop" {
            drop(work);
        } else {
            match stop {
                "cancel" => assert!(f.session.cancel("query")),
                "background" => f.session.set_foreground(false),
                "close" => f.session.close(),
                _ => tokio::time::advance(std::time::Duration::from_secs(2)).await,
            }
            assert!(work.await["data"]["retrieval"]["error"].is_object());
        }
        assert_eq!(
            f.store
                .account(account.id)
                .unwrap()
                .last_complete_inventory_at,
            Some(historical())
        );
        drop(f.vault.lease(account.id, &operation()).await.unwrap());
        f.reopen_with_power_grants(&[]);
    }
}
