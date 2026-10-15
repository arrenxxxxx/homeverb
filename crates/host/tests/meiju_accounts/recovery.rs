use super::*;

fn recover(id: Uuid) -> Value {
    json!({"method":"recover_account","account_id":id})
}

async fn pending_account(f: &Fixture) -> AccountRecord {
    let connected = f.connect(None).await;
    assert_eq!(
        connected["data"]["retrieval"]["status"], "complete",
        "{connected}"
    );
    assert_eq!(f.store.devices().unwrap().len(), 2);
    let account = f.store.accounts().unwrap().remove(0);
    f.store
        .set_credential_state(account.id, CredentialState::Pending)
        .unwrap();
    account
}

#[tokio::test(start_paused = true)]
async fn recovery_completes_interrupted_initial_connection_without_another_login() {
    let f = Fixture::new();
    f.secrets.uncertain.store(true, Ordering::Relaxed);
    assert_eq!(
        f.connect(None).await["error"]["code"],
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    let account = f.store.accounts().unwrap().remove(0);
    assert_eq!(account.credential_state, CredentialState::Pending);
    assert!(f.store.devices().unwrap().is_empty());
    f.secrets.uncertain.store(false, Ordering::Relaxed);
    let before = f.calls().len();
    let result = f.call(recover(account.id)).await;
    assert_eq!(result["data"]["account"]["id"], account.id.to_string());
    assert_eq!(result["data"]["account"]["credential_state"], "stored");
    assert_eq!(result["data"]["authentication"], "not_attempted");
    assert_eq!(result["data"]["retrieval"]["status"], "complete");
    assert_eq!(
        result["data"]["retrieval"]["device_ids"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        &f.calls()[before..],
        [CloudEndpoint::Homes, CloudEndpoint::HomeAppliances]
    );
    // One original session write and one new binding-map write, never a repeated login write.
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), 2);
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 0);
    let stored = f.call(recover(account.id)).await;
    assert_eq!(stored["data"]["retrieval"], result["data"]["retrieval"]);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), 2);
    assert_eq!(f.store.accounts().unwrap().len(), 1);
}

#[tokio::test(start_paused = true)]
async fn recovery_reuses_uncertain_saved_bindings_and_preserves_card_preferences() {
    let f = Fixture::new();
    let connected = f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let device = &connected["data"]["retrieval"]["device_ids"][0];
    let room = f
        .call(json!({"method":"create_room","name":"Kept room"}))
        .await;
    f.call(json!({"method":"set_device_preferences","device_id":device,
        "preferences":{"alias":"Kept name","room_id":room["data"]["id"],"favorite":true}}))
        .await;
    let preferences = serde_json::to_value(f.store.preferences().unwrap()).unwrap();
    f.wire.extra_device.store(true, Ordering::Relaxed);
    f.secrets.uncertain.store(true, Ordering::Relaxed);
    let failed = f.refresh(&account.id.to_string()).await;
    assert_eq!(failed["data"]["account"]["credential_state"], "pending");
    let reference = account.credential_ref.to_string();
    let saved = f.secrets.values.lock().unwrap()[&reference].clone();
    let payload: Value = serde_json::from_slice(&saved).unwrap();
    let mut expected: Vec<_> = payload["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|binding| binding["local"]["device_id"].as_str().unwrap().to_owned())
        .collect();
    expected.sort();
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    f.secrets.uncertain.store(false, Ordering::Relaxed);
    let before = f.calls().len();
    let result = f.call(recover(account.id)).await;
    let mut actual: Vec<String> =
        serde_json::from_value(result["data"]["retrieval"]["device_ids"].clone()).unwrap();
    actual.sort();
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 3);
    assert_eq!(f.secrets.values.lock().unwrap()[&reference], saved);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    assert_eq!(
        &f.calls()[before..],
        [CloudEndpoint::Homes, CloudEndpoint::HomeAppliances]
    );
    let current = serde_json::to_value(f.store.preferences().unwrap()).unwrap();
    for old in preferences.as_array().unwrap() {
        assert!(current.as_array().unwrap().contains(old));
    }
    let reopened = SqliteStore::open(&f.path).unwrap();
    assert_eq!(
        reopened.account(account.id).unwrap().credential_state,
        CredentialState::Stored
    );
    assert_eq!(reopened.devices().unwrap().len(), 3);
}

#[tokio::test(start_paused = true)]
async fn recovery_rejection_and_partial_inventory_leave_credentials_unconfirmed() {
    for (home_code, appliance_code, expected) in [
        (7400, 0, "AUTH_REQUIRED"),
        (0, 40002, "AUTH_REQUIRED"),
        (0, 1306, "MIDEA_INVENTORY_PARTIAL"),
    ] {
        let f = Fixture::new();
        let account = pending_account(&f).await;
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        f.wire.homes_code.store(home_code, Ordering::Relaxed);
        f.wire
            .inventory_code
            .store(appliance_code, Ordering::Relaxed);
        let result = f.call(recover(account.id)).await;
        assert_eq!(result["data"]["retrieval"]["error"]["code"], expected);
        assert_eq!(result["data"]["account"]["credential_state"], "pending");
        assert_eq!(
            f.store.account(account.id).unwrap().credential_state,
            CredentialState::Pending
        );
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        assert_eq!(f.store.devices().unwrap().len(), 2);
    }
}

