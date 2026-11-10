use super::*;

async fn connected() -> (Fixture, AccountRecord, Vec<DeviceBinding>) {
    let f = Fixture::new();
    let report = connect(
        &f.vault,
        &LoginTransport::complete("PRIVATE-LOGIN-ID"),
        login_credentials(),
        None,
        "Home",
        &operation(),
    )
    .await
    .unwrap();
    let bindings = report.retrieval.unwrap().bindings;
    (f, report.account, bindings)
}

fn edit_login(f: &Fixture, record: &AccountRecord, edit: impl FnOnce(&mut Value)) {
    let mut payload: Value = serde_json::from_slice(&f.bytes(record.credential_ref)).unwrap();
    edit(&mut payload);
    f.set_bytes(record.credential_ref, serde_json::to_vec(&payload).unwrap());
}

fn expired_then_login(subject: &str) -> LoginTransport {
    let cloud = LoginTransport::complete(subject);
    cloud
        .replies
        .lock()
        .unwrap()
        .push_front(json!({"code":40002}));
    cloud
}

#[tokio::test]
async fn protected_login_survives_restart_and_recovers_only_the_same_account() {
    let (f, record, bindings) = connected().await;
    edit_login(&f, &record, |p| {
        assert_eq!(p["version"], 4);
        assert_eq!(p["saved_login"]["password"], json!(b"PRIVATE-PASSWORD"));
        p["saved_login"]["last_attempt_ms"] = json!(0);
    });
    let mut account = f.reopen(record.id).await.unwrap();
    let cloud = expired_then_login("PRIVATE-LOGIN-ID");
    let result = account.retrieve(&cloud, &operation()).await.unwrap();
    assert!(result.inventory.is_complete());
    assert_eq!(result.bindings, bindings);
    assert_eq!(account.record().id, record.id);
    assert_eq!(cloud.calls.load(Ordering::Relaxed), 5);
    drop(account);
    let payload: Value = serde_json::from_slice(&f.bytes(record.credential_ref)).unwrap();
    assert_eq!(payload["saved_login"]["blocked"], false);
    assert!(payload["saved_login"]["last_attempt_ms"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn failed_login_or_identity_conflict_blocks_future_automatic_attempts() {
    for wrong_identity in [false, true] {
        let (f, record, _) = connected().await;
        edit_login(&f, &record, |p| {
            p["saved_login"]["last_attempt_ms"] = json!(0)
        });
        let before: Value = serde_json::from_slice(&f.bytes(record.credential_ref)).unwrap();
        let cloud = expired_then_login(if wrong_identity {
            "OTHER-PRIVATE-ID"
        } else {
            "PRIVATE-LOGIN-ID"
        });
        if !wrong_identity {
            cloud.replies.lock().unwrap()[1] = json!({"code":3101});
        }
        let mut account = f.reopen(record.id).await.unwrap();
        let error = account.retrieve(&cloud, &operation()).await.unwrap_err();
        assert_eq!(
            error.code,
            if wrong_identity {
                "ACCOUNT_IDENTITY_CONFLICT"
            } else {
                "MIDEA_REQUEST_REJECTED"
            }
        );
        drop(account);
        let after: Value = serde_json::from_slice(&f.bytes(record.credential_ref)).unwrap();
        assert_eq!(after["saved_login"]["blocked"], true);
        assert_eq!(after["session"], before["session"]);
        assert_eq!(after["bindings"], before["bindings"]);
        assert_eq!(after["subject"], before["subject"]);
        let denied = LoginTransport::new("unused", json!({}));
        *denied.replies.lock().unwrap() = vec![json!({"code":40002})].into();
        let mut reopened = f.reopen(record.id).await.unwrap();
        assert_eq!(
            reopened
                .retrieve(&denied, &operation())
                .await
                .unwrap_err()
                .code,
            "AUTH_REQUIRED"
        );
        assert_eq!(denied.calls.load(Ordering::Relaxed), 1);
    }
}

#[tokio::test]
async fn missing_credentials_cooldown_and_clock_rollback_do_not_login() {
    for state in ["legacy", "cooldown", "rollback"] {
        let (f, record, _) = connected().await;
        edit_login(&f, &record, |p| match state {
            "legacy" => {
                p.as_object_mut().unwrap().remove("saved_login");
                p["version"] = json!(3);
            }
            "rollback" => p["saved_login"]["last_attempt_ms"] = json!(i64::MAX),
            _ => (),
        });
        let before = f.bytes(record.credential_ref);
        let cloud = expired_then_login("PRIVATE-LOGIN-ID");
        let mut account = f.reopen(record.id).await.unwrap();
        assert_eq!(
            account
                .retrieve(&cloud, &operation())
                .await
                .unwrap_err()
                .code,
            "AUTH_REQUIRED"
        );
        assert_eq!(cloud.calls.load(Ordering::Relaxed), 1);
        assert_eq!(f.bytes(record.credential_ref), before);
    }
}

#[tokio::test]
async fn unavailable_or_uncertain_storage_prevents_automatic_login() {
    for mode in [1, 2, 3] {
        let (f, record, _) = connected().await;
        edit_login(&f, &record, |p| {
            p["saved_login"]["last_attempt_ms"] = json!(0)
        });
        let mut account = f.reopen(record.id).await.unwrap();
        f.secrets.mode.store(mode, Ordering::Relaxed);
        let cloud = expired_then_login("PRIVATE-LOGIN-ID");
        assert!(account.retrieve(&cloud, &operation()).await.is_err());
        assert_eq!(cloud.calls.load(Ordering::Relaxed), 1);
        assert_eq!(account.record().credential_state, CredentialState::Pending);
    }
}

#[tokio::test]
async fn cancellation_during_login_keeps_the_durable_attempt_blocked() {
    struct CancelLogin;
    #[async_trait]
    impl CloudTransport for CancelLogin {
        async fn send(
            &self,
            request: &SignedRequest,
            operation: &Operation,
        ) -> Result<SecretBytes> {
            if request.endpoint() == CloudEndpoint::LoginId {
                operation.cancellation.cancel();
                return Err(Error::new("CANCELLED", "Synthetic cancellation."));
            }
            Ok(SecretBytes::new(
                serde_json::to_vec(&json!({"code":40002})).unwrap(),
            ))
        }
    }
    let (f, record, _) = connected().await;
    edit_login(&f, &record, |p| {
        p["saved_login"]["last_attempt_ms"] = json!(0)
    });
    let mut account = f.reopen(record.id).await.unwrap();
    assert_eq!(
        account
            .retrieve(&CancelLogin, &operation())
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let payload: Value = serde_json::from_slice(&f.bytes(record.credential_ref)).unwrap();
    assert_eq!(payload["saved_login"]["blocked"], true);
}
