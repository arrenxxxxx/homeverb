use super::*;
use crate::{CloudRequest, CloudResponse, Region};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_util::sync::CancellationToken;

fn session() -> MijiaSession {
    MijiaSession {
        region: Region::Cn,
        user_id: PrivateString::new("12345".into()),
        security: PrivateString::new("AAECAwQFBgcICQoLDA0ODw==".into()),
        token: PrivateString::new("fictional-token".into()),
    }
}
fn device() -> CloudDevice {
    CloudDevice {
        did: PrivateString::new("fictional-fan".into()),
        name: "Fictional fan".into(),
        model: "fictional.fan.v1".into(),
        spec_type: None,
        online_hint: None,
    }
}
fn write() -> PropertyWrite {
    PropertyWrite {
        query: PropertyQuery { siid: 2, piid: 1 },
        value: json!(false),
    }
}
fn operation() -> Operation {
    Operation::new(5000, CancellationToken::new()).unwrap()
}
fn ack(code: i64) -> Value {
    json!({"code":0,"result":[{"did":"fictional-fan","siid":2,"piid":1,"code":code}]})
}
struct Script {
    reply: Value,
    calls: AtomicUsize,
    plaintext: bool,
    stall: bool,
}
impl Script {
    fn new(reply: Value) -> Self {
        Self {
            reply,
            calls: AtomicUsize::new(0),
            plaintext: false,
            stall: false,
        }
    }
}
#[async_trait]
impl CloudTransport for Script {
    async fn send(&self, _: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        panic!("A write must never use the read transport.");
    }
    async fn send_control(
        &self,
        request: &CloudRequest,
        progress: &WriteProgress,
        operation: &Operation,
    ) -> Result<CloudResponse> {
        operation
            .run(async {
                assert_eq!(request.endpoint(), CloudEndpoint::SetProperty);
                let url = reqwest::Url::parse(&format!(
                    "https://localhost/?{}",
                    std::str::from_utf8(request.body()).unwrap()
                ))
                .unwrap();
                let fields: std::collections::HashMap<_, _> =
                    url.query_pairs().into_owned().collect();
                let key =
                    codec::signed_nonce(session().security.as_str(), &fields["_nonce"]).unwrap();
                let data = codec::decrypt(&key, fields["data"].as_bytes()).unwrap();
                assert_eq!(
                    serde_json::from_slice::<Value>(data.expose()).unwrap(),
                    json!({"params":[{"did":"fictional-fan","siid":2,"piid":1,"value":false}]})
                );
                self.calls.fetch_add(1, Ordering::SeqCst);
                progress.mark_sent();
                if self.stall {
                    std::future::pending::<()>().await;
                }
                let bytes = serde_json::to_vec(&self.reply).unwrap();
                let bytes = if self.plaintext {
                    bytes
                } else {
                    codec::encrypt(&key, &bytes).as_str().as_bytes().to_vec()
                };
                Ok(CloudResponse {
                    body: SecretBytes::new(bytes),
                    service_token: None,
                })
            })
            .await
    }
}

#[tokio::test]
async fn exact_ack_is_accepted_and_never_verified() {
    let transport = Script::new(ack(0));
    let progress = WriteProgress::default();
    let result = session()
        .set_property(&transport, &device(), &write(), &progress, &operation())
        .await
        .unwrap();
    assert_eq!(result.phase, Phase::Accepted);
    assert_eq!(progress.phase(), Phase::Accepted);
    assert!(result.completion.is_none());
    assert!(result.error.is_none());
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    assert!(
        session()
            .set_property(&transport, &device(), &write(), &progress, &operation())
            .await
            .is_err()
    );
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn exact_rejection_has_no_completion_and_no_retry() {
    let transport = Script::new(ack(-704220043));
    let progress = WriteProgress::default();
    let result = session()
        .set_property(&transport, &device(), &write(), &progress, &operation())
        .await
        .unwrap();
    assert_eq!(result.phase, Phase::Rejected);
    assert_eq!(result.error.unwrap().code, "MIJIA_REQUEST_REJECTED");
    assert!(result.completion.is_none());
    assert_eq!(progress.phase(), Phase::PossiblySent);
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn global_plaintext_duplicate_missing_and_foreign_acks_stay_uncertain() {
    let mut replies = vec![
        json!({"code":0}),
        json!({"code":0,"result":[]}),
        json!({"code":0,"result":[ack(0)["result"][0].clone(),ack(0)["result"][0].clone()]}),
    ];
    for (field, value) in [
        ("did", json!("another-device")),
        ("siid", json!(3)),
        ("piid", json!(2)),
        ("code", json!("0")),
    ] {
        let mut reply = ack(0);
        reply["result"][0][field] = value;
        replies.push(reply);
    }
    let mut reply = ack(0);
    reply["code"] = json!(-3);
    replies.push(reply);
    for reply in replies {
        let transport = Script::new(reply);
        let progress = WriteProgress::default();
        assert!(
            session()
                .set_property(&transport, &device(), &write(), &progress, &operation())
                .await
                .is_err()
        );
        assert_eq!(progress.phase(), Phase::PossiblySent);
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    }
    let mut transport = Script::new(ack(0));
    transport.plaintext = true;
    let progress = WriteProgress::default();
    assert!(
        session()
            .set_property(&transport, &device(), &write(), &progress, &operation())
            .await
            .is_err()
    );
    assert_eq!(progress.phase(), Phase::PossiblySent);
}

#[tokio::test]
async fn invalid_and_cancelled_preflight_never_send() {
    let transport = Script::new(ack(0));
    let mut invalid = write();
    invalid.query.siid = 0;
    for value in [
        invalid,
        PropertyWrite {
            value: json!("private"),
            ..write()
        },
        PropertyWrite {
            value: Value::Null,
            ..write()
        },
    ] {
        let progress = WriteProgress::default();
        assert!(
            session()
                .set_property(&transport, &device(), &value, &progress, &operation())
                .await
                .is_err()
        );
        assert_eq!(progress.phase(), Phase::NotSent);
    }
    let op = operation();
    op.cancellation.cancel();
    let progress = WriteProgress::default();
    assert_eq!(
        session()
            .set_property(&transport, &device(), &write(), &progress, &op)
            .await
            .err()
            .unwrap()
            .code,
        "CANCELLED"
    );
    assert_eq!(progress.phase(), Phase::NotSent);
    assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
}

struct ReadOnly;
#[async_trait]
impl CloudTransport for ReadOnly {
    async fn send(&self, _: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        panic!("No fallback.");
    }
}

#[tokio::test]
async fn default_transport_denies_without_falling_back() {
    let progress = WriteProgress::default();
    assert_eq!(
        session()
            .set_property(&ReadOnly, &device(), &write(), &progress, &operation())
            .await
            .err()
            .unwrap()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(progress.phase(), Phase::NotSent);
}

#[tokio::test]
async fn cancellation_after_send_preserves_uncertainty() {
    let mut transport = Script::new(ack(0));
    transport.stall = true;
    let progress = WriteProgress::default();
    let op = operation();
    let session = session();
    let device = device();
    let write = write();
    let send = session.set_property(&transport, &device, &write, &progress, &op);
    tokio::pin!(send);
    tokio::select! { _ = &mut send => panic!("Expected pending send"), _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => {} }
    assert_eq!(progress.phase(), Phase::PossiblySent);
    op.cancellation.cancel();
    assert_eq!(send.await.err().unwrap().code, "CANCELLED");
    assert_eq!(progress.phase(), Phase::PossiblySent);
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
}
