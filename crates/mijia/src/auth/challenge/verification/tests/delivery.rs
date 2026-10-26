use super::*;

fn delivery(method: VerificationMethod, wait: u64) -> Vec<(Endpoint, VerificationResponse)> {
    let info = match method {
        VerificationMethod::Phone => Endpoint::PhoneInfo,
        VerificationMethod::Email => Endpoint::EmailInfo,
    };
    let send = match method {
        VerificationMethod::Phone => Endpoint::SendPhone,
        VerificationMethod::Email => Endpoint::SendEmail,
    };
    vec![
        (
            info,
            response(
                json!({"code":0, "contentType":"160010", "maskedPhone":"FICTIONAL-MASK", "maskedEmail":"FICTIONAL-MASK"}),
                &[],
            ),
        ),
        (
            Endpoint::Quota,
            response(json!({"code":0,"result":0,"info":"3"}), &[]),
        ),
        (send, response(json!({"code":0,"data":{"wt":wait}}), &[])),
    ]
}
#[tokio::test]
async fn explicit_phone_and_email_delivery_preserve_session_and_exact_forms() {
    for method in [VerificationMethod::Phone, VerificationMethod::Email] {
        let mut replies = vec![options()];
        replies.extend(delivery(method, 30));
        replies.extend([accepted(method), complete_response(), signed(false)]);
        let peer = Peer::new(replies);
        let value = challenge(&peer, &operation()).await;
        assert_eq!(value.code_requests(), 0);
        assert_eq!(peer.calls.lock().unwrap().len(), 1);
        let expiry = value.expires_at();
        let value = value
            .request_code(&peer, method, &operation())
            .await
            .unwrap();
        assert_eq!(value.expires_at(), expiry);
        assert_eq!(value.code_requests(), 1);
        assert!(value.code_request_cooldown() > Duration::from_secs(59));
        let session = value
            .answer(
                &peer,
                method,
                SecretBytes::new(b"fictional-ticket".to_vec()),
                &operation(),
            )
            .await
            .unwrap();
        assert_eq!(session.user_id.as_str(), "12345");
        let calls = peer.calls.lock().unwrap();
        assert!(
            calls[1]
                .1
                .ends_with(&format!("?_flag={}&_json=true", method.flag()))
        );
        assert!(calls[1].2.is_empty());
        assert_eq!(
            calls[2].2,
            format!(
                "addressType={}&contentType=160010&_json=true",
                if method == VerificationMethod::Phone {
                    "PH"
                } else {
                    "EM"
                }
            )
        );
        assert_eq!(calls[3].2, "retry=0&icode=&_json=true");
        assert!(calls[3].3.contains("identity_session="));
        assert!(
            calls
                .iter()
                .all(|c| !c.2.contains("FICTIONAL-MASK") && !c.3.contains("serviceToken="))
        );
        if method == VerificationMethod::Email {
            assert!(calls[3].1.contains("sendEmailTicket?_dc="));
        }
        assert_eq!(peer.legacy.lock().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn unavailable_quota_or_method_information_prevents_delivery() {
    for quota in [
        json!({"code":1,"result":0,"info":3}),
        json!({"code":0,"result":1,"info":3}),
        json!({"code":0,"result":0,"info":0}),
        json!({"code":0,"result":0,"info":"bad"}),
        json!({"code":0,"result":0,"info":-1}),
        json!({"code":0,"result":0,"info":true}),
        json!({"code":0,"result":0,"info":1000001}),
    ] {
        let peer = Peer::new(vec![
            options(),
            (Endpoint::PhoneInfo, response(json!({"code":0}), &[])),
            (Endpoint::Quota, response(quota, &[])),
        ]);
        let value = challenge(&peer, &operation()).await;
        assert!(
            value
                .request_code(&peer, VerificationMethod::Phone, &operation())
                .await
                .is_err()
        );
        assert_eq!(peer.calls.lock().unwrap().len(), 3);
    }
    for info in [
        json!({"code":1}),
        json!({"code":0,"contentType":"foreign&value"}),
        json!({"code":0,"contentType":true}),
    ] {
        let peer = Peer::new(vec![options(), (Endpoint::PhoneInfo, response(info, &[]))]);
        let value = challenge(&peer, &operation()).await;
        assert!(
            value
                .request_code(&peer, VerificationMethod::Phone, &operation())
                .await
                .is_err()
        );
        assert_eq!(peer.calls.lock().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn provider_risk_rejection_and_malformed_send_reply_never_trigger_another_attempt() {
    for body in [
        json!({"code":87001}),
        json!({"code":20031}),
        json!({"code":20024}),
        json!({"code":0,"data":{"wt":86401}}),
    ] {
        let mut replies = vec![options()];
        let mut path = delivery(VerificationMethod::Phone, 60);
        path.pop();
        replies.extend(path);
        replies.push((Endpoint::SendPhone, response(body, &[])));
        let peer = Peer::new(replies);
        let value = challenge(&peer, &operation()).await;
        assert!(
            value
                .request_code(&peer, VerificationMethod::Phone, &operation())
                .await
                .is_err()
        );
        assert_eq!(peer.calls.lock().unwrap().len(), 4);
        assert_eq!(peer.legacy.lock().unwrap().len(), 2);
    }
}

#[tokio::test(start_paused = true)]
async fn cooldown_and_total_count_are_monotonic_and_do_not_extend_the_original_lifetime() {
    let mut replies = vec![options()];
    for _ in 0..3 {
        replies.extend(delivery(VerificationMethod::Phone, 60));
    }
    let peer = Peer::new(replies);
    let mut value = challenge(&peer, &operation()).await;
    let expiry = value.expires_at();
    for index in 0..3 {
        if index > 0 {
            tokio::time::advance(Duration::from_secs(61)).await;
        }
        value = value
            .request_code(&peer, VerificationMethod::Phone, &operation())
            .await
            .unwrap();
        assert_eq!(value.code_requests(), index + 1);
        assert_eq!(value.expires_at(), expiry);
    }
    assert!(
        value
            .request_code(&peer, VerificationMethod::Email, &operation())
            .await
            .is_err()
    );
    {
        let calls = peer.calls.lock().unwrap();
        assert_eq!(calls.len(), 10);
        for index in 0..3 {
            assert_eq!(
                calls[3 + index * 3].2,
                format!("retry={index}&icode=&_json=true")
            );
        }
    }
    let mut replies = vec![options()];
    replies.extend(delivery(VerificationMethod::Phone, 90));
    let peer = Peer::new(replies);
    let value = challenge(&peer, &operation()).await;
    let value = value
        .request_code(&peer, VerificationMethod::Phone, &operation())
        .await
        .unwrap();
    tokio::time::advance(Duration::from_secs(61)).await;
    assert!(
        value
            .request_code(&peer, VerificationMethod::Email, &operation())
            .await
            .is_err()
    );
    assert_eq!(peer.calls.lock().unwrap().len(), 4);
}

#[tokio::test]
async fn cancelled_expired_or_unsupported_requests_do_not_send() {
    for mode in 0..4 {
        let peer = Peer::new(vec![(
            Endpoint::Options,
            response(
                json!({"code":0,"options":[4]}),
                &["identity_session=fictional; Path=/identity"],
            ),
        )]);
        let original = operation();
        let mut value = challenge(&peer, &original).await;
        let op = operation();
        match mode {
            0 => original.cancellation.cancel(),
            1 => op.cancellation.cancel(),
            2 => value.lifetime.deadline = tokio::time::Instant::now(),
            _ => (),
        }
        let method = if mode == 3 {
            VerificationMethod::Email
        } else {
            VerificationMethod::Phone
        };
        assert!(value.request_code(&peer, method, &op).await.is_err());
        assert_eq!(peer.calls.lock().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn pending_send_cancellation_timeout_and_drop_do_not_resend() {
    for mode in 0..3 {
        let mut replies = vec![options()];
        replies.extend(delivery(VerificationMethod::Phone, 60));
        let mut peer = Peer::new(replies);
        let value = challenge(&peer, &operation()).await;
        peer.stall_endpoint = Some(Endpoint::SendPhone);
        let op = Operation::new(
            if mode == 1 { 25 } else { 5000 },
            tokio_util::sync::CancellationToken::new(),
        )
        .unwrap();
        let mut work = Box::pin(value.request_code(&peer, VerificationMethod::Phone, &op));
        tokio::select! { result=&mut work=>panic!("Unexpected result: {result:?}"),_=tokio::time::sleep(Duration::from_millis(5))=>() }
        if mode == 0 {
            op.cancellation.cancel();
            assert_eq!(work.await.unwrap_err().code, "CANCELLED");
        } else if mode == 1 {
            assert_eq!(work.await.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            drop(work);
        }
        assert_eq!(peer.calls.lock().unwrap().len(), 4);
    }
}
