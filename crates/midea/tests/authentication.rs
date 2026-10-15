use async_trait::async_trait;
use homeverb_core::{CredentialRef, Operation, SecretBytes, SystemClock};
use homeverb_domain::{Error, Result};
use homeverb_midea::*;
use serde_json::{Value, json};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};
use tokio_util::sync::CancellationToken;

fn vector() -> Value {
    serde_json::from_str(include_str!("fixtures/login.json")).unwrap()
}
fn secret(s: &str) -> SecretBytes {
    SecretBytes::new(s.as_bytes().to_vec())
}
fn operation() -> Operation {
    Operation::new(1000, CancellationToken::new()).unwrap()
}
fn credentials() -> LoginCredentials {
    let v = vector();
    LoginCredentials::new(
        secret(v["account"].as_str().unwrap()),
        secret(v["password"].as_str().unwrap()),
    )
    .unwrap()
}
fn id_response() -> Value {
    json!({"code":0,"data":{"loginId":vector()["login_id"]}})
}
fn login_response() -> Value {
    json!({"code":0,"data":{"mdata":{"accessToken":"FICTIONAL-ACCESS-TOKEN"},"key":vector()["wrapped_key"]}})
}

struct Script {
    responses: Mutex<std::collections::VecDeque<Value>>,
    calls: Mutex<Vec<(CloudEndpoint, Value, bool)>>,
}
impl Script {
    fn new(responses: Vec<Value>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
            calls: Mutex::new(vec![]),
        }
    }
}
#[async_trait]
impl CloudTransport for Script {
    async fn send(&self, request: &SignedRequest, _: &Operation) -> Result<SecretBytes> {
        assert_eq!(format!("{request:?}"), "SignedRequest([REDACTED])");
        assert_eq!(request.signature().len(), 64);
        assert!(request.random().bytes().all(|b| b.is_ascii_digit()));
        let body: Value = serde_json::from_slice(request.body()).unwrap();
        assert_eq!(body["reqId"].as_str().unwrap().len(), 32);
        assert_eq!(body["stamp"].as_str().unwrap().len(), 14);
        if let Some(token) = request.token() {
            assert_eq!(token, b"FICTIONAL-ACCESS-TOKEN");
        }
        self.calls
            .lock()
            .unwrap()
            .push((request.endpoint(), body, request.token().is_some()));
        Ok(SecretBytes::new(
            serde_json::to_vec(
                &self
                    .responses
                    .lock()
                    .unwrap()
                    .pop_front()
                    .expect("No implicit request is allowed."),
            )
            .unwrap(),
        ))
    }
}

