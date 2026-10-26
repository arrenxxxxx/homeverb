use super::*;
use homeverb_host::{NativeVerificationLogin as Login, NativeVerificationPrompt as Prompt};
use homeverb_mijia::{VerificationAnswer as Answer, VerificationMethod as Method};

fn ticket() -> Answer {
    Answer::Ticket {
        method: Method::Email,
        ticket: secret(b"PRIVATE-FICTIONAL-TICKET"),
    }
}
async fn begin(f: &Fixture) -> Login {
    f.session
        .begin_account_verification(
            begin_metadata(),
            secret(b"fiction@example.invalid"),
            secret(b"fictional-password"),
        )
        .await
        .unwrap()
}
fn id(result: Login) -> u64 {
    assert_eq!(format!("{result:?}"), "NativeVerificationLogin([REDACTED])");
    let Login::Pending {
        challenge_id,
        prompt,
        expires_in_ms,
    } = result
    else {
        panic!("Expected challenge");
    };
    assert!((1..=180000).contains(&expires_in_ms));
    assert_eq!(
        format!("{prompt:?}"),
        "NativeVerificationPrompt([REDACTED])"
    );
    challenge_id
}

#[tokio::test]
async fn verification_and_captcha_transition_publish_only_sanitized_connected_cards() {
    for captcha in [false, true] {
        let f = Fixture::new(true);
        f.cloud.captcha.store(captcha, Ordering::SeqCst);
        let mut pending = begin(&f).await;
        if captcha {
            assert!(matches!(
                &pending,
                Login::Pending {
                    prompt: Prompt::Captcha { .. },
                    ..
                }
            ));
            let old = id(pending);
            pending = f
                .session
                .continue_account_verification(
                    answer_metadata(old),
                    Answer::Captcha(secret(b"correct")),
                )
                .await
                .unwrap();
            assert!(!f.session.discard_account_verification(old));
        }
        let Login::Pending {
            prompt:
                Prompt::Verification {
                    ref provider_page,
                    ref methods,
                },
            ..
        } = pending
        else {
            panic!("Expected provider page");
        };
        assert!(
            provider_page
                .expose()
                .starts_with(b"https://account.xiaomi.com/fe/service/identity/authStart?")
        );
        assert_eq!(methods, &[Method::Phone, Method::Email]);
        assert!(
            f.call(json!({"method":"list_accounts"})).await["data"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let challenge = id(pending);
        let Login::Connected(report) = f
            .session
            .continue_account_verification(answer_metadata(challenge), ticket())
            .await
            .unwrap()
        else {
            panic!("Expected connection");
        };
        let report = serde_json::to_value(report).unwrap();
        assert_eq!(report["retrieval"]["status"], "partial");
        assert_eq!(
            report["retrieval"]["device_ids"].as_array().unwrap().len(),
            2
        );
        assert!(!report.to_string().contains("PRIVATE"));
        assert!(
            f.session
                .continue_account_verification(answer_metadata(challenge), ticket())
                .await
                .is_err()
        );
    }
}

#[tokio::test]
async fn verification_registry_is_session_owned_and_cleared_by_all_lifecycle_paths() {
    for mode in 0..4 {
        let f = Fixture::new(false);
        let challenge = id(begin(&f).await);
        let other = Fixture::new(false);
        assert!(
            other
                .session
                .continue_account_verification(answer_metadata(challenge), ticket())
                .await
                .is_err()
        );
        let calls = f.cloud.calls.load(Ordering::SeqCst);
        match mode {
            0 => {
                f.session.set_foreground(false);
                f.session.set_foreground(true);
            }
            1 => {
                f.session.cancel("begin-captcha");
            }
            2 => f.session.close(),
            _ => assert!(f.session.discard_account_verification(challenge)),
        }
        assert!(
            f.session
                .continue_account_verification(answer_metadata(challenge), ticket())
                .await
                .is_err()
        );
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    }
}

#[tokio::test]
async fn legacy_and_verification_challenges_share_capacity_but_not_answer_endpoints() {
    let f = Fixture::new(false);
    let first = id(begin(&f).await);
    let second = id(begin(&f).await);
    f.cloud.captcha.store(true, Ordering::SeqCst);
    let old = pending_login(&f).await;
    pending_login(&f).await;
    let calls = f.cloud.calls.load(Ordering::SeqCst);
    assert_eq!(
        f.session
            .begin_account_verification(
                begin_metadata(),
                secret(b"fiction@example.invalid"),
                secret(b"fictional-password")
            )
            .await
            .unwrap_err()
            .code,
        "RESOURCE_BUSY"
    );
    assert_eq!(
        f.session
            .begin_account_login(
                begin_metadata(),
                secret(b"fiction@example.invalid"),
                secret(b"fictional-password")
            )
            .await
            .unwrap_err()
            .code,
        "RESOURCE_BUSY"
    );
    assert!(
        f.session
            .continue_account_login(answer_metadata(first), secret(b"correct"))
            .await
            .is_err()
    );
    assert!(
        f.session
            .continue_account_verification(answer_metadata(old), ticket())
            .await
            .is_err()
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    assert!(f.session.discard_account_verification(second));
    pending_login(&f).await;
    assert!(f.session.discard_account_verification(first));
    assert!(f.session.discard_account_login(old));
}

#[tokio::test(start_paused = true)]
async fn expired_wrong_kind_or_invalid_metadata_consumes_without_sending() {
    for mode in 0..3 {
        let f = Fixture::new(false);
        let challenge = id(begin(&f).await);
        let calls = f.cloud.calls.load(Ordering::SeqCst);
        let mut metadata = answer_metadata(challenge);
        if mode == 0 {
            tokio::time::advance(std::time::Duration::from_secs(181)).await;
        }
        if mode == 1 {
            metadata.schema_version = "invalid".into();
        }
        let answer = if mode == 2 {
            Answer::Captcha(secret(b"correct"))
        } else {
            ticket()
        };
        assert!(
            f.session
                .continue_account_verification(metadata, answer)
                .await
                .is_err()
        );
        assert!(
            f.session
                .continue_account_verification(answer_metadata(challenge), ticket())
                .await
                .is_err()
        );
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    }
}

#[tokio::test]
async fn in_flight_verification_cancellation_never_publishes_or_replays() {
    for background in [false, true] {
        let f = Fixture::new(false);
        let challenge = id(begin(&f).await);
        f.cloud.stall.store(true, Ordering::SeqCst);
        let started = f.cloud.started.notified();
        let work = f
            .session
            .continue_account_verification(answer_metadata(challenge), ticket());
        tokio::pin!(work);
        tokio::select! { _ = started => (), result = &mut work => panic!("Unexpected result: {result:?}") }
        if background {
            f.session.set_foreground(false);
        } else {
            f.session.cancel("answer-captcha");
        }
        assert_eq!(work.await.unwrap_err().code, "CANCELLED");
        f.session.set_foreground(true);
        f.cloud.stall.store(false, Ordering::SeqCst);
        let calls = f.cloud.calls.load(Ordering::SeqCst);
        assert!(
            f.session
                .continue_account_verification(answer_metadata(challenge), ticket())
                .await
                .is_err()
        );
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
        assert!(
            f.call(json!({"method":"list_accounts"})).await["data"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
}

#[tokio::test]
async fn verification_requires_mijia_and_is_absent_from_ordinary_json() {
    let f = Fixture::new(true);
    let mut metadata = begin_metadata();
    metadata.provider = Some(homeverb_host::AccountProvider::Meiju);
    assert_eq!(
        f.session
            .begin_account_verification(
                metadata,
                secret(b"fiction@example.invalid"),
                secret(b"fictional-password")
            )
            .await
            .unwrap_err()
            .code,
        "AUTH_FLOW_UNSUPPORTED"
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    for method in [
        "begin_account_verification",
        "continue_account_verification",
        "discard_account_verification",
    ] {
        assert!(serde_json::from_value::<HostRequest>(json!({"schema_version":"1.0","operation_id":"test","timeout_ms":1000,"request":{"method":method}})).is_err());
    }
}

#[tokio::test]
async fn explicit_code_request_replaces_handle_and_preserves_account_waiting_state() {
    let f = Fixture::new(false);
    let old = id(begin(&f).await);
    let calls = f.cloud.calls.load(Ordering::SeqCst);
    let sent = f
        .session
        .request_account_verification_code(answer_metadata(old), Method::Email)
        .await
        .unwrap();
    assert_eq!(sent.requests, 1);
    assert!((59000..=60000).contains(&sent.cooldown_ms));
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls + 3);
    let next = id(sent.login);
    assert_ne!(old, next);
    assert!(
        f.session
            .request_account_verification_code(answer_metadata(old), Method::Email)
            .await
            .is_err()
    );
    assert!(
        f.call(json!({"method":"list_accounts"})).await["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let Login::Connected(_) = f
        .session
        .continue_account_verification(answer_metadata(next), ticket())
        .await
        .unwrap()
    else {
        panic!("Expected connection");
    };
}

#[tokio::test]
async fn code_request_refuses_foreign_captcha_cancelled_and_removed_account_contexts() {
    for mode in 0..4 {
        let f = Fixture::new(false);
        if mode == 0 {
            f.cloud.captcha.store(true, Ordering::SeqCst);
        }
        let challenge = id(begin(&f).await);
        let calls = f.cloud.calls.load(Ordering::SeqCst);
        if mode == 1 {
            f.session.set_foreground(false);
            f.session.set_foreground(true);
        }
        if mode == 2 {
            let other = Fixture::new(false);
            assert!(
                other
                    .session
                    .request_account_verification_code(answer_metadata(challenge), Method::Phone)
                    .await
                    .is_err()
            );
            assert!(f.session.discard_account_verification(challenge));
        } else {
            let mut metadata = answer_metadata(challenge);
            if mode == 3 {
                metadata.schema_version = "invalid".into();
            }
            assert!(
                f.session
                    .request_account_verification_code(metadata, Method::Phone)
                    .await
                    .is_err()
            );
            assert!(!f.session.discard_account_verification(challenge));
        }
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    }
    let f = Fixture::new(false);
    let connected = f.connect(Some("xiaomi.mijia"), None).await;
    let challenge = id(begin(&f).await);
    let account = f.call(json!({"method":"list_accounts"})).await["data"][0]["id"].clone();
    let removed = f
        .call(json!({"method":"remove_account","account_id":account,"expected_device_ids":connected["data"]["retrieval"]["device_ids"]}))
        .await;
    assert!(removed["error"].is_null());
    let calls = f.cloud.calls.load(Ordering::SeqCst);
    assert!(
        f.session
            .request_account_verification_code(answer_metadata(challenge), Method::Phone)
            .await
            .is_err()
    );
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), calls);
    assert!(serde_json::from_value::<HostRequest>(json!({"schema_version":"1.0","operation_id":"test","timeout_ms":1000,"request":{"method":"request_account_verification_code"}})).is_err());
}
