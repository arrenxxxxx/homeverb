use super::*;
use crate::CloudResponse;
use crate::auth::tests::{credentials, operation};
use async_trait::async_trait;
use std::sync::Mutex;

pub(super) fn image() -> SecretBytes {
    use base64::Engine;
    SecretBytes::new(base64::engine::general_purpose::STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jP1sAAAAASUVORK5CYII=").unwrap())
}
#[derive(Default)]
struct Interactive {
    requests: Mutex<Vec<(CloudEndpoint, String, String)>>,
    images: Mutex<u8>,
    always_reject: bool,
    foreign_image: bool,
    notification: bool,
}
#[async_trait]
impl CloudTransport for Interactive {
    async fn send(&self, request: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        assert!(!format!("{request:?}").contains("fictional"));
        let body = std::str::from_utf8(request.body()).unwrap();
        self.requests.lock().unwrap().push((
            request.endpoint(),
            body.into(),
            request.cookie().into(),
        ));
        let body = match request.endpoint() {
            CloudEndpoint::LoginSign => r#"{"_sign":"fictional-sign"}"#,
            CloudEndpoint::LoginPassword if self.notification => {
                r#"{"code":0,"notificationUrl":"https://account.xiaomi.com/private-verification"}"#
            }
            CloudEndpoint::LoginPassword if self.foreign_image => {
                r#"{"code":87001,"captchaUrl":"https://foreign.invalid/image"}"#
            }
            CloudEndpoint::LoginPassword
                if body.contains("captCode=correct") && !self.always_reject =>
            {
                r#"{"code":0,"userId":"12345","ssecurity":"AAECAwQFBgcICQoLDA0ODw==","location":"https://sts.api.io.mi.com/sts?ticket=fictional"}"#
            }
            CloudEndpoint::LoginPassword => {
                r#"{"code":87001,"captchaUrl":"/pass/getCode?token=fictional"}"#
            }
            CloudEndpoint::LoginExchange => "",
            _ => panic!("Unexpected endpoint"),
        };
        Ok(CloudResponse {
            body: SecretBytes::new(body.as_bytes().to_vec()),
            service_token: (request.endpoint() == CloudEndpoint::LoginExchange)
                .then(|| SecretBytes::new(b"fictional-token".to_vec())),
        })
    }
    async fn fetch_captcha(
        &self,
        request: &CaptchaRequest,
        _: &Operation,
    ) -> Result<CaptchaResponse> {
        assert_eq!(format!("{request:?}"), "CaptchaRequest([REDACTED])");
        let mut count = self.images.lock().unwrap();
        *count += 1;
        CaptchaResponse::new(
            image(),
            SecretBytes::new(format!("fictional-ick-{count}").into_bytes()),
        )
    }
}
async fn challenge(peer: &Interactive, op: &Operation) -> CaptchaChallenge {
    let LoginOutcome::Captcha(value) = begin_login(peer, credentials(), Region::Cn, op)
        .await
        .unwrap()
    else {
        panic!("Expected captcha")
    };
    value
}

#[tokio::test]
async fn answers_preserve_login_context_replace_cookie_and_complete_only_after_sts() {
    let peer = Interactive::default();
    let first = challenge(&peer, &operation()).await;
    assert_eq!(first.media_type(), "image/png");
    assert_eq!(first.image(), image().expose());
    assert_eq!(format!("{first:?}"), "CaptchaChallenge([REDACTED])");
    let expiry = first.expires_at();
    let LoginOutcome::Captcha(second) = first
        .answer(&peer, SecretBytes::new(b"wrong".to_vec()), &operation())
        .await
        .unwrap()
    else {
        panic!("Expected replacement")
    };
    assert_eq!(second.expires_at(), expiry);
    let LoginOutcome::Connected(session) = second
        .answer(&peer, SecretBytes::new(b"correct".to_vec()), &operation())
        .await
        .unwrap()
    else {
        panic!("Expected session")
    };
    assert_eq!(session.user_id.as_str(), "12345");
    let calls = peer.requests.lock().unwrap();
    assert_eq!(
        calls
            .iter()
            .filter(|r| r.0 == CloudEndpoint::LoginSign)
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|r| r.0 == CloudEndpoint::LoginExchange)
            .count(),
        1
    );
    let passwords: Vec<_> = calls
        .iter()
        .filter(|r| r.0 == CloudEndpoint::LoginPassword)
        .collect();
    for call in &passwords {
        assert!(call.1.contains("_sign=fictional-sign"));
        assert!(call.1.contains("hash=0DC2E467542E1B7699F270EA9AC93D4B"));
        assert!(!call.1.contains("fictional-password"));
    }
    assert!(!passwords[0].2.contains("ick="));
    assert_eq!(
        passwords[1].2,
        format!("{}; ick=fictional-ick-1", passwords[0].2)
    );
    assert_eq!(
        passwords[2].2,
        format!("{}; ick=fictional-ick-2", passwords[0].2)
    );
    assert!(!calls.last().unwrap().2.contains("ick="));
    assert_eq!(*peer.images.lock().unwrap(), 2);
}