#[tokio::test(start_paused = true)]
async fn invalid_recovery_material_never_reaches_the_provider_or_resets_identity() {
    for kind in [
        "missing",
        "unavailable",
        "oversized",
        "malformed",
        "account",
        "reference",
        "provider",
        "region",
        "subject",
        "session",
        "bindings",
    ] {
        let f = Fixture::new();
        let account = pending_account(&f).await;
        let reference = account.credential_ref.to_string();
        let original = f.secrets.values.lock().unwrap()[&reference].clone();
        let mut payload: Value = serde_json::from_slice(&original).unwrap();
        let expected = match kind {
            "missing" => {
                f.secrets.values.lock().unwrap().remove(&reference);
                "AUTH_REQUIRED"
            }
            "unavailable" => {
                f.secrets.unavailable.store(true, Ordering::Relaxed);
                "SECRET_STORE_UNAVAILABLE"
            }
            _ => {
                let value = match kind {
                    "oversized" => vec![b' '; MAX_RESPONSE_BYTES + 1],
                    "malformed" => b"invalid-private-json".to_vec(),
                    _ => {
                        match kind {
                            "account" => payload["account_id"] = json!(Uuid::new_v4()),
                            "reference" => payload["credential_ref"] = json!(Uuid::new_v4()),
                            "provider" => payload["provider"] = json!("foreign.provider"),
                            "region" => payload["region"] = json!("US"),
                            "subject" => payload["subject"] = json!(b"foreign-subject".to_vec()),
                            "session" => payload["session"] = json!(b"invalid-session".to_vec()),
                            _ => {
                                let duplicate = payload["bindings"][0].clone();
                                payload["bindings"].as_array_mut().unwrap().push(duplicate);
                            }
                        }
                        serde_json::to_vec(&payload).unwrap()
                    }
                };
                f.secrets.values.lock().unwrap().insert(reference, value);
                "ACCOUNT_SESSION_UNCONFIRMED"
            }
        };
        let before = f.calls();
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        let result = f.call(recover(account.id)).await;
        assert_eq!(
            result["data"]["retrieval"]["error"]["code"], expected,
            "{kind}"
        );
        assert_eq!(f.calls(), before);
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        assert_eq!(
            f.store.account(account.id).unwrap().credential_state,
            CredentialState::Pending
        );
        assert_eq!(f.store.devices().unwrap().len(), 2);
    }
}

#[tokio::test(start_paused = true)]
async fn recovery_confirms_exact_bytes_and_handles_sqlite_failure_without_rewriting_secrets() {
    for changed_bytes in [true, false] {
        let f = Fixture::new();
        let account = pending_account(&f).await;
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        *f.wire.delay.lock().unwrap() = Some(CloudEndpoint::Homes);
        let mut work = Box::pin(f.call_id("recover", recover(account.id)));
        tokio::select! {
            _ = f.wire.started.notified() => (),
            value = &mut work => panic!("Expected an in-flight recovery: {value}"),
        }
        let reference = account.credential_ref.to_string();
        let original = f.secrets.values.lock().unwrap()[&reference].clone();
        let database = rusqlite::Connection::open(f.path.join("homeverb.sqlite3")).unwrap();
        if changed_bytes {
            f.secrets
                .values
                .lock()
                .unwrap()
                .insert(reference.clone(), b"changed-during-validation".to_vec());
        } else {
            database.execute_batch("CREATE TRIGGER refuse_confirmation BEFORE UPDATE OF credential_state ON accounts BEGIN SELECT RAISE(ABORT, 'synthetic'); END").unwrap();
        }
        f.wire.resume.notify_one();
        let result = work.await;
        assert_eq!(
            result["data"]["retrieval"]["error"]["code"],
            if changed_bytes {
                "ACCOUNT_SESSION_UNCONFIRMED"
            } else {
                "STORAGE_ERROR"
            }
        );
        assert_eq!(
            f.store.account(account.id).unwrap().credential_state,
            CredentialState::Pending
        );
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        if changed_bytes {
            f.secrets.values.lock().unwrap().insert(reference, original);
        } else {
            database
                .execute_batch("DROP TRIGGER refuse_confirmation")
                .unwrap();
        }
        *f.wire.delay.lock().unwrap() = None;
        assert_eq!(
            f.call(recover(account.id)).await["data"]["retrieval"]["status"],
            "complete"
        );
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    }
}

