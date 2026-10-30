use super::*;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

fn operation() -> Operation {
    Operation::new(180_000, CancellationToken::new()).unwrap()
}
fn location() -> String {
    Url::parse_with_params(
        "https://account.xiaomi.com/fe/service/login",
        &[
            ("sid", "mijia"),
            ("callback", CALLBACK),
            ("_sign", "fictional-sign"),
            ("qs", "%3Fsid%3Dmijia"),
        ],
    )
    .unwrap()
    .to_string()
}
fn login_reply() -> serde_json::Value {
    json!({"code":0,"userId":12345,"cUserId":"fictional-c-user","passToken":"fictional-pass-token",
        "ssecurity":"AAECAwQFBgcICQoLDA0ODw==","location":format!("{CALLBACK}?ticket=fictional-sts")})
}
struct Script {
    calls: Mutex<Vec<ModernAuthEndpoint>>,
    bootstrap: serde_json::Value,
    qr: serde_json::Value,
    login: serde_json::Value,
    token: Option<&'static str>,
    renewal: bool,
    exchange_subject: Option<&'static str>,
    exchange_c_subject: Option<&'static str>,
    stall: bool,
}
impl Script {
    fn new() -> Self {
        Self {
            calls: Mutex::new(vec![]),
            bootstrap: json!({"code":70016,"location":location()}),
            qr: json!({"code":0,"loginUrl":"https://c3.account.xiaomi.com/longPolling/login?sid=mijia&ticket=fictional-qr&dc=1&ts=2","lp":"https://c3.lp.account.xiaomi.com/lp/s?k=fictional-poll"}),
            login: login_reply(),
            token: Some("fictional-service-token"),
            renewal: false,
            exchange_subject: None,
            exchange_c_subject: None,
            stall: false,
        }
    }
    fn calls(&self) -> Vec<ModernAuthEndpoint> {
        self.calls.lock().unwrap().clone()
    }
}
#[async_trait]
impl ModernAuthTransport for Script {
    async fn send_modern_auth(
        &self,
        request: &ModernAuthRequest,
        _: &Operation,
    ) -> Result<ModernAuthResponse> {
        self.calls.lock().unwrap().push(request.endpoint());
        assert_eq!(format!("{request:?}"), "ModernAuthRequest([REDACTED])");
        let url = request.url()?;
        assert!(!request.cookie().contains("serviceToken="));
        if request.endpoint() == ModernAuthEndpoint::Bootstrap {
            assert_eq!(url.host_str(), Some("account.xiaomi.com"));
            assert_eq!(query(&url).unwrap()["sid"].as_str(), "mijia");
            assert!(request.cookie().contains("deviceId="));
            assert_eq!(
                request.cookie().contains("passToken=fictional-pass-token"),
                self.renewal
            );
        } else {
            assert!(request.cookie().is_empty());
        }
        if self.stall {
            std::future::pending::<()>().await;
        }
        let body = match request.endpoint() {
            ModernAuthEndpoint::Bootstrap if self.renewal => self.login.clone(),
            ModernAuthEndpoint::Bootstrap => self.bootstrap.clone(),
            ModernAuthEndpoint::Qr => {
                let q = query(&url).unwrap();
                assert_eq!(q["callback"].as_str(), CALLBACK);
                assert_eq!(q["_qrsize"].as_str(), "240");
                assert_eq!(q["_hasLogo"].as_str(), "false");
                self.qr.clone()
            }
            ModernAuthEndpoint::Poll => self.login.clone(),
            ModernAuthEndpoint::Exchange => json!({}),
        };
        Ok(ModernAuthResponse {
            body: SecretBytes::new(serde_json::to_vec(&body).unwrap()),
            user_id: self
                .exchange_subject
                .map(|v| SecretBytes::new(v.as_bytes().to_vec())),
            c_user_id: self
                .exchange_c_subject
                .map(|v| SecretBytes::new(v.as_bytes().to_vec())),
            service_token: if request.endpoint() == ModernAuthEndpoint::Exchange {
                self.token.map(|v| SecretBytes::new(v.as_bytes().to_vec()))
            } else {
                None
            },
        })
    }
}
async fn session() -> ModernMijiaSession {
    let script = Script::new();
    begin_modern_qr_login(&script, &operation())
        .await
        .unwrap()
        .finish(&script, &operation())
        .await
        .unwrap()
}

