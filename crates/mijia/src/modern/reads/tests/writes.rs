use super::*;
use crate::{ModernControlRequest, ModernControlResponse, ModernControlTransport, PropertyWrite};
use homeverb_core::WriteProgress;
use homeverb_domain::Phase;
use std::sync::atomic::{AtomicUsize, Ordering};

fn target(s: &ModernMijiaSession) -> ModernDirectoryEntry {
    s.directory_entry(EntryWire {
        did: PrivateString::new("fictional-root.s7".into()),
        name: "Fan".into(),
        model: "jdjz.fan.2072fc".into(),
        spec_type: None,
        online: None,
        pid: Some(DirectoryKind(0)),
        parent_id: None,
    })
    .unwrap()
}
fn write() -> PropertyWrite {
    PropertyWrite {
        query: PropertyQuery { siid: 2, piid: 1 },
        value: json!(false),
    }
}
struct Writer {
    mode: u8,
    calls: AtomicUsize,
}
impl Writer {
    fn new(mode: u8) -> Self {
        Self {
            mode,
            calls: AtomicUsize::new(0),
        }
    }
}
#[async_trait]
impl ModernControlTransport for Writer {
    async fn send_modern_control(
        &self,
        request: &ModernControlRequest,
        progress: &WriteProgress,
        op: &Operation,
    ) -> Result<ModernControlResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            request.url(),
            "https://core.api.mijia.tech/app/miotspec/prop/set"
        );
        assert_eq!(format!("{request:?}"), "ModernControlRequest([REDACTED])");
        assert_eq!(
            request.cookie(),
            "cUserId=fictional-c-user; serviceToken=fictional-service; yetAnotherServiceToken=fictional-service; countryCode=CN"
        );
        let fields: BTreeMap<_, _> = Url::parse(&format!(
            "https://example.invalid/?{}",
            std::str::from_utf8(request.body()).unwrap()
        ))
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"]).unwrap();
        let mut joint = codec::unbase64(&fields["data"]).unwrap();
        let split = joint.len();
        joint.extend_from_slice(&codec::unbase64(&fields["rc4_hash__"]).unwrap());
        let decoded = codec::decrypt(&key, codec::base64(&joint).as_bytes()).unwrap();
        let body: Value = serde_json::from_slice(&decoded.expose()[..split]).unwrap();
        assert_eq!(
            body,
            json!({"params":[{"did":"fictional-root.s7","siid":2,"piid":1,"value":false}]})
        );
        use sha1::Digest;
        let hash = format!(
            "POST&{}&data={}&{}",
            request.path(),
            std::str::from_utf8(&decoded.expose()[..split]).unwrap(),
            codec::base64(&key)
        );
        assert_eq!(
            &decoded.expose()[split..],
            codec::base64(&sha1::Sha1::digest(hash.as_bytes())).as_bytes()
        );
        let signature = format!(
            "POST&{}&data={}&rc4_hash__={}&{}",
            request.path(),
            fields["data"],
            fields["rc4_hash__"],
            codec::base64(&key)
        );
        assert_eq!(
            fields["signature"],
            codec::base64(&sha1::Sha1::digest(signature.as_bytes()))
        );
        if self.mode == 16 {
            return Err(crate::failed());
        }
        if self.mode != 10 {
            progress.mark_sent();
        }
        if self.mode == 9 {
            return Err(crate::failed());
        }
        if self.mode == 15 {
            std::future::pending::<()>().await;
        }
        let mut response =
            json!({"code":0,"result":[{"did":"fictional-root.s7","siid":2,"piid":1,"code":0}]});
        match self.mode {
            1 => response["result"][0]["code"] = json!(-4001),
            2 => response["result"][0]["did"] = json!("foreign"),
            3 => response["result"][0]["siid"] = json!(7),
            4 => response["result"][0]["piid"] = json!(7),
            5 => {
                let item = response["result"][0].clone();
                response["result"].as_array_mut().unwrap().push(item);
            }
            6 => response["result"] = json!([]),
            7 => response["code"] = json!(-10001),
            _ => {}
        }
        let plain = serde_json::to_vec(&response).unwrap();
        let bytes = match self.mode {
            12 => gzip(&plain),
            13 => vec![31, 139, 0],
            14 => vec![b'x'; crate::MAX_RESPONSE_BYTES + 1],
            17 => gzip(&vec![b'x'; crate::MAX_RESPONSE_BYTES + 1]),
            18 => {
                let mut bytes = gzip(&plain);
                bytes.extend(gzip(&plain));
                bytes
            }
            _ => plain,
        };
        if self.mode == 11 {
            op.cancellation.cancel();
        }
        Ok(ModernControlResponse {
            body: SecretBytes::new(if self.mode == 8 {
                bytes
            } else {
                codec::encrypt(&key, &bytes).as_str().as_bytes().to_vec()
            }),
            encoding: if matches!(self.mode, 12 | 13 | 17 | 18) {
                ModernContentEncoding::Gzip
            } else {
                ModernContentEncoding::Plain
            },
        })
    }
}
#[tokio::test]
async fn modern_write_correlates_acceptance_and_rejection_without_verification() {
    for mode in [0, 1, 12] {
        let s = session();
        let p = Writer::new(mode);
        let progress = WriteProgress::default();
        let result = s
            .set_property(&p, &target(&s), &write(), &progress, &operation())
            .await
            .unwrap();
        assert_eq!(
            result.phase,
            if mode == 1 {
                Phase::Rejected
            } else {
                Phase::Accepted
            }
        );
        assert_eq!(
            progress.phase(),
            if mode == 1 {
                Phase::PossiblySent
            } else {
                Phase::Accepted
            }
        );
        assert!(result.completion.is_none());
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn modern_write_bad_or_late_replies_never_prove_rejection_or_retry() {
    for mode in (2..12).chain([13, 14, 17, 18]) {
        let s = session();
        let p = Writer::new(mode);
        let progress = WriteProgress::default();
        assert!(
            s.set_property(&p, &target(&s), &write(), &progress, &operation())
                .await
                .is_err()
        );
        assert_eq!(
            progress.phase(),
            if mode == 10 {
                Phase::NotSent
            } else {
                Phase::PossiblySent
            }
        );
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn modern_write_refuses_unreviewed_targets_subjects_domains_and_existing_progress_before_io()
{
    let s = session();
    let p = Writer::new(0);
    for mode in 0..9 {
        let mut e = target(&s);
        let mut w = write();
        let progress = WriteProgress::default();
        let op = operation();
        match mode {
            0 => e.subject = PrivateString::new("54321".into()),
            1 => e.encrypted_subject = PrivateString::new("foreign".into()),
            2 => e.kind = Some(DirectoryKind(21)),
            3 => e.mapping = None,
            4 => w.query.piid = 999,
            5 => w.value = json!(2),
            6 => w.value = json!({"state":false}),
            7 => progress.mark_sent(),
            _ => op.cancellation.cancel(),
        }
        assert!(s.set_property(&p, &e, &w, &progress, &op).await.is_err());
    }
    assert_eq!(p.calls.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn modern_write_preserves_pre_send_failure_and_pending_deadline_progress() {
    let s = session();
    let e = target(&s);
    for mode in [15, 16] {
        let p = Writer::new(mode);
        let progress = WriteProgress::default();
        let op = Operation::new(25, CancellationToken::new()).unwrap();
        assert!(
            s.set_property(&p, &e, &write(), &progress, &op)
                .await
                .is_err()
        );
        assert_eq!(
            progress.phase(),
            if mode == 15 {
                Phase::PossiblySent
            } else {
                Phase::NotSent
            }
        );
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
    }
}
