use super::*;

async fn pending(f: &Fixture) -> AccountRecord {
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    f.store
        .set_credential_state(account.id, CredentialState::Pending)
        .unwrap();
    account
}

fn saved(f: &Fixture, account: &AccountRecord) -> Vec<u8> {
    f.secrets.values.lock().unwrap()[&account.credential_ref.to_string()].clone()
}

#[tokio::test(start_paused = true)]
async fn explicit_pending_reauthorization_preserves_bindings_preferences_and_scoped_grants() {
    let mut f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let device = f
        .store
        .devices()
        .unwrap()
        .into_iter()
        .find(|d| d.device.model == "KFR-35G/WF2")
        .unwrap()
        .device
        .id;
    let room = f
        .call(json!({"method":"create_room","name":"Kept room"}))
        .await;
    f.call(json!({"method":"set_device_preferences","device_id":device,
        "preferences":{"alias":"Kept name","room_id":room["data"]["id"],"favorite":true}}))
        .await;
    let preferences = serde_json::to_value(f.store.preferences().unwrap()).unwrap();
    f.reopen_with_power_grants(&[MeijuPowerGrant {
        device_id: device.clone(),
        state: false,
    }]);
    f.wire.extra_device.store(true, Ordering::Relaxed);
    f.secrets.uncertain.store(true, Ordering::Relaxed);
    assert_eq!(
        f.refresh(&account.id.to_string()).await["data"]["account"]["credential_state"],
        "pending"
    );
    f.secrets.uncertain.store(false, Ordering::Relaxed);
    f.wire.homes_code.store(7400, Ordering::Relaxed);
    assert_eq!(
        f.call(json!({"method":"recover_account","account_id":account.id}))
            .await["data"]["retrieval"]["error"]["code"],
        "AUTH_REQUIRED"
    );
    let before: Value = serde_json::from_slice(&saved(&f, &account)).unwrap();
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    let calls = f.calls().len();
    *f.wire.token.lock().unwrap() = "SYNTHETIC-REPLACEMENT-TOKEN".into();
    f.wire.homes_code.store(0, Ordering::Relaxed);
    let result = f.connect(Some(account.id)).await;
    assert_eq!(result["data"]["account"]["id"], account.id.to_string());
    assert_eq!(result["data"]["account"]["credential_state"], "stored");
    assert_eq!(result["data"]["authentication"], "succeeded");
    assert_eq!(result["data"]["retrieval"]["status"], "complete");
    assert_eq!(
        &f.calls()[calls..],
        [
            CloudEndpoint::LoginId,
            CloudEndpoint::Login,
            CloudEndpoint::Homes,
            CloudEndpoint::HomeAppliances
        ]
    );
    let after: Value = serde_json::from_slice(&saved(&f, &account)).unwrap();
    assert_ne!(before["session"], after["session"]);
    for field in [
        "account_id",
        "credential_ref",
        "provider",
        "region",
        "subject",
        "bindings",
    ] {
        assert_eq!(before[field], after[field], "{field}");
    }
    let current = serde_json::to_value(f.store.preferences().unwrap()).unwrap();
    for old in preferences.as_array().unwrap() {
        assert!(current.as_array().unwrap().contains(old));
    }
    assert_eq!(f.store.devices().unwrap().len(), 3);
    assert_eq!(f.store.accounts().unwrap().len(), 1);
    assert_eq!(f.store.account(account.id).unwrap().label, account.label);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts + 1);
    assert_eq!(f.secrets.deletes.load(Ordering::Relaxed), 0);
    assert!(
        f.call(json!({"method":"plan","command":power_command(&device,false)}))
            .await["error"]
            .is_null()
    );
    assert_eq!(
        f.call(json!({"method":"plan","command":power_command(&device,true)}))
            .await["error"]["code"],
        "POLICY_DENIED"
    );
    assert!(!f.calls().contains(&CloudEndpoint::ApplianceControl));
    let reopened = SqliteStore::open(&f.path).unwrap();
    assert_eq!(
        reopened.account(account.id).unwrap().credential_state,
        CredentialState::Stored
    );
    for marker in [
        "SYNTHETIC",
        "credential_ref",
        "owner@example.test",
        "88000000000000",
    ] {
        assert!(!result.to_string().contains(marker));
    }
}

