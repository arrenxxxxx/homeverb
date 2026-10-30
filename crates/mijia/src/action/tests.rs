use super::*;
use crate::{
    CloudRequest, CloudResponse, ModernContentEncoding, ModernControlRequest,
    ModernControlResponse, ModernControlTransport,
};
use async_trait::async_trait;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio_util::sync::CancellationToken;

pub(crate) const DID: &str = "fictional-root.s7";
pub(crate) const SECURITY: &str = "AAECAwQFBgcICQoLDA0ODw==";
pub(crate) fn operation() -> Operation {
    Operation::new(5000, CancellationToken::new()).unwrap()
}
pub(crate) fn profiles() -> Vec<(String, String, ActionQuery)> {
    let profiles: Vec<Profile> = crate::decode(include_bytes!("../action_profiles.json")).unwrap();
    profiles
        .into_iter()
        .map(|p| {
            (
                p.model,
                p.spec_type,
                ActionQuery {
                    siid: p.siid,
                    aiid: p.aiid,
                },
            )
        })
        .collect()
}
fn session() -> MijiaSession {
    MijiaSession {
        region: Region::Cn,
        user_id: PrivateString::new("12345".into()),
        security: PrivateString::new(SECURITY.into()),
        token: PrivateString::new("fictional-token".into()),
    }
}
fn device(model: &str, specification: &str) -> CloudDevice {
    CloudDevice {
        did: PrivateString::new(DID.into()),
        name: "Fictional action target".into(),
        model: model.into(),
        spec_type: Some(specification.into()),
        online_hint: None,
    }
}
pub(crate) fn ack(query: ActionQuery, code: i64) -> Value {
    json!({"code":0,"result":{"did":DID,"siid":query.siid,"aiid":query.aiid,"code":code,"out":[]}})
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fault {
    None,
    Plaintext,
    NoMarker,
    BeforeSend,
    AfterSend,
    CancelReply,
    DeadlineReply,
    Stall,
}
pub(crate) struct Peer {
    pub query: ActionQuery,
    pub reply: Vec<u8>,
    pub encoding: ModernContentEncoding,
    pub fault: Fault,
    pub calls: AtomicUsize,
    pub active: AtomicUsize,
    pub started: tokio::sync::Notify,
}
impl Peer {
    pub fn new(query: ActionQuery, reply: Value) -> Self {
        Self::raw(query, serde_json::to_vec(&reply).unwrap())
    }
    pub fn raw(query: ActionQuery, reply: Vec<u8>) -> Self {
        Self {
            query,
            reply,
            encoding: ModernContentEncoding::Plain,
            fault: Fault::None,
            calls: AtomicUsize::new(0),
            active: AtomicUsize::new(0),
            started: tokio::sync::Notify::new(),
        }
    }
    pub fn count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
    async fn respond(
        &self,
        body: &[u8],
        path: &str,
        modern: bool,
        progress: &WriteProgress,
        op: &Operation,
    ) -> Result<SecretBytes> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(path, "/miotspec/action");
        let fields: BTreeMap<_, _> = reqwest::Url::parse(&format!(
            "https://example.invalid/?{}",
            std::str::from_utf8(body).unwrap()
        ))
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
        let key = codec::signed_nonce(SECURITY, &fields["_nonce"]).unwrap();
        let (data, hash) = if modern {
            assert_eq!(fields["ssecurity"], SECURITY);
            let mut joined = codec::unbase64(&fields["data"]).unwrap();
            let split = joined.len();
            joined.extend_from_slice(&codec::unbase64(&fields["rc4_hash__"]).unwrap());
            let decoded = codec::decrypt(&key, codec::base64(&joined).as_bytes()).unwrap();
            (
                decoded.expose()[..split].to_vec(),
                decoded.expose()[split..].to_vec(),
            )
        } else {
            assert_eq!(fields["ssecurity"], SECURITY);
            (
                codec::decrypt(&key, fields["data"].as_bytes())
                    .unwrap()
                    .expose()
                    .to_vec(),
                codec::decrypt(&key, fields["rc4_hash__"].as_bytes())
                    .unwrap()
                    .expose()
                    .to_vec(),
            )
        };
        assert_eq!(
            serde_json::from_slice::<Value>(&data).unwrap(),
            json!({"params":{"did":DID,"siid":self.query.siid,"aiid":self.query.aiid,"in":[]}})
        );
        use sha1::Digest;
        let plain_hash = format!(
            "POST&{path}&data={}&{}",
            std::str::from_utf8(&data).unwrap(),
            codec::base64(&key)
        );
        assert_eq!(
            hash,
            codec::base64(&sha1::Sha1::digest(plain_hash.as_bytes())).as_bytes()
        );
        let sign = format!(
            "POST&{path}&data={}&rc4_hash__={}&{}",
            fields["data"],
            fields["rc4_hash__"],
            codec::base64(&key)
        );
        assert_eq!(
            fields["signature"],
            codec::base64(&sha1::Sha1::digest(sign.as_bytes()))
        );
        if self.fault == Fault::BeforeSend {
            return Err(crate::failed());
        }
        if self.fault != Fault::NoMarker {
            progress.mark_sent();
        }
        if self.fault == Fault::AfterSend {
            return Err(crate::failed());
        }
        if self.fault == Fault::Stall {
            struct Active<'a>(&'a AtomicUsize);
            impl Drop for Active<'_> {
                fn drop(&mut self) {
                    self.0.fetch_sub(1, Ordering::SeqCst);
                }
            }
            self.active.fetch_add(1, Ordering::SeqCst);
            let _guard = Active(&self.active);
            self.started.notify_one();
            std::future::pending::<()>().await;
        }
        if self.fault == Fault::CancelReply {
            op.cancellation.cancel();
        }
        if self.fault == Fault::DeadlineReply {
            tokio::time::sleep_until(op.deadline).await;
        }
        Ok(SecretBytes::new(if self.fault == Fault::Plaintext {
            self.reply.clone()
        } else {
            codec::encrypt(&key, &self.reply)
                .as_str()
                .as_bytes()
                .to_vec()
        }))
    }
}
#[async_trait]
impl CloudTransport for Peer {
    async fn send(&self, _: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        panic!("Actions must never use a read transport");
    }
    async fn send_control(
        &self,
        r: &CloudRequest,
        p: &WriteProgress,
        op: &Operation,
    ) -> Result<CloudResponse> {
        assert_eq!(r.endpoint(), CloudEndpoint::Action);
        assert_eq!(r.region(), Region::Cn);
        assert!(
            r.cookie()
                .starts_with("userId=12345; serviceToken=fictional-token;")
        );
        assert!(!r.cookie().contains("cUserId"));
        assert_eq!(format!("{r:?}"), "CloudRequest([REDACTED])");
        Ok(CloudResponse {
            body: self
                .respond(r.body(), r.endpoint().path(), false, p, op)
                .await?,
            service_token: None,
        })
    }
}
#[async_trait]
impl ModernControlTransport for Peer {
    async fn send_modern_control(
        &self,
        r: &ModernControlRequest,
        p: &WriteProgress,
        op: &Operation,
    ) -> Result<ModernControlResponse> {
        assert_eq!(r.url(), "https://core.api.mijia.tech/app/miotspec/action");
        assert_eq!(
            r.cookie(),
            "cUserId=fictional-c-user; serviceToken=fictional-service; yetAnotherServiceToken=fictional-service; countryCode=CN"
        );
        assert_eq!(format!("{r:?}"), "ModernControlRequest([REDACTED])");
        Ok(ModernControlResponse {
            body: self.respond(r.body(), r.path(), true, p, op).await?,
            encoding: self.encoding,
        })
    }
}

