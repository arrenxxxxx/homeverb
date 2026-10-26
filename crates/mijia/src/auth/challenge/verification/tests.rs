use super::*;
use crate::auth::tests::{credentials, operation};
use crate::{CloudRequest, CloudResponse};
use async_trait::async_trait;
use serde_json::json;
use std::{collections::VecDeque, sync::Mutex};

fn response(body: serde_json::Value, cookies: &[&str]) -> VerificationResponse {
    VerificationResponse::new(
        200,
        SecretBytes::new(serde_json::to_vec(&body).unwrap()),
        None,
        cookies
            .iter()
            .map(|v| SecretBytes::new(v.as_bytes().to_vec()))
            .collect(),
    )
    .unwrap()
}
fn redirect(location: &str, cookies: &[&str]) -> VerificationResponse {
    VerificationResponse::new(
        302,
        SecretBytes::new(vec![]),
        Some(SecretBytes::new(location.as_bytes().to_vec())),
        cookies
            .iter()
            .map(|v| SecretBytes::new(v.as_bytes().to_vec()))
            .collect(),
    )
    .unwrap()
}
struct Peer {
    legacy: Mutex<Vec<CloudEndpoint>>,
    calls: Mutex<Vec<(Endpoint, String, String, String)>>,
    replies: Mutex<VecDeque<(Endpoint, VerificationResponse)>>,
    stall: bool,
    page: String,
    captcha_first: bool,
    direct: bool,
}
impl Peer {
    fn new(replies: Vec<(Endpoint, VerificationResponse)>) -> Self {
        Self {
            legacy: Mutex::new(vec![]),
            calls: Mutex::new(vec![]),
            replies: Mutex::new(replies.into()),
            stall: false,
            page: "https://account.xiaomi.com/fe/service/identity/authStart?context=fictional"
                .into(),
            captcha_first: false,
            direct: false,
        }
    }
}
#[async_trait]
impl CloudTransport for Peer {
    async fn send(&self, request: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        self.legacy.lock().unwrap().push(request.endpoint());
        let body = match request.endpoint() {
            CloudEndpoint::LoginSign => json!({"_sign":"fictional-sign"}),
            CloudEndpoint::LoginPassword
                if self.captcha_first
                    && !std::str::from_utf8(request.body())
                        .unwrap()
                        .contains("captCode=correct") =>
            {
                json!({"code":87001,"captchaUrl":"/pass/getCode?fictional"})
            }
            CloudEndpoint::LoginPassword => json!({"code":0,"notificationUrl":self.page}),
            _ => panic!("Unexpected legacy request"),
        };
        Ok(CloudResponse {
            body: SecretBytes::new(serde_json::to_vec(&body).unwrap()),
            service_token: None,
        })
    }
    async fn fetch_captcha(&self, _: &CaptchaRequest, _: &Operation) -> Result<CaptchaResponse> {
        CaptchaResponse::new(
            super::super::tests::image(),
            SecretBytes::new(b"fictional-ick".to_vec()),
        )
    }
    async fn send_verification(
        &self,
        request: &VerificationRequest,
        _: &Operation,
    ) -> Result<VerificationResponse> {
        if request.endpoint() == Endpoint::SigningKey && self.legacy.lock().unwrap().is_empty() {
            self.legacy.lock().unwrap().push(CloudEndpoint::LoginSign);
            return Ok(response(
                json!({"_sign":"fictional-sign"}),
                &["loginContext=fictional-context; Path=/; Secure"],
            ));
        }
        if request.endpoint() == Endpoint::Password {
            self.legacy
                .lock()
                .unwrap()
                .push(CloudEndpoint::LoginPassword);
            assert!(request.cookie().contains("loginContext=fictional-context"));
            let body = if self.captcha_first
                && !std::str::from_utf8(request.body())
                    .unwrap()
                    .contains("captCode=correct")
            {
                json!({"code":87001,"captchaUrl":"/pass/getCode?fictional"})
            } else if self.direct {
                json!({"code":0,"userId":"12345","ssecurity":"AAECAwQFBgcICQoLDA0ODw==",
                    "location":"https://sts.api.io.mi.com/sts?ticket=fictional"})
            } else {
                json!({"code":0,"notificationUrl":self.page})
            };
            return Ok(response(
                body,
                &["verifyContext=fictional-verify; Path=/identity; Secure"],
            ));
        }
        if request.endpoint() == Endpoint::Options {
            assert!(request.cookie().contains("loginContext=fictional-context"));
            assert!(request.cookie().contains("verifyContext=fictional-verify"));
        }
        assert_eq!(format!("{request:?}"), "VerificationRequest([REDACTED])");
        request.url()?;
        self.calls.lock().unwrap().push((
            request.endpoint(),
            request.location().into(),
            std::str::from_utf8(request.body()).unwrap().into(),
            request.cookie().into(),
        ));
        if self.stall {
            std::future::pending::<()>().await;
        }
        let (endpoint, response) = self
            .replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("Unexpected verification request");
        assert_eq!(endpoint, request.endpoint());
        assert_eq!(format!("{response:?}"), "VerificationResponse([REDACTED])");
        Ok(response)
    }
}
fn options() -> (Endpoint, VerificationResponse) {
    (
        Endpoint::Options,
        response(
            json!({"code":0,"options":[4,8]}),
            &["identity_session=fictional-identity; Path=/identity; Secure; HttpOnly"],
        ),
    )
}
fn accepted(method: VerificationMethod) -> (Endpoint, VerificationResponse) {
    (
        method.endpoint(),
        response(
            json!({"code":0,"location":"/identity/result/check?id=fictional"}),
            &[],
        ),
    )
}
fn complete_response() -> (Endpoint, VerificationResponse) {
    (
        Endpoint::Completion,
        response(
            json!({"code":0}),
            &[
                "passToken=fictional-pass; Domain=account.xiaomi.com; Path=/; Secure; Max-Age=180",
                "userId=12345; Path=/",
                "serviceToken=fictional-token; Path=/",
            ],
        ),
    )
}
fn signed(location: bool) -> (Endpoint, VerificationResponse) {
    let mut body = json!({"code":0,"userId":"12345","ssecurity":"AAECAwQFBgcICQoLDA0ODw=="});
    if location {
        body["location"] = json!("https://sts.api.io.mi.com/sts?ticket=fictional");
    }
    (Endpoint::SigningKey, response(body, &[]))
}
async fn challenge(peer: &Peer, op: &Operation) -> VerificationChallenge {
    let VerificationLoginOutcome::Verification(value) =
        begin_verification_login(peer, credentials(), Region::Cn, op)
            .await
            .unwrap()
    else {
        panic!("Expected verification")
    };
    value
}
#[tokio::test]
async fn both_explicit_methods_finish_only_with_signing_key_and_consistent_subject() {
    for method in [VerificationMethod::Phone, VerificationMethod::Email] {
        for exchange in [false, true] {
            let mut replies = vec![
                options(),
                accepted(method),
                complete_response(),
                signed(exchange),
            ];
            if exchange {
                replies.push((
                    Endpoint::Exchange,
                    response(
                        json!({}),
                        &[
                            "serviceToken=fictional-sts-token; Domain=.io.mi.com; Path=/; Secure",
                            "userId=12345; Path=/",
                        ],
                    ),
                ));
            }
            let peer = Peer::new(replies);
            let value = challenge(&peer, &operation()).await;
            assert_eq!(
                value.methods(),
                &[VerificationMethod::Phone, VerificationMethod::Email]
            );
            assert_eq!(value.provider_page(), peer.page);
            assert_eq!(format!("{value:?}"), "VerificationChallenge([REDACTED])");
            let session = value
                .answer(
                    &peer,
                    method,
                    SecretBytes::new(b"fictional+ticket/value=".to_vec()),
                    &operation(),
                )
                .await
                .unwrap();
            assert_eq!(session.user_id.as_str(), "12345");
            assert_eq!(
                session.token.as_str(),
                if exchange {
                    "fictional-sts-token"
                } else {
                    "fictional-token"
                }
            );
            let calls = peer.calls.lock().unwrap();
            assert_eq!(
                calls[0].1,
                "https://account.xiaomi.com/identity/list?context=fictional"
            );
            assert!(calls[1].2.contains(&format!("_flag={}", method.flag())));
            assert!(calls[1].2.contains("ticket=fictional%2Bticket%2Fvalue%3D"));
            assert!(calls[1].2.contains("trust=false"));
            assert!(calls[1].2.contains("_json=true"));
            assert!(calls[1].3.contains("identity_session=fictional-identity"));
            assert!(calls[3].3.contains("passToken=fictional-pass"));
            assert!(!calls[3].3.contains("identity_session="));
            assert!(calls.iter().all(|c| !c.3.contains("serviceToken=")));
            if exchange {
                assert!(calls[4].3.is_empty());
            }
            assert_eq!(
                *peer.legacy.lock().unwrap(),
                [CloudEndpoint::LoginSign, CloudEndpoint::LoginPassword]
            );
        }
    }
}
#[tokio::test]
async fn captcha_can_transition_only_on_the_explicit_extended_path() {
    let mut peer = Peer::new(vec![options()]);
    peer.captcha_first = true;
    let VerificationLoginOutcome::Captcha(value) =
        begin_verification_login(&peer, credentials(), Region::Cn, &operation())
            .await
            .unwrap()
    else {
        panic!()
    };
    let expiry = value.expires_at();
    let VerificationLoginOutcome::Verification(value) = value
        .answer_with_verification(&peer, SecretBytes::new(b"correct".to_vec()), &operation())
        .await
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(value.expires_at(), expiry);
    let peer = Peer::new(vec![]);
    assert_eq!(
        begin_login(&peer, credentials(), Region::Cn, &operation())
            .await
            .unwrap_err()
            .code,
        "MIJIA_AUTH_CHALLENGE_REQUIRED"
    );
    assert!(peer.calls.lock().unwrap().is_empty());
    let peer = Peer::new(vec![]);
    assert_eq!(
        crate::login(&peer, credentials(), Region::Cn, &operation())
            .await
            .unwrap_err()
            .code,
        "MIJIA_AUTH_CHALLENGE_REQUIRED"
    );
    assert!(peer.calls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn invalid_origin_options_and_cookies_never_create_a_challenge() {
    for page in [
        "https://foreign.invalid/fe/service/identity/authStart",
        "//account.xiaomi.com/fe/service/identity/authStart",
        "https://account.xiaomi.com:444/fe/service/identity/authStart",
        "https://account.xiaomi.com/other",
        "https://account.xiaomi.com/fe/../fe/service/identity/authStart",
    ] {
        let mut peer = Peer::new(vec![]);
        peer.page = page.into();
        assert!(
            begin_verification_login(&peer, credentials(), Region::Cn, &operation())
                .await
                .is_err()
        );
        assert!(peer.calls.lock().unwrap().is_empty());
    }
    for (body, cookie) in [
        (
            json!({"code":0,"options":[]}),
            "identity_session=fictional; Path=/",
        ),
        (json!({"code":0}), "identity_session=fictional; Path=/"),
        (
            json!({"code":0,"options":[1,2,3]}),
            "identity_session=fictional; Path=/",
        ),
        (
            json!({"code":0,"options":[4,4]}),
            "identity_session=fictional; Path=/",
        ),
        (
            json!({"code":0,"options":[4]}),
            "identity_session=fictional; Domain=foreign.invalid; Path=/",
        ),
        (
            json!({"code":0,"options":[4]}),
            "identity_session=fictional; Path=/other",
        ),
        (
            json!({"code":0,"options":[4]}),
            "identity_session=fictional; Max-Age=0; Path=/",
        ),
        (
            json!({"code":0,"options":[4]}),
            "identity_session=fictional; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Path=/",
        ),
        (
            json!({"code":0,"options":[4]}),
            "identity_session=fictional; Expires=invalid; Path=/",
        ),
        (
            json!({"code":1,"options":[4]}),
            "identity_session=fictional; Path=/",
        ),
    ] {
        let peer = Peer::new(vec![(Endpoint::Options, response(body, &[cookie]))]);
        assert!(
            begin_verification_login(&peer, credentials(), Region::Cn, &operation())
                .await
                .is_err()
        );
        assert_eq!(peer.calls.lock().unwrap().len(), 1);
    }
}
#[tokio::test]
async fn rejected_tickets_do_not_try_other_methods_or_replay_passwords() {
    let peer = Peer::new(vec![
        options(),
        (Endpoint::Phone, response(json!({"code":70016}), &[])),
    ]);
    let value = challenge(&peer, &operation()).await;
    assert_eq!(
        value
            .answer(
                &peer,
                VerificationMethod::Phone,
                SecretBytes::new(b"fictional-rejected".to_vec()),
                &operation()
            )
            .await
            .unwrap_err()
            .code,
        "MIJIA_AUTH_CHALLENGE_REQUIRED"
    );
    assert_eq!(peer.calls.lock().unwrap().len(), 2);
    assert_eq!(peer.legacy.lock().unwrap().len(), 2);
}
#[tokio::test]
async fn missing_signing_key_token_or_subject_cannot_complete() {
    for body in [
        json!({"code":0}),
        json!({"code":0,"ssecurity":"bad"}),
        json!({"code":0,"userId":"99999","ssecurity":"AAECAwQFBgcICQoLDA0ODw=="}),
        json!({"code":0,"userId":"0","ssecurity":"AAECAwQFBgcICQoLDA0ODw=="}),
    ] {
        let peer = Peer::new(vec![
            options(),
            accepted(VerificationMethod::Phone),
            complete_response(),
            (Endpoint::SigningKey, response(body, &[])),
        ]);
        let value = challenge(&peer, &operation()).await;
        assert!(
            value
                .answer(
                    &peer,
                    VerificationMethod::Phone,
                    SecretBytes::new(b"fictional-ticket".to_vec()),
                    &operation()
                )
                .await
                .is_err()
        );
        assert_eq!(peer.calls.lock().unwrap().len(), 4);
    }
    for cookies in [
        vec![],
        vec!["userId=12345; Path=/"],
        vec![
            "userId=12345; Path=/",
            "serviceToken=fictional; Max-Age=0; Path=/",
        ],
    ] {
        let peer = Peer::new(vec![
            options(),
            accepted(VerificationMethod::Phone),
            (Endpoint::Completion, response(json!({"code":0}), &cookies)),
            signed(false),
        ]);
        let value = challenge(&peer, &operation()).await;
        assert!(
            value
                .answer(
                    &peer,
                    VerificationMethod::Phone,
                    SecretBytes::new(b"fictional-ticket".to_vec()),
                    &operation()
                )
                .await
                .is_err()
        );
    }
    let peer = Peer::new(vec![
        options(),
        accepted(VerificationMethod::Phone),
        complete_response(),
        signed(true),
        (
            Endpoint::Exchange,
            response(
                json!({}),
                &["serviceToken=fictional; Path=/", "userId=99999; Path=/"],
            ),
        ),
    ]);
    let value = challenge(&peer, &operation()).await;
    assert!(
        value
            .answer(
                &peer,
                VerificationMethod::Phone,
                SecretBytes::new(b"fictional-ticket".to_vec()),
                &operation()
            )
            .await
            .is_err()
    );
}
#[tokio::test]
async fn redirects_are_bounded_and_never_confirm_provider_account_settings() {
    for location in [
        "https://foreign.invalid/identity/result/check",
        "/fe/service/verifyEmail?skipUrl=fictional",
        "/pass2/confirmPhone?selected=0",
        "/identity/result/check?id=fictional",
    ] {
        let peer = Peer::new(vec![
            options(),
            accepted(VerificationMethod::Phone),
            (Endpoint::Completion, redirect(location, &[])),
        ]);
        let value = challenge(&peer, &operation()).await;
        assert!(
            value
                .answer(
                    &peer,
                    VerificationMethod::Phone,
                    SecretBytes::new(b"fictional-ticket".to_vec()),
                    &operation()
                )
                .await
                .is_err()
        );
        assert_eq!(peer.calls.lock().unwrap().len(), 3);
    }
    let mut replies = vec![options(), accepted(VerificationMethod::Email)];
    for n in 0..4 {
        replies.push((
            Endpoint::Completion,
            redirect(&format!("/identity/result/check?id=next-{n}"), &[]),
        ));
    }
    let peer = Peer::new(replies);
    let value = challenge(&peer, &operation()).await;
    assert!(
        value
            .answer(
                &peer,
                VerificationMethod::Email,
                SecretBytes::new(b"fictional-ticket".to_vec()),
                &operation()
            )
            .await
            .is_err()
    );
    assert_eq!(peer.calls.lock().unwrap().len(), 6);
}
#[tokio::test]
async fn cancellation_expiry_invalid_method_and_ticket_consume_without_sending() {
    for mode in 0..7 {
        let peer = Peer::new(vec![(
            Endpoint::Options,
            response(
                json!({"code":0,"flag":4}),
                &["identity_session=fictional; Path=/identity"],
            ),
        )]);
        let original = operation();
        let mut value = challenge(&peer, &original).await;
        let op = operation();
        let mut ticket = SecretBytes::new(b"fictional-ticket".to_vec());
        let mut method = VerificationMethod::Phone;
        match mode {
            0 => original.cancellation.cancel(),
            1 => op.cancellation.cancel(),
            2 => value.lifetime.deadline = tokio::time::Instant::now(),
            3 => ticket = SecretBytes::new(vec![]),
            4 => ticket = SecretBytes::new(vec![b'a'; 2049]),
            5 => ticket = SecretBytes::new(b"bad\nvalue".to_vec()),
            6 => method = VerificationMethod::Email,
            _ => unreachable!(),
        }
        assert!(value.answer(&peer, method, ticket, &op).await.is_err());
        assert_eq!(peer.calls.lock().unwrap().len(), 1);
    }
}
#[tokio::test]
async fn in_flight_ticket_cancellation_timeout_and_drop_do_not_replay() {
    for mode in 0..3 {
        let mut peer = Peer::new(vec![options()]);
        let value = challenge(&peer, &operation()).await;
        peer.stall = true;
        let op = Operation::new(
            if mode == 1 { 25 } else { 5000 },
            tokio_util::sync::CancellationToken::new(),
        )
        .unwrap();
        let mut work = Box::pin(value.answer(
            &peer,
            VerificationMethod::Phone,
            SecretBytes::new(b"fictional-ticket".to_vec()),
            &op,
        ));
        tokio::select! { r=&mut work=>panic!("Unexpected completion: {r:?}"), _=tokio::time::sleep(Duration::from_millis(5))=>() }
        if mode == 0 {
            op.cancellation.cancel();
            assert_eq!(work.await.unwrap_err().code, "CANCELLED");
        } else if mode == 1 {
            assert_eq!(work.await.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            drop(work);
        }
        assert_eq!(peer.calls.lock().unwrap().len(), 2);
        assert_eq!(peer.legacy.lock().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn explicit_result_to_sts_redirect_and_cookie_subject_complete_without_another_exchange() {
    let peer = Peer::new(vec![
        options(),
        accepted(VerificationMethod::Email),
        (
            Endpoint::Completion,
            redirect(
                "https://sts.api.io.mi.com/sts?ticket=fictional",
                &["passToken=fictional-pass; Path=/", "userId=12345; Path=/"],
            ),
        ),
        (
            Endpoint::Exchange,
            response(
                json!({}),
                &["serviceToken=fictional-token; Domain=.io.mi.com; Path=/"],
            ),
        ),
        (
            Endpoint::SigningKey,
            response(
                json!({"code":0,"ssecurity":"AAECAwQFBgcICQoLDA0ODw=="}),
                &[],
            ),
        ),
    ]);
    let value = challenge(&peer, &operation()).await;
    let session = value
        .answer(
            &peer,
            VerificationMethod::Email,
            SecretBytes::new(b"fictional-ticket".to_vec()),
            &operation(),
        )
        .await
        .unwrap();
    assert_eq!(session.user_id.as_str(), "12345");
    let calls = peer.calls.lock().unwrap();
    assert_eq!(calls.len(), 5);
    assert!(calls[3].3.is_empty());
    assert!(calls[4].3.contains("passToken=fictional-pass"));
    assert!(!calls[4].3.contains("serviceToken="));
}

#[test]
fn cookie_ownership_rejects_ambiguity_and_honors_path_expiry_and_identity() {
    let account = account_url("/identity/list").unwrap();
    let verify = account_url("/identity/auth/verifyPhone").unwrap();
    let sign = account_url("/pass/serviceLogin?sid=xiaomiio&_json=true").unwrap();
    let mut jar = Cookies::new("sdkVersion=3.8.6; deviceId=fictional").unwrap();
    jar.absorb(
        &account,
        vec![SecretBytes::new(b"identity_session=fictional".to_vec())],
    )
    .unwrap();
    assert!(jar.has_identity_session(&verify));
    assert!(!jar.header(&sign).as_str().contains("identity_session="));
    let sibling = account_url("/identity-other/test").unwrap();
    assert!(!jar.has_identity_session(&sibling));
    jar.absorb(
        &account,
        vec![SecretBytes::new(b"identity_session=; Max-Age=0".to_vec())],
    )
    .unwrap();
    assert!(!jar.has_identity_session(&verify));
    for headers in [
        vec![
            "identity_session=one; Path=/",
            "identity_session=two; Path=/",
        ],
        vec!["identity_session=one; Domain=.com; Path=/"],
        vec!["identity_session=one; Domain=foreign.xiaomi.com; Path=/"],
        vec!["identity_session=one; Path=/; Path=/identity"],
        vec!["identity_session=one; Max-Age=invalid; Path=/"],
        vec!["identity_session=bad value; Path=/"],
    ] {
        let mut jar = Cookies::default();
        assert!(
            jar.absorb(
                &account,
                headers
                    .iter()
                    .map(|v| SecretBytes::new(v.as_bytes().to_vec()))
                    .collect()
            )
            .is_err()
        );
    }
    let mut jar = Cookies::default();
    jar.absorb(
        &account,
        vec![SecretBytes::new(b"userId=12345; Path=/".to_vec())],
    )
    .unwrap();
    assert!(
        jar.absorb(
            &account,
            vec![SecretBytes::new(b"userId=99999; Path=/".to_vec())]
        )
        .is_err()
    );
    let mut jar = Cookies::default();
    jar.absorb(
        &account,
        vec![SecretBytes::new(b"identity_session=one; Path=/".to_vec())],
    )
    .unwrap();
    assert!(
        jar.absorb(
            &account,
            vec![SecretBytes::new(
                b"identity_session=two; Path=/identity".to_vec()
            )]
        )
        .is_err()
    );
}

#[tokio::test(start_paused = true)]
async fn identity_cookie_expiry_is_monotonic_and_blocks_later_ticket_submission() {
    let peer = Peer::new(vec![(
        Endpoint::Options,
        response(
            json!({"code":0,"options":[4]}),
            &["identity_session=fictional; Path=/identity; Max-Age=2"],
        ),
    )]);
    let value = challenge(&peer, &operation()).await;
    tokio::time::advance(Duration::from_secs(3)).await;
    assert_eq!(
        value
            .answer(
                &peer,
                VerificationMethod::Phone,
                SecretBytes::new(b"fictional-ticket".to_vec()),
                &operation()
            )
            .await
            .unwrap_err()
            .code,
        "MIJIA_AUTH_CHALLENGE_REQUIRED"
    );
    assert_eq!(peer.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn extended_direct_and_captcha_success_validate_exchange_identity_without_options() {
    for captcha_first in [false, true] {
        for subject in ["12345", "99999"] {
            let mut peer = Peer::new(vec![(
                Endpoint::Exchange,
                response(
                    json!({}),
                    &[
                        "serviceToken=fictional-direct-token; Path=/; Secure",
                        &format!("userId={subject}; Path=/; Secure"),
                    ],
                ),
            )]);
            peer.direct = true;
            peer.captcha_first = captcha_first;
            let result =
                begin_verification_login(&peer, credentials(), Region::Cn, &operation()).await;
            let result = if captcha_first {
                let VerificationLoginOutcome::Captcha(challenge) = result.unwrap() else {
                    panic!("Expected captcha");
                };
                challenge
                    .answer_with_verification(
                        &peer,
                        SecretBytes::new(b"correct".to_vec()),
                        &operation(),
                    )
                    .await
            } else {
                result
            };
            if subject == "12345" {
                let VerificationLoginOutcome::Connected(session) = result.unwrap() else {
                    panic!("Expected connected session");
                };
                assert_eq!(session.user_id.as_str(), subject);
                assert_eq!(session.token.as_str(), "fictional-direct-token");
            } else {
                assert_eq!(result.unwrap_err().code, "MIJIA_INVALID_RESPONSE");
            }
            let calls = peer.calls.lock().unwrap();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].0, Endpoint::Exchange);
            assert!(calls[0].3.is_empty());
        }
    }
}