#[tokio::test(start_paused = true)]
async fn pending_reauthorization_rejects_wrong_identity_and_login_failure_without_mutation() {
    for wrong_subject in [true, false] {
        let f = Fixture::new();
        let account = pending(&f).await;
        let original = saved(&f, &account);
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        let calls = f.calls().len();
        if wrong_subject {
            *f.wire.subject.lock().unwrap() = "different-subject".into();
        } else {
            f.wire.login_code.store(7400, Ordering::Relaxed);
        }
        let result = f.connect(Some(account.id)).await;
        assert_eq!(
            result["error"]["code"],
            if wrong_subject {
                "ACCOUNT_IDENTITY_CONFLICT"
            } else {
                "AUTH_REQUIRED"
            }
        );
        assert_eq!(
            &f.calls()[calls..],
            [CloudEndpoint::LoginId, CloudEndpoint::Login]
        );
        assert_eq!(saved(&f, &account), original);
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        assert_eq!(
            f.store.account(account.id).unwrap().credential_state,
            CredentialState::Pending
        );
        assert_eq!(f.store.devices().unwrap().len(), 2);
    }
}

#[tokio::test(start_paused = true)]
async fn pending_reauthorization_requires_explicit_selection_and_intact_identity_before_login() {
    for kind in [
        "unselected",
        "missing",
        "malformed",
        "session",
        "subject",
        "account",
        "reference",
        "provider",
        "region",
        "bindings",
    ] {
        let f = Fixture::new();
        let account = pending(&f).await;
        let reference = account.credential_ref.to_string();
        let mut payload: Value = serde_json::from_slice(&saved(&f, &account)).unwrap();
        if kind == "missing" {
            f.secrets.values.lock().unwrap().remove(&reference);
        } else if kind != "unselected" {
            match kind {
                "session" => payload["session"] = json!(b"invalid-session".to_vec()),
                "subject" => payload["subject"] = json!(b"foreign-subject".to_vec()),
                "account" => payload["account_id"] = json!(Uuid::new_v4()),
                "reference" => payload["credential_ref"] = json!(Uuid::new_v4()),
                "provider" => payload["provider"] = json!("foreign.provider"),
                "region" => payload["region"] = json!("US"),
                "bindings" => {
                    let duplicate = payload["bindings"][0].clone();
                    payload["bindings"].as_array_mut().unwrap().push(duplicate);
                }
                _ => (),
            }
            let value = if kind == "malformed" {
                b"invalid-json".to_vec()
            } else {
                serde_json::to_vec(&payload).unwrap()
            };
            f.secrets.values.lock().unwrap().insert(reference, value);
        }
        let original = f.secrets.values.lock().unwrap().clone();
        let calls = f.calls();
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        let result = f
            .connect(if kind == "unselected" {
                None
            } else {
                Some(account.id)
            })
            .await;
        assert_eq!(
            result["error"]["code"], "ACCOUNT_SESSION_UNCONFIRMED",
            "{kind}"
        );
        assert_eq!(f.calls(), calls);
        assert_eq!(*f.secrets.values.lock().unwrap(), original);
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        assert_eq!(
            f.store.account(account.id).unwrap().credential_state,
            CredentialState::Pending
        );
    }
}

#[tokio::test(start_paused = true)]
async fn reauthorization_refuses_changed_protected_bytes_after_login() {
    let f = Fixture::new();
    let account = pending(&f).await;
    let puts = f.secrets.puts.load(Ordering::Relaxed);
    *f.wire.delay.lock().unwrap() = Some(CloudEndpoint::Login);
    let mut work = Box::pin(f.connect(Some(account.id)));
    tokio::select! {
        _ = f.wire.started.notified() => (),
        value = &mut work => panic!("Expected an in-flight login: {value}"),
    }
    let mut changed = saved(&f, &account);
    changed.push(b' ');
    f.secrets
        .values
        .lock()
        .unwrap()
        .insert(account.credential_ref.to_string(), changed.clone());
    f.wire.resume.notify_one();
    assert_eq!(work.await["error"]["code"], "ACCOUNT_SESSION_UNCONFIRMED");
    assert_eq!(saved(&f, &account), changed);
    assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
    assert_eq!(
        f.store.account(account.id).unwrap().credential_state,
        CredentialState::Pending
    );
}