#[tokio::test]
async fn modern_qr_and_renewal_are_explicit_exact_and_separate() {
    let script = Script::new();
    let original = operation();
    let challenge = begin_modern_qr_login(&script, &original).await.unwrap();
    assert_eq!(
        script.calls(),
        [ModernAuthEndpoint::Bootstrap, ModernAuthEndpoint::Qr]
    );
    assert_eq!(challenge.expires_at(), original.deadline);
    assert!(
        std::str::from_utf8(challenge.scan_payload())
            .unwrap()
            .contains("ticket=fictional-qr")
    );
    assert_eq!(format!("{challenge:?}"), "ModernQrChallenge([REDACTED])");
    let saved = challenge.finish(&script, &operation()).await.unwrap();
    assert_eq!(format!("{saved:?}"), "ModernMijiaSession([REDACTED])");
    assert_eq!(saved.subject.as_str(), "12345");
    assert_eq!(
        script.calls(),
        [
            ModernAuthEndpoint::Bootstrap,
            ModernAuthEndpoint::Qr,
            ModernAuthEndpoint::Poll,
            ModernAuthEndpoint::Exchange
        ]
    );
    let mut renewal = Script::new();
    renewal.renewal = true;
    renewal.login.as_object_mut().unwrap().remove("passToken");
    renewal.login.as_object_mut().unwrap().remove("cUserId");
    renewal.login["ssecurity"] = json!("EBESExQVFhcYGRobHB0eHw==");
    renewal.token = Some("fictional-new-service");
    let next = saved.renew(&renewal, &operation()).await.unwrap();
    assert_eq!(next.identity.device, saved.identity.device);
    assert_eq!(next.identity.agent, saved.identity.agent);
    assert_eq!(next.pass_token, saved.pass_token);
    assert_ne!(next.security, saved.security);
    assert_ne!(next.token, saved.token);
    assert_eq!(saved.token.as_str(), "fictional-service-token");
    assert_eq!(
        renewal.calls(),
        [ModernAuthEndpoint::Bootstrap, ModernAuthEndpoint::Exchange]
    );
}

#[tokio::test]
async fn bootstrap_refuses_unsafe_and_wrong_service_continuations_before_qr() {
    for bad in [
        "http://account.xiaomi.com/fe/service/login?sid=mijia",
        "https://account.xiaomi.com.evil.invalid/fe/service/login?sid=mijia",
        "https://user@account.xiaomi.com/fe/service/login?sid=mijia",
        "https://account.xiaomi.com:8443/fe/service/login?sid=mijia",
        "https://account.xiaomi.com/fe/service/login?sid=mijia#secret",
        "https://account.xiaomi.com/fe/service/login?sid=xiaomiio",
        "https://account.xiaomi.com/fe/service/login?sid=mijia&sid=mijia",
        "https://account.xiaomi.com/fe/service/login?sid=mijia&callback=https://sts.api.io.mi.com/sts",
        "https://account.xiaomi.com/fe/service/login?sid=mijia&callback=https://sts.api.mijia.tech/mijia/sts&unexpected=1",
    ] {
        let mut script = Script::new();
        script.bootstrap["location"] = json!(bad);
        assert_eq!(
            begin_modern_qr_login(&script, &operation())
                .await
                .unwrap_err()
                .code,
            "MIJIA_INVALID_RESPONSE"
        );
        assert_eq!(script.calls().len(), 1);
    }
    for code in [0, 3, 87001, -1] {
        let mut script = Script::new();
        script.bootstrap["code"] = json!(code);
        assert_eq!(
            begin_modern_qr_login(&script, &operation())
                .await
                .unwrap_err()
                .code,
            "AUTH_REQUIRED"
        );
        assert_eq!(script.calls().len(), 1);
    }
}