#[tokio::test]
async fn third_failed_answer_ends_without_refetch_or_relogin() {
    let peer = Interactive {
        always_reject: true,
        ..Default::default()
    };
    let mut value = challenge(&peer, &operation()).await;
    for _ in 0..2 {
        let LoginOutcome::Captcha(next) = value
            .answer(&peer, SecretBytes::new(b"wrong".to_vec()), &operation())
            .await
            .unwrap()
        else {
            panic!()
        };
        value = next;
    }
    let error = value
        .answer(&peer, SecretBytes::new(b"wrong".to_vec()), &operation())
        .await
        .unwrap_err();
    assert_eq!(error.code, "MIJIA_AUTH_CHALLENGE_REQUIRED");
    assert_eq!(peer.requests.lock().unwrap().len(), 5);
    assert_eq!(*peer.images.lock().unwrap(), 3);
}

#[tokio::test]
async fn cancelled_expired_or_invalid_answers_never_contact_transport() {
    for mode in 0..5 {
        let peer = Interactive::default();
        let initial = operation();
        let mut value = challenge(&peer, &initial).await;
        let next = operation();
        let (answer, expected) = match mode {
            0 => {
                initial.cancellation.cancel();
                (b"correct".to_vec(), "CANCELLED")
            }
            1 => {
                value.context.lifetime.deadline = tokio::time::Instant::now();
                (b"correct".to_vec(), "DEADLINE_EXCEEDED")
            }
            2 => {
                next.cancellation.cancel();
                (b"correct".to_vec(), "CANCELLED")
            }
            3 => (b"bad;cookie".to_vec(), "INVALID_ARGUMENT"),
            _ => (vec![b'a'; 33], "INVALID_ARGUMENT"),
        };
        let error = value
            .answer(&peer, SecretBytes::new(answer), &next)
            .await
            .unwrap_err();
        assert_eq!(error.code, expected);
        assert_eq!(peer.requests.lock().unwrap().len(), 2);
        assert_eq!(*peer.images.lock().unwrap(), 1);
    }
}

#[tokio::test]
async fn legacy_login_and_other_verification_never_fetch_a_captcha() {
    for peer in [
        Interactive::default(),
        Interactive {
            notification: true,
            ..Default::default()
        },
    ] {
        assert_eq!(
            login(&peer, credentials(), Region::Cn, &operation())
                .await
                .unwrap_err()
                .code,
            "MIJIA_AUTH_CHALLENGE_REQUIRED"
        );
        assert_eq!(*peer.images.lock().unwrap(), 0);
    }
    let peer = Interactive {
        foreign_image: true,
        ..Default::default()
    };
    assert_eq!(
        begin_login(&peer, credentials(), Region::Cn, &operation())
            .await
            .unwrap_err()
            .code,
        "MIJIA_INVALID_RESPONSE"
    );
    assert_eq!(*peer.images.lock().unwrap(), 0);
}
