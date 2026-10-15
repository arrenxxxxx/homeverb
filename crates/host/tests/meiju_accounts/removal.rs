use super::*;

fn removal_request(connection: &Value) -> Value {
    json!({"method":"remove_account","account_id":connection["data"]["account"]["id"],
        "expected_device_ids":connection["data"]["retrieval"]["device_ids"]})
}

#[tokio::test]
async fn removal_of_unreadable_credentials_never_reads_secrets_or_changes_another_account() {
    for state in [CredentialState::Stored, CredentialState::Pending] {
        let f = Fixture::new();
        let connected = f.connect(None).await;
        let id = Uuid::parse_str(connected["data"]["account"]["id"].as_str().unwrap()).unwrap();
        f.store.set_credential_state(id, state).unwrap();
        let mut other = f
            .vault
            .prepare("other.provider", "CN", "Kept", &operation())
            .await
            .unwrap();
        other
            .store_session(&secret("FICTIONAL-KEPT-SESSION"), &operation())
            .await
            .unwrap();
        let other_id = other.record().id;
        let other_reference = other.record().credential_ref.to_string();
        drop(other);
        let calls = f.calls();
        let reads = f.secrets.reads.load(Ordering::Relaxed);
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        // Model unreadable encrypted data with an available exact-reference delete path.
        f.secrets.unavailable.store(true, Ordering::Relaxed);
        f.secrets.deletion.store(1, Ordering::Relaxed);
        let request = removal_request(&connected);
        let result = f.call(request.clone()).await;
        assert_eq!(result["data"]["status"], "removed");
        assert!(result["data"]["error"].is_null());
        assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 1);
        assert_eq!(f.store.accounts().unwrap().len(), 1);
        assert_eq!(f.store.account(other_id).unwrap().label, "Kept");
        assert!(f.store.devices().unwrap().is_empty());
        {
            let remaining = f.secrets.values.lock().unwrap();
            assert_eq!(remaining.len(), 1);
            assert_eq!(remaining[&other_reference], b"FICTIONAL-KEPT-SESSION");
        }
        assert_eq!(f.call(request).await, result);
        assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
        assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 1);
        assert_eq!(f.calls(), calls);
    }
}

#[tokio::test]
async fn removal_keeps_receipts_and_rooms_and_refuses_late_discovery_and_cache() {
    let f = Fixture::new();
    let connected = f.connect(None).await;
    let account = Uuid::parse_str(connected["data"]["account"]["id"].as_str().unwrap()).unwrap();
    let descriptors = f.store.devices().unwrap();
    let device = &descriptors[0].device.id;
    let room = f
        .call(json!({"method":"create_room","name":"Kept room"}))
        .await;
    f.call(json!({"method":"set_device_preferences","device_id":device,
        "preferences":{"alias":"Removed card","room_id":room["data"]["id"],"favorite":true}}))
        .await;
    let mut snapshot: homeverb_domain::Snapshot =
        serde_json::from_str(include_str!("../../../../examples/state.json")).unwrap();
    snapshot.device_id = device.clone();
    f.store.save_snapshot(&snapshot).unwrap();
    let mut command: homeverb_domain::Command =
        serde_json::from_value(power_command(device, false)).unwrap();
    command.request_id = "keep-uncertainty".into();
    let mut receipt = homeverb_domain::Execution::new(&command, SystemClock.now());
    receipt.status = homeverb_domain::Status::Unknown;
    receipt.error = Some(Error::new(
        "WRITE_OUTCOME_UNKNOWN",
        "Synthetic lost acknowledgement.",
    ));
    let _device_lock = f.store.lock_device(device).unwrap();
    f.store.save_execution(&command, &receipt).unwrap();
    let calls = f.calls();
    f.secrets.deletion.store(1, Ordering::Relaxed);
    let request = removal_request(&connected);
    let removed = f.call(request.clone()).await;
    assert_eq!(removed["data"]["status"], "removed");
    assert_eq!(removed["data"]["remote_revocation"], "not_attempted");
    assert!(removed["data"]["error"].is_null());
    assert!(f.store.accounts().unwrap().is_empty());
    assert!(f.store.devices().unwrap().is_empty());
    assert!(f.store.preferences().unwrap().is_empty());
    assert!(f.store.snapshot(device, "main").unwrap().is_none());
    assert_eq!(f.store.rooms().unwrap().len(), 1);
    f.store.finish_execution(&command, &receipt).unwrap();
    assert_eq!(
        f.call(json!({"method":"get_execution","request_id":"keep-uncertainty"}))
            .await["data"],
        json!(receipt)
    );
    assert!(f.secrets.values.lock().unwrap().is_empty());
    let reads = f.secrets.reads.load(Ordering::Relaxed);
    f.secrets.unavailable.store(true, Ordering::Relaxed);
    assert_eq!(f.call(request).await, removed);
    assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 1);
    let store = Arc::new(SqliteStore::open(&f.path).unwrap());
    let adapter = Arc::new(MeijuAdapter::new(
        f.vault.clone(),
        f.wire.clone(),
        Arc::new(SystemClock),
    ));
    let core = Core::new(
        store.clone(),
        vec![adapter],
        Arc::new(SystemClock),
        Policy::default(),
    );
    assert_eq!(
        core.publish_discovery("midea.meiju", &descriptors, &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_NOT_FOUND"
    );
    assert_eq!(
        store.save_snapshot(&snapshot).unwrap_err().code,
        "DEVICE_NOT_FOUND"
    );
    assert!(store.account_removal(account).unwrap().unwrap().completed);
    assert!(store.devices().unwrap().is_empty());
    assert_eq!(f.calls(), calls);
}