#[tokio::test(start_paused = true)]
async fn interrupted_recovery_never_confirms_or_runs_on_restart_and_releases_leases() {
    for stop in ["cancel", "deadline", "background", "close", "drop"] {
        let mut f = Fixture::new();
        let account = pending_account(&f).await;
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        *f.wire.delay.lock().unwrap() = Some(CloudEndpoint::HomeAppliances);
        let mut work = Box::pin(f.call_id("recover", recover(account.id)));
        tokio::select! {
            _ = f.wire.started.notified() => (),
            value = &mut work => panic!("Expected an in-flight recovery: {value}"),
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
        assert_eq!(
            f.vault
                .lease(account.id, &operation())
                .await
                .err()
                .unwrap()
                .code,
            "RESOURCE_BUSY"
        );
        assert_eq!(
            f.call_id("recover", json!({"method":"list_accounts"}))
                .await["error"]["code"],
            "RESOURCE_BUSY"
        );
        if stop == "drop" {
            drop(work);
        } else {
            match stop {
                "cancel" => assert!(f.session.cancel("recover")),
                "background" => f.session.set_foreground(false),
                "close" => f.session.close(),
                _ => tokio::time::advance(std::time::Duration::from_secs(2)).await,
            }
            let result = work.await;
            assert_eq!(result["data"]["account"]["credential_state"], "pending");
            assert_eq!(
                result["data"]["retrieval"]["error"]["code"],
                if stop == "deadline" {
                    "DEADLINE_EXCEEDED"
                } else {
                    "CANCELLED"
                }
            );
        }
        drop(f.vault.lock_registration(&operation()).await.unwrap());
        drop(f.vault.lease(account.id, &operation()).await.unwrap());
        let calls = f.calls().len();
        let reopened = SqliteStore::open(&f.path).unwrap();
        assert_eq!(
            reopened.account(account.id).unwrap().credential_state,
            CredentialState::Pending
        );
        f.reopen_with_power_grants(&[]);
        assert_eq!(f.calls().len(), calls);
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        *f.wire.delay.lock().unwrap() = None;
        assert_eq!(
            f.call(recover(account.id)).await["data"]["retrieval"]["status"],
            "complete"
        );
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    }
}

#[tokio::test(start_paused = true)]
async fn recovery_admission_refuses_wrong_account_region_busy_and_removal_intent() {
    let f = Fixture::new();
    let account = pending_account(&f).await;
    let calls = f.calls();
    let reads = f.secrets.reads.load(Ordering::Relaxed);
    assert_eq!(
        f.call(recover(Uuid::new_v4())).await["error"]["code"],
        "ACCOUNT_NOT_FOUND"
    );
    for (provider, region) in [("foreign.provider", "CN"), ("midea.meiju", "US")] {
        let foreign = f
            .vault
            .prepare(provider, region, "Foreign", &operation())
            .await
            .unwrap();
        let id = foreign.record().id;
        drop(foreign);
        assert_eq!(
            f.call(recover(id)).await["error"]["code"],
            "ACCOUNT_NOT_FOUND"
        );
    }
    let lease = f.vault.lease(account.id, &operation()).await.unwrap();
    assert_eq!(
        f.call(recover(account.id)).await["error"]["code"],
        "RESOURCE_BUSY"
    );
    drop(lease);
    let devices: Vec<_> = f
        .store
        .devices()
        .unwrap()
        .into_iter()
        .map(|descriptor| descriptor.device.id)
        .collect();
    f.store.begin_account_removal(account.id, &devices).unwrap();
    assert_eq!(
        f.call(recover(account.id)).await["error"]["code"],
        "ACCOUNT_REMOVAL_PENDING"
    );
    assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
    assert_eq!(f.calls(), calls);
    assert_eq!(
        f.store.account(account.id).unwrap().credential_state,
        CredentialState::Removing
    );
}

#[tokio::test(start_paused = true)]
async fn recovery_reports_new_mapping_uncertainty_and_publication_failure_separately() {
    let f = Fixture::new();
    f.secrets.uncertain.store(true, Ordering::Relaxed);
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let uncertain = f.call(recover(account.id)).await;
    assert_eq!(uncertain["data"]["account"]["credential_state"], "pending");
    assert_eq!(
        uncertain["data"]["retrieval"]["error"]["code"],
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert!(f.store.devices().unwrap().is_empty());
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    f.secrets.uncertain.store(false, Ordering::Relaxed);
    let db = rusqlite::Connection::open(f.path.join("homeverb.sqlite3")).unwrap();
    db.execute_batch("CREATE TRIGGER refuse_publication BEFORE INSERT ON devices BEGIN SELECT RAISE(ABORT, 'synthetic'); END").unwrap();
    let result = f.call(recover(account.id)).await;
    assert_eq!(result["data"]["account"]["credential_state"], "stored");
    assert_eq!(
        result["data"]["retrieval"]["error"]["code"],
        "STORAGE_ERROR"
    );
    assert!(f.store.devices().unwrap().is_empty());
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    db.execute_batch("DROP TRIGGER refuse_publication").unwrap();
    let retry = f.call(recover(account.id)).await;
    assert_eq!(retry["data"]["retrieval"]["status"], "complete");
    assert_eq!(f.store.devices().unwrap().len(), 2);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
}