#[tokio::test]
async fn qr_refuses_other_shards_urls_missing_and_duplicate_fields() {
    for (key, value) in [
        (
            "loginUrl",
            "https://c3.account.xiaomi.com/longPolling/login?sid=xiaomiio&ticket=x&dc=1&ts=2",
        ),
        (
            "loginUrl",
            "https://c4.account.xiaomi.com/longPolling/login?sid=mijia&ticket=x&dc=1&ts=2",
        ),
        (
            "loginUrl",
            "https://c3.account.xiaomi.com/longPolling/login?sid=mijia&ticket=x&ticket=y&dc=1&ts=2",
        ),
        (
            "loginUrl",
            "https://c3.account.xiaomi.com/longPolling/login?sid=mijia&ticket=&dc=1&ts=2",
        ),
        ("lp", "https://c3.lp.account.xiaomi.com/lp/s?k=x&k=y"),
        ("lp", "https://c3.lp.account.xiaomi.com/lp/s?k="),
        (
            "lp",
            "https://c3.lp.account.xiaomi.com/lp/s?k=x&url=private",
        ),
        ("lp", "https://c3.lp.account.xiaomi.com/lp/s?k=x#secret"),
        ("lp", "https://foreign.invalid/lp/s?k=x"),
        ("lp", "https://127.0.0.1/lp/s?k=x"),
        ("lp", "https://c3.lp.account.xiaomi.com/other?k=x"),
    ] {
        let mut script = Script::new();
        script.qr[key] = json!(value);
        assert_eq!(
            begin_modern_qr_login(&script, &operation())
                .await
                .unwrap_err()
                .code,
            "MIJIA_INVALID_RESPONSE"
        );
        assert_eq!(script.calls().len(), 2);
    }
}

#[tokio::test]
async fn completion_requires_valid_identity_key_callback_and_token() {
    for (key, value) in [
        ("code", json!(70016)),
        ("userId", json!("012345")),
        ("userId", json!(0)),
        ("userId", json!(-1)),
        ("userId", json!("private-account")),
        ("cUserId", json!("injected;cookie=x")),
        ("passToken", json!("")),
        ("ssecurity", json!("AA==")),
        ("ssecurity", json!(null)),
        (
            "location",
            json!("https://sts.api.io.mi.com/sts?ticket=fictional"),
        ),
        (
            "location",
            json!("https://sts.api.mijia.tech/mijia/sts?ticket=x&ticket=y"),
        ),
        (
            "location",
            json!("http://sts.api.mijia.tech/mijia/sts?ticket=x"),
        ),
    ] {
        let mut script = Script::new();
        script.login[key] = value;
        let pending = begin_modern_qr_login(&script, &operation()).await.unwrap();
        let error = pending.finish(&script, &operation()).await.unwrap_err();
        assert!(["MIJIA_INVALID_RESPONSE", "AUTH_REQUIRED"].contains(&error.code.as_str()));
        assert_eq!(script.calls().len(), 3);
        assert!(!format!("{error:?}").contains("fictional"));
    }
    for token in [None, Some(""), Some("invalid;cookie=x")] {
        let mut script = Script::new();
        script.token = token;
        let pending = begin_modern_qr_login(&script, &operation()).await.unwrap();
        assert!(pending.finish(&script, &operation()).await.is_err());
        assert_eq!(script.calls().len(), 4);
    }
}

#[tokio::test]
async fn renewal_rejects_identity_change_and_expiry_without_qr_or_exchange() {
    let saved = session().await;
    for (key, value) in [
        ("userId", json!(54321)),
        ("cUserId", json!("fictional-other-user")),
        ("code", json!(70016)),
        ("code", json!(-10001)),
        ("ssecurity", json!(null)),
        ("passToken", json!("injected\r\nsecret")),
    ] {
        let mut script = Script::new();
        script.renewal = true;
        script.login[key] = value;
        assert!(saved.renew(&script, &operation()).await.is_err());
        assert_eq!(script.calls(), [ModernAuthEndpoint::Bootstrap]);
        assert_eq!(saved.subject.as_str(), "12345");
        assert_eq!(saved.token.as_str(), "fictional-service-token");
    }
}

