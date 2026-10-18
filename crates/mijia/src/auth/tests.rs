use super::*;
use crate::{CloudResponse, codec};
use async_trait::async_trait;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

fn operation() -> Operation {
    Operation::new(10_000, CancellationToken::new()).unwrap()
}
fn credentials() -> LoginCredentials {
    LoginCredentials::new(
        SecretBytes::new(b"fictional@example.invalid".to_vec()),
        SecretBytes::new(b"fictional-password".to_vec()),
    )
    .unwrap()
}
struct Script {
    calls: Mutex<Vec<CloudEndpoint>>,
    reply: String,
    token: Option<&'static str>,
}
impl Script {
    fn valid() -> Self {
        Self { calls: Mutex::new(vec![]), reply: r#"{"code":0,"userId":12345,"ssecurity":"AAECAwQFBgcICQoLDA0ODw==","location":"https://sts.api.io.mi.com/sts?ticket=fictional"}"#.into(), token: Some("fictional-token") }
    }
}
#[async_trait]
impl CloudTransport for Script {
    async fn send(&self, request: &CloudRequest, operation: &Operation) -> Result<CloudResponse> {
        operation
            .run(async {
                self.calls.lock().unwrap().push(request.endpoint());
                assert!(!format!("{request:?}").contains("fictional"));
                let body = match request.endpoint() {
                    CloudEndpoint::LoginSign => {
                        r#"&&&START&&&{"_sign":"fictional-sign"}"#.to_owned()
                    }
                    CloudEndpoint::LoginPassword => {
                        let form = reqwest::Url::parse(&format!(
                            "https://localhost/?{}",
                            std::str::from_utf8(request.body()).unwrap()
                        ))
                        .unwrap();
                        let fields: std::collections::HashMap<_, _> =
                            form.query_pairs().into_owned().collect();
                        assert_eq!(fields["user"], "fictional@example.invalid");
                        assert_eq!(fields["hash"], "0DC2E467542E1B7699F270EA9AC93D4B");
                        assert_eq!(fields["callback"], "https://sts.api.io.mi.com/sts");
                        self.reply.clone()
                    }
                    CloudEndpoint::LoginExchange => "".into(),
                    _ => panic!("No implicit inventory request."),
                };
                Ok(CloudResponse {
                    body: SecretBytes::new(body.into_bytes()),
                    service_token: if request.endpoint() == CloudEndpoint::LoginExchange {
                        self.token.map(|t| SecretBytes::new(t.as_bytes().to_vec()))
                    } else {
                        None
                    },
                })
            })
            .await
    }
}

#[tokio::test]
async fn login_is_exact_bounded_and_redacted() {
    let transport = Script::valid();
    let session = login(&transport, credentials(), Region::Cn, &operation())
        .await
        .unwrap();
    assert_eq!(session.region(), Region::Cn);
    assert_eq!(session.user_id.as_str(), "12345");
    assert_eq!(session.token.as_str(), "fictional-token");
    assert_eq!(format!("{session:?}"), "MijiaSession([REDACTED])");
    assert_eq!(
        transport.calls.lock().unwrap().as_slice(),
        &[
            CloudEndpoint::LoginSign,
            CloudEndpoint::LoginPassword,
            CloudEndpoint::LoginExchange
        ]
    );
}

#[tokio::test]
async fn challenge_rejection_and_unsafe_exchange_never_send_third_request() {
    for (reply, code) in [
        (
            r#"{"code":87001,"captchaUrl":"/private-challenge"}"#,
            "MIJIA_AUTH_CHALLENGE_REQUIRED",
        ),
        (
            r#"{"code":0,"notificationUrl":"https://account.xiaomi.com/private-ticket"}"#,
            "MIJIA_AUTH_CHALLENGE_REQUIRED",
        ),
        (
            r#"{"code":70016,"message":"private-account"}"#,
            "MIJIA_REQUEST_REJECTED",
        ),
        (
            r#"{"code":0,"userId":"12345","ssecurity":"AAECAwQFBgcICQoLDA0ODw==","location":"https://evil.invalid/sts"}"#,
            "MIJIA_INVALID_RESPONSE",
        ),
        (
            r#"{"code":0,"userId":"12345;inject=true","ssecurity":"AAECAwQFBgcICQoLDA0ODw==","location":"https://sts.api.io.mi.com/sts"}"#,
            "MIJIA_INVALID_RESPONSE",
        ),
        (
            r#"{"code":0,"userId":"12345","ssecurity":"AA==","location":"https://sts.api.io.mi.com/sts"}"#,
            "MIJIA_INVALID_RESPONSE",
        ),
    ] {
        let mut transport = Script::valid();
        transport.reply = reply.into();
        let error = login(&transport, credentials(), Region::Cn, &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, code);
        assert!(!format!("{error:?}").contains("private"));
        assert_eq!(transport.calls.lock().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn missing_or_injected_cookie_fails_without_success() {
    for token in [None, Some("bad;cookie"), Some("bad\r\nheader")] {
        let mut transport = Script::valid();
        transport.token = token;
        assert!(
            login(&transport, credentials(), Region::Cn, &operation())
                .await
                .is_err()
        );
        assert_eq!(transport.calls.lock().unwrap().len(), 3);
    }
}

#[tokio::test]
async fn cancelled_and_expired_login_never_contacts_transport() {
    let transport = Script::valid();
    let op = operation();
    op.cancellation.cancel();
    assert_eq!(
        login(&transport, credentials(), Region::Cn, &op)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let mut op = operation();
    op.deadline = tokio::time::Instant::now();
    assert_eq!(
        login(&transport, credentials(), Region::Cn, &op)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert!(transport.calls.lock().unwrap().is_empty());
}

struct ReadReply {
    payload: &'static str,
    plain: bool,
}
#[async_trait]
impl CloudTransport for ReadReply {
    async fn send(&self, request: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        assert!(request.cookie().contains("serviceToken=fictional-token"));
        let url = reqwest::Url::parse(&format!(
            "https://localhost/?{}",
            std::str::from_utf8(request.body()).unwrap()
        ))
        .unwrap();
        let fields: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"])?;
        let body = if self.plain {
            self.payload.to_owned()
        } else {
            codec::encrypt(&key, self.payload.as_bytes())
                .as_str()
                .to_owned()
        };
        Ok(CloudResponse {
            body: SecretBytes::new(body.into_bytes()),
            service_token: None,
        })
    }
}
#[tokio::test]
async fn request_decrypts_and_classifies_without_accepting_plain_success() {
    let session = login(&Script::valid(), credentials(), Region::Cn, &operation())
        .await
        .unwrap();
    for (payload, plain, error) in [
        (r#"{"code":0,"result":{"list":[]}}"#, false, None),
        (
            r#"{"code":0,"result":{"list":[]}}"#,
            true,
            Some("MIJIA_INVALID_RESPONSE"),
        ),
        (
            r#"{"code":-3,"message":"private-token"}"#,
            true,
            Some("AUTH_REQUIRED"),
        ),
        (
            r#"{"code":123,"message":"private-token"}"#,
            false,
            Some("MIJIA_REQUEST_REJECTED"),
        ),
    ] {
        let result = session
            .inventory(&ReadReply { payload, plain }, &operation())
            .await;
        assert_eq!(result.err().map(|e| e.code), error.map(str::to_owned));
    }
}