#[tokio::test]
async fn removal_pending_cleanup_is_visible_and_explicit_retry_delegates_exact_entry_check() {
    for mode in [2, 3] {
        let f = Fixture::new();
        let connected = f.connect(None).await;
        let id = connected["data"]["account"]["id"].as_str().unwrap();
        let request = removal_request(&connected);
        f.secrets.deletion.store(mode, Ordering::Relaxed);
        let pending = f.call(request.clone()).await;
        assert_eq!(pending["data"]["status"], "pending");
        assert_eq!(
            pending["data"]["error"]["code"],
            if mode == 2 {
                "SECRET_STORE_WRITE_UNKNOWN"
            } else {
                "SECRET_STORE_UNAVAILABLE"
            }
        );
        assert_eq!(
            f.call(json!({"method":"get_account","account_id":id}))
                .await["data"]["account"]["credential_state"],
            "removing"
        );
        assert_eq!(f.store.devices().unwrap().len(), 2);
        let calls = f.calls();
        assert_eq!(
            f.refresh(id).await["error"]["code"],
            "ACCOUNT_REMOVAL_PENDING"
        );
        assert_eq!(
            f.connect(Some(Uuid::parse_str(id).unwrap())).await["error"]["code"],
            "ACCOUNT_REMOVAL_PENDING"
        );
        assert_eq!(
            f.store
                .set_credential_state(Uuid::parse_str(id).unwrap(), CredentialState::Stored)
                .unwrap_err()
                .code,
            "ACCOUNT_REMOVAL_PENDING"
        );
        assert_eq!(f.calls(), calls);
        let store = Arc::new(SqliteStore::open(&f.path).unwrap());
        let vault = AccountVault::new(store, f.secrets.clone());
        f.secrets.deletion.store(1, Ordering::Relaxed);
        let ids: Vec<String> =
            serde_json::from_value(request["expected_device_ids"].clone()).unwrap();
        let recovered = vault
            .remove_account(
                Uuid::parse_str(id).unwrap(),
                "midea.meiju",
                "CN",
                &ids,
                &operation(),
            )
            .await
            .unwrap();
        assert_eq!(recovered.status, "removed");
        assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 2);
        assert_eq!(f.secrets.deleted_entries.load(Ordering::Relaxed), 1);
        assert!(f.store.accounts().unwrap().is_empty());
    }
}