#[tokio::test(start_paused = true)]
async fn initial_lifetime_cannot_be_extended_or_uncancelled_by_continuation() {
    for mode in 0..4 {
        let script = Script::new();
        let mut original = operation();
        original.deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        let pending = begin_modern_qr_login(&script, &original).await.unwrap();
        let continuation = operation();
        let expected = if mode % 2 == 0 {
            "CANCELLED"
        } else {
            "DEADLINE_EXCEEDED"
        };
        if mode == 0 {
            original.cancellation.cancel();
        }
        if mode == 1 {
            tokio::time::advance(Duration::from_secs(3)).await;
        }
        if mode == 2 {
            continuation.cancellation.cancel();
        }
        let mut continuation = continuation;
        if mode == 3 {
            continuation.deadline = tokio::time::Instant::now();
        }
        assert_eq!(
            pending
                .finish(&script, &continuation)
                .await
                .unwrap_err()
                .code,
            expected
        );
        assert_eq!(script.calls().len(), 2);
    }
}

#[tokio::test(start_paused = true)]
async fn pending_poll_has_one_send_and_a_120_second_cap() {
    let mut script = Script::new();
    let pending = begin_modern_qr_login(&script, &operation()).await.unwrap();
    script.stall = true;
    let start = tokio::time::Instant::now();
    // Even a trusted test transport that ignores its operation cannot extend the caller.
    let op = operation();
    let result = pending.finish(&script, &op).await;
    assert_eq!(result.unwrap_err().code, "DEADLINE_EXCEEDED");
    assert_eq!(
        tokio::time::Instant::now() - start,
        Duration::from_secs(120)
    );
    assert_eq!(script.calls().len(), 3);
}

#[tokio::test]
async fn cancellation_and_drop_stop_a_pending_uncooperative_transport() {
    for mode in 0..2 {
        let mut script = Script::new();
        let original = operation();
        let pending = begin_modern_qr_login(&script, &original).await.unwrap();
        script.stall = true;
        let next = operation();
        let mut future = Box::pin(pending.finish(&script, &next));
        tokio::select! {
            result = &mut future => panic!("Unexpected {result:?}"),
            _ = tokio::task::yield_now() => {}
        }
        assert_eq!(script.calls().len(), 3);
        if mode == 0 {
            original.cancellation.cancel();
            assert_eq!(future.await.unwrap_err().code, "CANCELLED");
        } else {
            drop(future);
        }
        tokio::task::yield_now().await;
        assert_eq!(script.calls().len(), 3);
    }
}

#[tokio::test]
async fn pre_cancelled_start_and_renewal_send_nothing() {
    let saved = session().await;
    let script = Script::new();
    let op = operation();
    op.cancellation.cancel();
    assert_eq!(
        begin_modern_qr_login(&script, &op).await.unwrap_err().code,
        "CANCELLED"
    );
    assert_eq!(
        saved.renew(&script, &op).await.unwrap_err().code,
        "CANCELLED"
    );
    assert!(script.calls().is_empty());
}

#[test]
fn protected_decoding_rejects_duplicate_status_and_oversized_bodies() {
    assert!(auth_decode::<Status>(br#"{"code":0,"code":70016}"#).is_err());
    assert!(auth_decode::<Status>(&vec![b' '; MAX_AUTH_BYTES + 1]).is_err());
    assert!(auth_decode::<Status>(br#"{"code":"0"}"#).is_err());
    assert!(auth_decode::<QrResult>(br#"{"loginUrl":"x","loginUrl":"y","lp":"z"}"#).is_err());
}

#[tokio::test]
async fn exchange_identity_evidence_must_match_the_login_and_renewal_subject() {
    for renewal in [false, true] {
        for (user_id, c_user_id, valid) in [
            (Some("12345"), Some("fictional-c-user"), true),
            (Some("54321"), None, false),
            (None, Some("fictional-other-user"), false),
            (Some("012345"), None, false),
        ] {
            let saved = session().await;
            let mut script = Script::new();
            script.renewal = renewal;
            script.exchange_subject = user_id;
            script.exchange_c_subject = c_user_id;
            let result = if renewal {
                saved.renew(&script, &operation()).await
            } else {
                begin_modern_qr_login(&script, &operation())
                    .await
                    .unwrap()
                    .finish(&script, &operation())
                    .await
            };
            assert_eq!(result.is_ok(), valid);
            if let Err(error) = result {
                assert_eq!(error.code, "MIJIA_INVALID_RESPONSE");
            }
            assert_eq!(script.calls().len(), if renewal { 2 } else { 4 });
        }
    }
}
