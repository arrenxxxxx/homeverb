use super::*;
use crate::{
    AccountLoginChallenge, VerificationAccountLogin, VerificationAnswer, VerificationMethod,
};

fn ticket() -> VerificationAnswer {
    VerificationAnswer::Ticket {
        method: VerificationMethod::Phone,
        ticket: SecretBytes::new(b"PRIVATE-VERIFICATION-TICKET".to_vec()),
    }
}
async fn pending(f: &Fixture, existing: Option<Uuid>) -> crate::PendingVerificationConnection {
    let VerificationAccountLogin::Pending(pending) = f
        .adapter
        .begin_verification_connection(credentials(), existing, "Home", &operation())
        .await
        .unwrap()
    else {
        panic!("Expected pending verification");
    };
    pending
}

#[tokio::test]
async fn account_verification_waits_without_leases_then_preserves_identity_and_metadata() {
    for captcha in [false, true] {
        let f = Fixture::new();
        let original = f.connect().await;
        f.publish(&original).await;
        let fan = f.fan();
        let puts = f.secrets.puts.load(Ordering::SeqCst);
        let inventories = f.cloud.count(CloudEndpoint::Devices);
        if captcha {
            f.cloud.captcha.store(1, Ordering::SeqCst);
        }
        let mut p = pending(&f, Some(original.account.id)).await;
        let expiry = p.expires_at();
        if captcha {
            assert!(matches!(
                p.challenge(),
                AccountLoginChallenge::Captcha { .. }
            ));
            let VerificationAccountLogin::Pending(next) = f
                .adapter
                .continue_verification_connection(
                    p,
                    VerificationAnswer::Captcha(SecretBytes::new(b"correct".to_vec())),
                    &operation(),
                )
                .await
                .unwrap()
            else {
                panic!("Expected verification after captcha");
            };
            p = next;
            assert_eq!(p.expires_at(), expiry);
        }
        assert_eq!(
            format!("{p:?}"),
            "PendingVerificationConnection([REDACTED])"
        );
        let AccountLoginChallenge::Verification { methods, .. } = p.challenge() else {
            panic!("Expected provider verification");
        };
        assert_eq!(
            methods,
            [VerificationMethod::Phone, VerificationMethod::Email]
        );
        assert_eq!(f.secrets.puts.load(Ordering::SeqCst), puts);
        assert_eq!(f.cloud.count(CloudEndpoint::Devices), inventories);
        drop(f.vault.lock_registration(&operation()).await.unwrap());
        drop(
            f.vault
                .lease(original.account.id, &operation())
                .await
                .unwrap(),
        );
        let VerificationAccountLogin::Connected(report) = f
            .adapter
            .continue_verification_connection(p, ticket(), &operation())
            .await
            .unwrap()
        else {
            panic!("Expected connection");
        };
        assert_eq!(report.account.id, original.account.id);
        f.publish(&report).await;
        assert_eq!(f.fan().id, fan.id);
        for path in ["homeverb.sqlite3", "homeverb.sqlite3-wal"] {
            if path.ends_with("-wal") && !f.directory.0.join(path).exists() {
                continue;
            }
            {
                let bytes = std::fs::read(f.directory.0.join(path)).unwrap();
                for marker in [
                    b"PRIVATE-VERIFICATION-TICKET".as_slice(),
                    b"PRIVATE-VERIFICATION-PASS",
                    b"PRIVATE-VERIFICATION-TOKEN",
                ] {
                    assert!(!bytes.windows(marker.len()).any(|v| v == marker));
                }
            }
        }
    }
}

#[tokio::test]
async fn changed_removed_or_foreign_account_and_wrong_answer_fail_before_ticket_send() {
    for mode in 0..5 {
        let f = Fixture::new();
        let original = f.connect().await;
        let p = pending(
            &f,
            if mode == 3 {
                None
            } else {
                Some(original.account.id)
            },
        )
        .await;
        let calls = f.cloud.verification_calls.lock().unwrap().len();
        if mode == 1 {
            let mut lease = f
                .vault
                .lease(original.account.id, &operation())
                .await
                .unwrap();
            let bytes = lease.inspect_session(&operation()).await.unwrap().unwrap();
            let mut value: Value = serde_json::from_slice(bytes.expose()).unwrap();
            value["session"]["token"] = json!("CHANGED-FICTIONAL-TOKEN");
            lease
                .store_session(
                    &SecretBytes::new(serde_json::to_vec(&value).unwrap()),
                    &operation(),
                )
                .await
                .unwrap();
        }
        if mode == 2 || mode == 3 {
            f.adapter
                .remove_account(original.account.id, &[], &operation())
                .await
                .unwrap();
        }
        let result = if mode == 0 {
            MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
                .continue_verification_connection(p, ticket(), &operation())
                .await
        } else {
            let answer = if mode == 4 {
                VerificationAnswer::Captcha(SecretBytes::new(b"correct".to_vec()))
            } else {
                ticket()
            };
            f.adapter
                .continue_verification_connection(p, answer, &operation())
                .await
        };
        assert!(result.is_err());
        assert_eq!(f.cloud.verification_calls.lock().unwrap().len(), calls);
    }
}