pub(crate) fn outcomes() -> Vec<(i64, Option<Phase>)> {
    vec![
        (0, Some(Phase::Accepted)),
        (-4003, Some(Phase::Rejected)),
        (-4006, Some(Phase::Rejected)),
        (-4007, Some(Phase::Rejected)),
        (-704220005, Some(Phase::Rejected)),
        (-704220008, Some(Phase::Rejected)),
        (-704220025, Some(Phase::Rejected)),
        (-704220035, Some(Phase::Rejected)),
        (1, None),
        (-4004, None),
        (-704220015, None),
        (-704220034, None),
        (-704220036, None),
        (-704220901, None),
        (-704220999, None),
        (-7000000050, None),
        (-70422005, None),
        (704220005, None),
        (i64::MIN, None),
    ]
}
pub(crate) fn assert_outcome(
    result: Result<AdapterOutcome>,
    progress: &WriteProgress,
    expected: Option<Phase>,
) {
    if let Some(phase) = expected {
        let result = result.unwrap();
        assert_eq!(result.phase, phase);
        assert!(result.completion.is_none());
        assert_eq!(
            result.error.as_ref().map(|e| e.code.as_str()),
            if phase == Phase::Rejected {
                Some("MIJIA_REQUEST_REJECTED")
            } else {
                None
            }
        );
    } else {
        assert!(result.is_err());
    }
    assert_eq!(
        progress.phase(),
        if expected == Some(Phase::Accepted) {
            Phase::Accepted
        } else {
            Phase::PossiblySent
        }
    );
}
pub(crate) fn bad_replies(query: ActionQuery) -> Vec<Vec<u8>> {
    let mut values = vec![
        json!({"code":0}),
        json!({"code":0,"result":null}),
        json!({"code":0,"result":[ack(query,0)["result"].clone()]}),
        json!({"code":0,"result":{}}),
    ];
    for (field, value) in [
        ("did", json!("foreign")),
        ("siid", json!(99)),
        ("aiid", json!(99)),
        ("code", json!("0")),
        ("out", json!([1])),
        ("out", json!(null)),
        ("out", json!({})),
    ] {
        let mut reply = ack(query, 0);
        reply["result"][field] = value;
        values.push(reply);
    }
    for code in [-3, -10001, -4003] {
        let mut reply = ack(query, 0);
        reply["code"] = json!(code);
        values.push(reply);
    }
    let mut replies: Vec<_> = values
        .iter()
        .map(|v| serde_json::to_vec(v).unwrap())
        .collect();
    replies.push(br#"{"code":0,"code":0,"result":{}}"#.to_vec());
    let good = serde_json::to_string(&ack(query, 0)).unwrap();
    replies.push(
        good.replace("\"aiid\":", "\"aiid\":1,\"aiid\":")
            .into_bytes(),
    );
    replies.push(b"invalid".to_vec());
    replies.push(vec![b'x'; crate::MAX_RESPONSE_BYTES + 1]);
    replies
}
#[tokio::test]
async fn every_pinned_action_preserves_exact_wire_identity_and_never_replays() {
    assert_eq!(profiles().len(), 11);
    for (model, spec, query) in profiles() {
        for (code, expected) in outcomes() {
            let peer = Peer::new(query, ack(query, code));
            let progress = WriteProgress::default();
            let s = session();
            let d = device(&model, &spec);
            assert_outcome(
                s.invoke_action(&peer, &d, &query, &progress, &operation())
                    .await,
                &progress,
                expected,
            );
            assert!(
                s.invoke_action(&peer, &d, &query, &progress, &operation())
                    .await
                    .is_err()
            );
            assert_eq!(peer.count(), 1);
        }
        let mut reply = ack(query, 0);
        reply["result"].as_object_mut().unwrap().remove("out");
        let peer = Peer::new(query, reply);
        let progress = WriteProgress::default();
        assert_outcome(
            session()
                .invoke_action(
                    &peer,
                    &device(&model, &spec),
                    &query,
                    &progress,
                    &operation(),
                )
                .await,
            &progress,
            Some(Phase::Accepted),
        );
    }
}
#[tokio::test]
async fn malformed_private_foreign_and_ambiguous_replies_remain_uncertain() {
    let (model, spec, query) = profiles().remove(0);
    for reply in bad_replies(query) {
        let peer = Peer::raw(query, reply);
        let p = WriteProgress::default();
        assert_outcome(
            session()
                .invoke_action(&peer, &device(&model, &spec), &query, &p, &operation())
                .await,
            &p,
            None,
        );
        assert_eq!(peer.count(), 1);
    }
    for fault in [
        Fault::Plaintext,
        Fault::NoMarker,
        Fault::BeforeSend,
        Fault::AfterSend,
        Fault::CancelReply,
        Fault::DeadlineReply,
    ] {
        let mut peer = Peer::new(query, ack(query, 0));
        peer.fault = fault;
        let p = WriteProgress::default();
        assert!(
            session()
                .invoke_action(
                    &peer,
                    &device(&model, &spec),
                    &query,
                    &p,
                    &Operation::new(20, CancellationToken::new()).unwrap()
                )
                .await
                .is_err()
        );
        assert_eq!(
            p.phase(),
            if matches!(fault, Fault::NoMarker | Fault::BeforeSend) {
                Phase::NotSent
            } else {
                Phase::PossiblySent
            }
        );
        assert_eq!(peer.count(), 1);
    }
}
#[tokio::test]
async fn wrong_model_spec_query_region_and_expired_work_refuse_before_io() {
    let (model, spec, query) = profiles().remove(0);
    let peer = Peer::new(query, ack(query, 0));
    for mode in 0..11 {
        let mut s = session();
        let mut d = device(&model, &spec);
        let mut q = query;
        let p = WriteProgress::default();
        let mut op = operation();
        match mode {
            0 => d.model = "unreviewed.light".into(),
            1 => d.spec_type = None,
            2 => d.spec_type = Some(spec.replace("0000C802", "0000C803")),
            3 => q.siid = 99,
            4 => q.aiid = 99,
            5 => s.region = Region::De,
            6 => d.did = PrivateString::new("".into()),
            7 => p.mark_sent(),
            8 => p.mark_accepted(),
            9 => op.cancellation.cancel(),
            _ => op.deadline = tokio::time::Instant::now(),
        }
        let before = p.phase();
        assert!(s.invoke_action(&peer, &d, &q, &p, &op).await.is_err());
        assert_eq!(p.phase(), before);
    }
    assert_eq!(peer.count(), 0);
}
#[tokio::test]
async fn cancellation_deadline_and_drop_release_pending_action_without_replay() {
    let (model, spec, query) = profiles().remove(0);
    let s = session();
    let d = device(&model, &spec);
    for mode in 0..3 {
        let mut peer = Peer::new(query, ack(query, 0));
        peer.fault = Fault::Stall;
        let p = WriteProgress::default();
        let op =
            Operation::new(if mode == 1 { 40 } else { 5000 }, CancellationToken::new()).unwrap();
        let mut work = Box::pin(s.invoke_action(&peer, &d, &query, &p, &op));
        tokio::select! { _=peer.started.notified()=>{},_=&mut work=>panic!("Expected pending action") }
        assert_eq!(peer.active.load(Ordering::SeqCst), 1);
        if mode == 0 {
            op.cancellation.cancel();
            assert_eq!(work.await.err().unwrap().code, "CANCELLED");
        } else if mode == 1 {
            assert_eq!(work.await.err().unwrap().code, "DEADLINE_EXCEEDED");
        } else {
            drop(work);
        }
        assert_eq!(peer.active.load(Ordering::SeqCst), 0);
        assert_eq!(p.phase(), Phase::PossiblySent);
        assert!(
            s.invoke_action(&peer, &d, &query, &p, &operation())
                .await
                .is_err()
        );
        assert_eq!(peer.count(), 1);
    }
}
struct ReadOnly;
#[async_trait]
impl CloudTransport for ReadOnly {
    async fn send(&self, _: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        panic!("No read fallback");
    }
}
#[tokio::test]
async fn unsupported_transport_and_read_helper_cannot_send_actions() {
    let (model, spec, query) = profiles().remove(0);
    let p = WriteProgress::default();
    let s = session();
    assert_eq!(
        s.invoke_action(&ReadOnly, &device(&model, &spec), &query, &p, &operation())
            .await
            .err()
            .unwrap()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert!(
        s.request(
            &ReadOnly,
            CloudEndpoint::Action,
            &SecretBytes::new(vec![]),
            &operation()
        )
        .await
        .is_err()
    );
    assert_eq!(p.phase(), Phase::NotSent);
}