#[tokio::test]
async fn login_persists_only_private_session_and_signs_inventory_and_state_requests() {
    let script = Script::new(vec![
        id_response(),
        login_response(),
        json!({"code":0,"data":{"homeList":[{"homegroupId":"19"}]}}),
        json!({"code":0,"data":{"homeList":[{"roomList":[{"applianceList":[{"applianceCode":"31","type":"AC","productModel":"Synthetic AC"}]}]}]}}),
        json!({"code":0,"data":{"power":"on"}}),
    ]);
    let credentials = credentials();
    assert_eq!(format!("{credentials:?}"), "LoginCredentials([REDACTED])");
    let session = login(&script, &credentials, &operation()).await.unwrap();
    assert_eq!(format!("{session:?}"), "MeijuSession([REDACTED])");
    assert_eq!(
        session.subject().expose(),
        b"meiju.login-id.v1:synthetic-login-001"
    );
    let encoded = session.encode().unwrap();
    let stored: Value = serde_json::from_slice(encoded.expose()).unwrap();
    assert_eq!(stored["key"], vector()["session_key"]);
    for forbidden in ["account", "password", "iampwd", "deviceId"] {
        assert!(stored.get(forbidden).is_none());
    }
    let session = MeijuSession::restore(&encoded).unwrap();
    let reference = CredentialRef::new();
    let reads = session.reads(&script, reference);
    let inventory = retrieve_inventory(&reads, reference, &operation())
        .await
        .unwrap();
    assert_eq!(inventory.appliances.len(), 1);
    read_cloud_state(
        &reads,
        reference,
        &inventory.appliances[0],
        StateQuery::Default,
        &SystemClock,
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(
        reads
            .request(&CredentialRef::new(), ReadEndpoint::Homes, &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_IDENTITY_CONFLICT"
    );
    let calls = script.calls.lock().unwrap();
    assert_eq!(calls.len(), 5);
    assert_eq!(calls[0].0, CloudEndpoint::LoginId);
    assert!(!calls[0].2 && !calls[1].2);
    assert_eq!(calls[1].1["iotData"]["password"], vector()["iot_password"]);
    assert_eq!(calls[1].1["iotData"]["iampwd"], vector()["iam_password"]);
    assert_eq!(calls[1].1["iotData"]["deviceId"], vector()["device_id"]);
    assert_eq!(calls[1].1["iotData"]["iotAppId"], "900");
    assert_eq!(calls[1].1["data"]["platform"], 2);
    assert_eq!(calls[1].1["stamp"], calls[1].1["timestamp"]);
    assert_eq!(calls[1].1["stamp"], calls[1].1["iotData"]["stamp"]);
    assert_eq!(calls[3].1["homegroupId"], 19);
    assert!(calls[2..].iter().all(|call| call.2));
    assert_eq!(calls[4].1["applianceCode"], "31");
    assert_eq!(calls[4].1["command"], json!({"query":{}}));
}

#[tokio::test]
async fn invalid_login_or_challenge_stops_without_retry_or_session() {
    for response in [
        json!({"code":7400,"msg":"PRIVATE-PROVIDER-TEXT"}),
        json!({"code":40002}),
        json!({"code":123,"msg":"PRIVATE-PROVIDER-TEXT","challenge":"PRIVATE-CHALLENGE"}),
        json!({"code":0}),
        json!({"code":true}),
        json!({"code":0,"data":{"loginId":""}}),
        json!({"code":0,"data":{"loginId":"x\nprivate"}}),
    ] {
        let script = Script::new(vec![response]);
        let error = login(&script, &credentials(), &operation())
            .await
            .unwrap_err();
        assert!(!format!("{error:?}").contains("PRIVATE"));
        assert_eq!(script.calls.lock().unwrap().len(), 1);
    }
    for data in [
        json!({"mdata":{"accessToken":""},"key":vector()["wrapped_key"]}),
        json!({"mdata":{"accessToken":"token\r\nprivate"},"key":vector()["wrapped_key"]}),
        json!({"mdata":{"accessToken":"ok"},"key":"invalid"}),
        json!({"mdata":{}}),
    ] {
        let script = Script::new(vec![id_response(), json!({"code":0,"data":data})]);
        assert_eq!(
            login(&script, &credentials(), &operation())
                .await
                .unwrap_err()
                .code,
            "MIDEA_RESPONSE_INVALID"
        );
        assert_eq!(script.calls.lock().unwrap().len(), 2);
    }
}

#[test]
fn input_and_persisted_session_validation_fail_closed() {
    for (account, password) in [
        ("", "x"),
        ("owner@example.test", ""),
        ("owner\n@example.test", "x"),
        ("owner@example.test", "\u{00e9}"),
        ("owner@example.test", "\0"),
    ] {
        assert_eq!(
            LoginCredentials::new(secret(account), secret(password))
                .unwrap_err()
                .code,
            "MIDEA_LOGIN_INPUT_UNSUPPORTED"
        );
    }
    assert!(LoginCredentials::new(secret(&"a".repeat(257)), secret("x")).is_err());
    for raw in ["{}", "null", "private plaintext", &"x".repeat(65537)] {
        assert_eq!(
            MeijuSession::restore(&secret(raw)).unwrap_err().code,
            "ACCOUNT_SESSION_UNCONFIRMED"
        );
    }
    let baseline = json!({"version":1,"subject":"meiju.login-id.v1:test-id","token":"FICTIONAL-ACCESS-TOKEN","key":"synthetic-key-01"});
    for (field, value) in [
        ("version", json!(2)),
        ("subject", json!("meiju.login-id.v1:")),
        ("subject", json!("other")),
        ("token", json!("a\nb")),
        ("key", json!("short")),
        ("extra", json!(true)),
    ] {
        let mut bad = baseline.clone();
        bad[field] = value;
        assert!(
            MeijuSession::restore(&SecretBytes::new(serde_json::to_vec(&bad).unwrap())).is_err()
        );
    }
}

struct Hanging {
    calls: AtomicUsize,
    dropped: AtomicUsize,
}
struct Dropped<'a>(&'a AtomicUsize);
impl Drop for Dropped<'_> {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}
#[async_trait]
impl CloudTransport for Hanging {
    async fn send(&self, _: &SignedRequest, _: &Operation) -> Result<SecretBytes> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        let _guard = Dropped(&self.dropped);
        std::future::pending().await
    }
}

#[tokio::test]
async fn shared_deadline_and_cancellation_drop_login_without_followup_requests() {
    let transport = Hanging {
        calls: AtomicUsize::new(0),
        dropped: AtomicUsize::new(0),
    };
    let op = operation();
    op.cancellation.cancel();
    assert_eq!(
        login(&transport, &credentials(), &op)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    assert_eq!(transport.calls.load(Ordering::Relaxed), 0);
    let op = Operation::new(20, CancellationToken::new()).unwrap();
    assert_eq!(
        login(&transport, &credentials(), &op)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(transport.calls.load(Ordering::Relaxed), 1);
    assert_eq!(transport.dropped.load(Ordering::Relaxed), 1);
    let op = operation();
    let cancel = async {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        op.cancellation.cancel();
    };
    let input = credentials();
    let (result, _) = tokio::join!(login(&transport, &input, &op), cancel);
    assert_eq!(result.unwrap_err().code, "CANCELLED");
    assert_eq!(transport.calls.load(Ordering::Relaxed), 2);
    assert_eq!(transport.dropped.load(Ordering::Relaxed), 2);
}

struct Failing;
#[async_trait]
impl CloudTransport for Failing {
    async fn send(&self, _: &SignedRequest, _: &Operation) -> Result<SecretBytes> {
        Err(Error::new(
            "PRIVATE-PROVIDER-ERROR",
            "https://private.test/token",
        ))
    }
}
#[tokio::test]
async fn transport_errors_never_export_provider_details() {
    let error = login(&Failing, &credentials(), &operation())
        .await
        .unwrap_err();
    assert_eq!(error.code, "MIDEA_REQUEST_FAILED");
    assert!(!format!("{error:?}").contains("private"));
}