#[tokio::test]
async fn ticket_cannot_replace_foreign_subject_or_write_when_cancelled() {
    for cancelled in [false, true] {
        let f = Fixture::new();
        let original = f.connect().await;
        let p = pending(&f, Some(original.account.id)).await;
        let puts = f.secrets.puts.load(Ordering::SeqCst);
        let op = operation();
        if cancelled {
            op.cancellation.cancel();
        } else {
            *f.cloud.subject.lock().unwrap() = "999002".into();
        }
        let result = f
            .adapter
            .continue_verification_connection(p, ticket(), &op)
            .await
            .unwrap_err();
        assert_eq!(
            result.code,
            if cancelled {
                "CANCELLED"
            } else {
                "ACCOUNT_IDENTITY_CONFLICT"
            }
        );
        assert_eq!(f.secrets.puts.load(Ordering::SeqCst), puts);
    }
}

#[tokio::test]
async fn account_changed_during_ticket_request_is_rechecked_before_persistence() {
    let f = Fixture::new();
    let original = f.connect().await;
    let p = pending(&f, Some(original.account.id)).await;
    f.cloud.verification_gate.store(1, Ordering::SeqCst);
    let started = f.cloud.verification_started.notified();
    let operation = operation();
    let work = f
        .adapter
        .continue_verification_connection(p, ticket(), &operation);
    tokio::pin!(work);
    tokio::select! { _ = started => (), result = &mut work => panic!("Unexpected result: {result:?}") }
    let mut lease = f
        .vault
        .lease(original.account.id, &operation)
        .await
        .unwrap();
    let bytes = lease.inspect_session(&operation).await.unwrap().unwrap();
    let mut value: Value = serde_json::from_slice(bytes.expose()).unwrap();
    value["session"]["token"] = json!("CONCURRENT-FICTIONAL-TOKEN");
    lease
        .store_session(
            &SecretBytes::new(serde_json::to_vec(&value).unwrap()),
            &operation,
        )
        .await
        .unwrap();
    drop(lease);
    let puts = f.secrets.puts.load(Ordering::SeqCst);
    f.cloud.verification_resume.notify_one();
    assert_eq!(work.await.unwrap_err().code, "ACCOUNT_SESSION_UNCONFIRMED");
    assert_eq!(f.secrets.puts.load(Ordering::SeqCst), puts);
}

#[tokio::test]
async fn code_request_rechecks_concurrent_account_changes_without_saving_or_returning_a_prompt() {
    let f = Fixture::new();
    let original = f.connect().await;
    let p = pending(&f, Some(original.account.id)).await;
    f.cloud.verification_gate.store(1, Ordering::SeqCst);
    let started = f.cloud.verification_started.notified();
    let operation = operation();
    let work = f
        .adapter
        .request_verification_code(p, VerificationMethod::Phone, &operation);
    tokio::pin!(work);
    tokio::select! { _ = started => (), result = &mut work => panic!("Unexpected result: {result:?}") }
    let mut lease = f
        .vault
        .lease(original.account.id, &operation)
        .await
        .unwrap();
    let bytes = lease.inspect_session(&operation).await.unwrap().unwrap();
    let mut value: Value = serde_json::from_slice(bytes.expose()).unwrap();
    value["session"]["token"] = json!("CONCURRENT-FICTIONAL-TOKEN");
    lease
        .store_session(
            &SecretBytes::new(serde_json::to_vec(&value).unwrap()),
            &operation,
        )
        .await
        .unwrap();
    drop(lease);
    let puts = f.secrets.puts.load(Ordering::SeqCst);
    f.cloud.verification_resume.notify_one();
    assert_eq!(work.await.unwrap_err().code, "ACCOUNT_SESSION_UNCONFIRMED");
    assert_eq!(f.secrets.puts.load(Ordering::SeqCst), puts);
    assert_eq!(
        f.cloud
            .verification_calls
            .lock()
            .unwrap()
            .iter()
            .filter(|e| **e == crate::VerificationEndpoint::SendPhone)
            .count(),
        1
    );
}