#[tokio::test]
async fn removal_scope_and_lease_failures_precede_mutation() {
    let f = Fixture::new();
    let connected = f.connect(None).await;
    let id = Uuid::parse_str(connected["data"]["account"]["id"].as_str().unwrap()).unwrap();
    let request = removal_request(&connected);
    let mut wrong = request.clone();
    wrong["expected_device_ids"] = json!([]);
    assert_eq!(
        f.call(wrong).await["error"]["code"],
        "ACCOUNT_SCOPE_CHANGED"
    );
    let lease = f.vault.lease(id, &operation()).await.unwrap();
    assert_eq!(
        f.call(request.clone()).await["error"]["code"],
        "RESOURCE_BUSY"
    );
    drop(lease);
    let foreign = f
        .vault
        .prepare("other.provider", "CN", "Other", &operation())
        .await
        .unwrap();
    let foreign_id = foreign.record().id;
    drop(foreign);
    assert_eq!(
        f.call(json!({"method":"remove_account","account_id":foreign_id,"expected_device_ids":[]}))
            .await["error"]["code"],
        "ACCOUNT_NOT_FOUND"
    );
    let mut descriptor = f.store.devices().unwrap().remove(0);
    let mut extra = descriptor.device.connections[0].clone();
    extra.id = "another-route".into();
    extra.account_id = Some(foreign_id.to_string());
    descriptor.device.connections.push(extra);
    let db = rusqlite::Connection::open(f.path.join("homeverb.sqlite3")).unwrap();
    db.execute(
        "UPDATE devices SET descriptor=?2 WHERE id=?1",
        rusqlite::params![
            descriptor.device.id,
            serde_json::to_string(&descriptor).unwrap()
        ],
    )
    .unwrap();
    assert_eq!(
        f.call(request).await["error"]["code"],
        "ACCOUNT_REMOVAL_UNSUPPORTED"
    );
    assert!(f.store.account_removal(id).unwrap().is_none());
    assert_eq!(
        f.store.account(id).unwrap().credential_state,
        CredentialState::Stored
    );
    assert_eq!(f.store.devices().unwrap().len(), 2);
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn removal_metadata_failure_rolls_back_and_retry_does_not_repeat_secret_deletion() {
    let f = Fixture::new();
    let connected = f.connect(None).await;
    let request = removal_request(&connected);
    let db = rusqlite::Connection::open(f.path.join("homeverb.sqlite3")).unwrap();
    db.execute_batch("CREATE TRIGGER refuse_removal BEFORE DELETE ON accounts BEGIN SELECT RAISE(ABORT, 'synthetic'); END").unwrap();
    f.secrets.deletion.store(1, Ordering::Relaxed);
    let result = f.call(request.clone()).await;
    assert_eq!(result["data"]["status"], "pending");
    assert_eq!(result["data"]["error"]["code"], "STORAGE_ERROR");
    assert_eq!(f.store.devices().unwrap().len(), 2);
    assert_eq!(f.store.preferences().unwrap().len(), 2);
    assert_eq!(
        f.store.accounts().unwrap()[0].credential_state,
        CredentialState::Removing
    );
    assert!(f.secrets.values.lock().unwrap().is_empty());
    for descriptor in f.store.devices().unwrap() {
        assert_eq!(
            f.store.save_device(&descriptor).unwrap_err().code,
            "ACCOUNT_REMOVAL_PENDING"
        );
        let mut snapshot: homeverb_domain::Snapshot =
            serde_json::from_str(include_str!("../../../../examples/state.json")).unwrap();
        snapshot.device_id = descriptor.device.id;
        assert_eq!(
            f.store.save_snapshot(&snapshot).unwrap_err().code,
            "ACCOUNT_REMOVAL_PENDING"
        );
    }
    db.execute_batch("DROP TRIGGER refuse_removal").unwrap();
    assert_eq!(f.call(request).await["data"]["status"], "removed");
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 2);
    assert_eq!(f.secrets.deleted_entries.load(Ordering::Relaxed), 1);
}