#[tokio::test(start_paused = true)]
async fn interrupted_reauthorization_persistence_stays_pending_and_can_recover() {
    for metadata_failure in [false, true] {
        let f = Fixture::new();
        let account = pending(&f).await;
        let original: Value = serde_json::from_slice(&saved(&f, &account)).unwrap();
        *f.wire.token.lock().unwrap() = "SYNTHETIC-NEW-TOKEN".into();
        let database = rusqlite::Connection::open(f.path.join("homeverb.sqlite3")).unwrap();
        if metadata_failure {
            database.execute_batch("CREATE TRIGGER refuse_confirmation BEFORE UPDATE OF credential_state ON accounts WHEN NEW.credential_state='stored' BEGIN SELECT RAISE(ABORT, 'synthetic'); END").unwrap();
        } else {
            f.secrets.uncertain.store(true, Ordering::Relaxed);
        }
        let result = f.connect(Some(account.id)).await;
        assert_eq!(
            result["error"]["code"],
            if metadata_failure {
                "STORAGE_ERROR"
            } else {
                "SECRET_STORE_WRITE_UNKNOWN"
            }
        );
        assert_eq!(
            f.store.account(account.id).unwrap().credential_state,
            CredentialState::Pending
        );
        let updated: Value = serde_json::from_slice(&saved(&f, &account)).unwrap();
        assert_ne!(updated["session"], original["session"]);
        assert_eq!(updated["bindings"], original["bindings"]);
        if metadata_failure {
            database
                .execute_batch("DROP TRIGGER refuse_confirmation")
                .unwrap();
        }
        f.secrets.uncertain.store(false, Ordering::Relaxed);
        let puts = f.secrets.puts.load(Ordering::Relaxed);
        let calls = f.calls().len();
        assert_eq!(
            f.call(json!({"method":"recover_account","account_id":account.id}))
                .await["data"]["retrieval"]["status"],
            "complete"
        );
        assert_eq!(f.secrets.puts.load(Ordering::Relaxed), puts);
        assert_eq!(
            &f.calls()[calls..],
            [CloudEndpoint::Homes, CloudEndpoint::HomeAppliances]
        );
    }
}

#[tokio::test(start_paused = true)]
async fn reauthorization_lifecycle_preserves_last_persisted_state_and_releases_leases() {
    for endpoint in [CloudEndpoint::Login, CloudEndpoint::HomeAppliances] {
        for stop in ["cancel", "deadline", "background", "close", "drop"] {
            let mut f = Fixture::new();
            let account = pending(&f).await;
            let original = saved(&f, &account);
            *f.wire.token.lock().unwrap() = "SYNTHETIC-NEW-TOKEN".into();
            *f.wire.delay.lock().unwrap() = Some(endpoint);
            let mut work = Box::pin(f.connect(Some(account.id)));
            tokio::select! {
                _ = f.wire.started.notified() => (),
                value = &mut work => panic!("Expected an in-flight reauthorization: {value}"),
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
                    "cancel" => assert!(f.session.cancel("connect")),
                    "background" => f.session.set_foreground(false),
                    "close" => f.session.close(),
                    _ => tokio::time::advance(std::time::Duration::from_secs(2)).await,
                }
                let result = work.await;
                let error = if endpoint == CloudEndpoint::Login {
                    &result["error"]
                } else {
                    &result["data"]["retrieval"]["error"]
                };
                assert_eq!(
                    error["code"],
                    if stop == "deadline" {
                        "DEADLINE_EXCEEDED"
                    } else {
                        "CANCELLED"
                    }
                );
            }
            assert_eq!(
                f.store.account(account.id).unwrap().credential_state,
                if endpoint == CloudEndpoint::Login {
                    CredentialState::Pending
                } else {
                    CredentialState::Stored
                }
            );
            assert_eq!(
                saved(&f, &account) == original,
                endpoint == CloudEndpoint::Login
            );
            drop(f.vault.lock_registration(&operation()).await.unwrap());
            drop(f.vault.lease(account.id, &operation()).await.unwrap());
            let calls = f.calls().len();
            f.reopen_with_power_grants(&[]);
            assert_eq!(f.calls().len(), calls);
            assert_eq!(f.store.devices().unwrap().len(), 2);
        }
    }
}

#[tokio::test(start_paused = true)]
async fn reauthorized_pending_account_reports_partial_inventory_without_losing_cards() {
    let f = Fixture::new();
    let account = pending(&f).await;
    let cards = f.call(json!({"method":"list_device_cards"})).await;
    f.wire.inventory_code.store(1306, Ordering::Relaxed);
    let result = f.connect(Some(account.id)).await;
    assert_eq!(result["data"]["authentication"], "succeeded");
    assert_eq!(result["data"]["account"]["credential_state"], "stored");
    assert_eq!(result["data"]["retrieval"]["status"], "partial");
    assert_eq!(f.call(json!({"method":"list_device_cards"})).await, cards);
}