#[tokio::test(start_paused = true)]
async fn removal_interruption_retains_cleanup_intent_and_releases_leases() {
    for stop in ["cancel", "background", "close", "deadline"] {
        let f = Fixture::new();
        let connected = f.connect(None).await;
        let id = Uuid::parse_str(connected["data"]["account"]["id"].as_str().unwrap()).unwrap();
        f.secrets.deletion.store(4, Ordering::Relaxed);
        let work = f.call_id("remove", removal_request(&connected));
        tokio::pin!(work);
        tokio::select! {
            _ = f.secrets.delete_started.notified() => (),
            value = &mut work => panic!("Expected an in-flight secret deletion: {value}"),
        }
        match stop {
            "cancel" => {
                assert!(f.session.cancel("remove"));
            }
            "background" => f.session.set_foreground(false),
            "close" => f.session.close(),
            _ => tokio::time::advance(std::time::Duration::from_secs(2)).await,
        }
        let result = work.await;
        assert_eq!(result["data"]["status"], "pending");
        assert_eq!(
            result["data"]["error"]["code"],
            "SECRET_STORE_WRITE_UNKNOWN"
        );
        assert_eq!(
            f.store.account(id).unwrap().credential_state,
            CredentialState::Removing
        );
        assert_eq!(f.store.devices().unwrap().len(), 2);
        drop(f.vault.lock_registration(&operation()).await.unwrap());
        drop(f.store.lock_account(id).unwrap());
    }
}

#[tokio::test]
async fn dropped_removal_has_no_callback_or_startup_retry_and_can_resume_explicitly() {
    let f = Fixture::new();
    let connected = f.connect(None).await;
    let id = Uuid::parse_str(connected["data"]["account"]["id"].as_str().unwrap()).unwrap();
    f.secrets.deletion.store(4, Ordering::Relaxed);
    let mut work = Box::pin(f.call_id("remove", removal_request(&connected)));
    tokio::select! {
        _ = f.secrets.delete_started.notified() => (),
        value = &mut work => panic!("Unexpected completion: {value}"),
    }
    drop(work);
    let store = Arc::new(SqliteStore::open(&f.path).unwrap());
    assert_eq!(
        store.account(id).unwrap().credential_state,
        CredentialState::Removing
    );
    assert_eq!(store.devices().unwrap().len(), 2);
    let vault = AccountVault::new(store, f.secrets.clone());
    let ids: Vec<String> =
        serde_json::from_value(connected["data"]["retrieval"]["device_ids"].clone()).unwrap();
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 1);
    assert_eq!(
        vault
            .remove_account(id, "midea.meiju", "CN", &ids, &operation())
            .await
            .unwrap()
            .status,
        "removed"
    );
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 2);
    assert_eq!(f.secrets.deleted_entries.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn deletion_refusals_leave_intent_without_erasure_and_scope_retry_is_exact() {
    for (mode, error) in [
        (3, "SECRET_STORE_UNAVAILABLE"),
        (5, "SECRET_STORE_LOCKED"),
        (6, "SECRET_STORE_CONFLICT"),
    ] {
        let f = Fixture::new();
        let connected = f.connect(None).await;
        let request = removal_request(&connected);
        let reads = f.secrets.reads.load(Ordering::Relaxed);
        let saved = f.secrets.values.lock().unwrap().clone();
        let calls = f.calls();
        f.secrets.unavailable.store(true, Ordering::Relaxed);
        f.secrets.deletion.store(mode, Ordering::Relaxed);
        let pending = f.call(request.clone()).await;
        assert_eq!(pending["data"]["status"], "pending");
        assert_eq!(pending["data"]["error"]["code"], error);
        assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
        assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 1);
        assert_eq!(*f.secrets.values.lock().unwrap(), saved);
        assert_eq!(f.store.devices().unwrap().len(), 2);
        let mut changed = request.clone();
        changed["expected_device_ids"] = json!([]);
        assert_eq!(
            f.call(changed).await["error"]["code"],
            "ACCOUNT_SCOPE_CHANGED"
        );
        assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 1);
        f.secrets.deletion.store(1, Ordering::Relaxed);
        assert_eq!(f.call(request).await["data"]["status"], "removed");
        assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
        assert_eq!(f.secrets.deleted_entries.load(Ordering::Relaxed), 1);
        assert_eq!(f.calls(), calls);
    }
}
