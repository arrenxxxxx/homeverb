use super::*;
use crate::PropertyHistoryWindow;
use std::sync::atomic::{AtomicUsize, Ordering};
fn target(s: &ModernMijiaSession) -> ModernDirectoryEntry {
    s.directory_entry(EntryWire {
        did: PrivateString::new("fictional.s7".into()),
        name: "Sensor".into(),
        model: "linp.sensor_ht.ks1bp".into(),
        spec_type: None,
        online: None,
        pid: Some(DirectoryKind(0)),
        parent_id: None,
    })
    .unwrap()
}
fn window() -> PropertyHistoryWindow {
    PropertyHistoryWindow::new(1700000000, 1700086400).unwrap()
}
fn query() -> PropertyQuery {
    PropertyQuery {
        siid: 2,
        piid: 1001,
    }
}
struct HistoryPeer {
    mode: u8,
    calls: AtomicUsize,
    started: tokio::sync::Notify,
}
impl HistoryPeer {
    fn new(mode: u8) -> Self {
        Self {
            mode,
            calls: AtomicUsize::new(0),
            started: tokio::sync::Notify::new(),
        }
    }
}
#[async_trait]
impl ModernReadTransport for HistoryPeer {
    async fn send_modern_read(
        &self,
        r: &ModernReadRequest,
        op: &Operation,
    ) -> Result<ModernReadResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(r.endpoint(), ModernReadEndpoint::PropertyHistory);
        assert_eq!(
            r.endpoint().url(),
            "https://api.mijia.tech/app/user/get_user_device_data"
        );
        assert!(
            r.cookie()
                .starts_with("userId=12345;cUserId=fictional-c-user;")
        );
        assert!(r.cookie().contains("serviceToken=fictional-service;"));
        assert!(r.cookie().contains("PassportDeviceId="));
        assert!(!r.cookie().contains("passToken"));
        assert!(r.agent.as_str().starts_with("Android-15-11.0.701-Xiaomi-"));
        let fields: BTreeMap<_, _> = Url::parse(&format!(
            "https://example.invalid/?{}",
            std::str::from_utf8(r.body()).unwrap()
        ))
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"])?;
        let data = codec::decrypt(&key, fields["data"].as_bytes())?;
        let hash = codec::decrypt(&key, fields["rc4_hash__"].as_bytes())?;
        use sha1::Digest;
        let expected = format!(
            "POST&/user/get_user_device_data&data={}&{}",
            std::str::from_utf8(data.expose()).unwrap(),
            codec::base64(&key)
        );
        assert_eq!(
            hash.expose(),
            codec::base64(&sha1::Sha1::digest(expected.as_bytes())).as_bytes()
        );
        let signature = format!(
            "POST&/user/get_user_device_data&data={}&rc4_hash__={}&{}",
            fields["data"],
            fields["rc4_hash__"],
            codec::base64(&key)
        );
        assert_eq!(
            fields["signature"],
            codec::base64(&sha1::Sha1::digest(signature.as_bytes()))
        );
        assert_eq!(
            serde_json::from_slice::<Value>(data.expose()).unwrap(),
            json!({"did":"fictional.s7","key":"2.1001","type":"prop","time_start":1700000000,"time_end":1700086400,"limit":31})
        );
        self.started.notify_one();
        if self.mode == 9 {
            std::future::pending::<()>().await;
        }
        if self.mode == 8 {
            op.cancellation.cancel();
        }
        let reply = match self.mode {
            4 => json!({"code":-10001}),
            5 => json!({"code":0,"result":[{"did":"foreign","time":1700000000,"value":"[1]"}]}),
            _ => {
                json!({"code":0,"result":[{"did":"fictional.s7","key":"2.1001","type":"prop","time":1700000000,"value":"[-1.2]","uid":"PRIVATE-ACTOR"}]})
            }
        };
        let plain = serde_json::to_vec(&reply).unwrap();
        let bytes = match self.mode {
            2 | 3 => gzip(&plain),
            6 => gzip(&vec![b'x'; crate::MAX_RESPONSE_BYTES + 1]),
            7 => {
                let mut b = gzip(&plain);
                b.extend(gzip(&plain));
                b
            }
            10 => vec![b'x'; crate::MAX_RESPONSE_BYTES + 1],
            _ => plain,
        };
        Ok(ModernReadResponse {
            body: SecretBytes::new(if self.mode == 0 || self.mode == 4 {
                bytes
            } else {
                codec::encrypt(&key, &bytes).as_str().as_bytes().to_vec()
            }),
            encoding: if matches!(self.mode, 3 | 6 | 7) {
                ModernContentEncoding::Gzip
            } else {
                ModernContentEncoding::Plain
            },
        })
    }
}
#[tokio::test]
async fn modern_property_history_uses_community_profile_and_preserves_signed_values() {
    for mode in 0..4 {
        let p = HistoryPeer::new(mode);
        let s = session();
        let r = s
            .numeric_property_history(&p, &target(&s), &query(), window(), &operation())
            .await
            .unwrap();
        assert_eq!(r.records[0].value, -1.2);
        assert_eq!(r.unit.as_deref(), Some("celsius"));
        assert!(!format!("{r:?}").contains("PRIVATE"));
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn modern_property_history_rejects_auth_foreign_oversized_late_and_ambiguous_gzip() {
    for mode in [4, 5, 6, 7, 8, 10] {
        let p = HistoryPeer::new(mode);
        let s = session();
        let r = s
            .numeric_property_history(&p, &target(&s), &query(), window(), &operation())
            .await;
        assert!(r.is_err());
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
        if mode == 4 {
            assert_eq!(r.unwrap_err().code, "AUTH_REQUIRED");
        }
    }
}
#[tokio::test]
async fn modern_property_history_independently_refuses_subject_child_and_mapping_tampering() {
    let p = HistoryPeer::new(0);
    let s = session();
    for mode in 0..8 {
        let mut e = target(&s);
        let mut q = query();
        let op = operation();
        match mode {
            0 => e.subject = PrivateString::new("foreign".into()),
            1 => e.kind = Some(DirectoryKind(21)),
            2 => e.mapping = None,
            3 => q.piid = 999,
            4 => e.mapping.as_mut().unwrap().properties[0].definition["unit"] = json!("fake"),
            5 => op.cancellation.cancel(),
            6 => e.model = "foreign.sensor".into(),
            _ => e.encrypted_subject = PrivateString::new("foreign".into()),
        };
        assert!(
            s.numeric_property_history(&p, &e, &q, window(), &op)
                .await
                .is_err()
        );
    }
    let fan = s
        .directory_entry(EntryWire {
            did: PrivateString::new("fictional.s7".into()),
            name: "Fan".into(),
            model: "jdjz.fan.2072fc".into(),
            spec_type: None,
            online: None,
            pid: Some(DirectoryKind(0)),
            parent_id: None,
        })
        .unwrap();
    assert!(
        s.numeric_property_history(
            &p,
            &fan,
            &PropertyQuery { siid: 2, piid: 1 },
            window(),
            &operation()
        )
        .await
        .is_err()
    );
    assert_eq!(p.calls.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn modern_property_history_pending_cancel_deadline_and_drop_do_not_retry() {
    for mode in 0..3 {
        let p = HistoryPeer::new(9);
        let s = session();
        let e = target(&s);
        let q = query();
        let op =
            Operation::new(if mode == 1 { 25 } else { 1000 }, CancellationToken::new()).unwrap();
        let mut work = Box::pin(s.numeric_property_history(&p, &e, &q, window(), &op));
        tokio::select! {_ = p.started.notified()=>{},_=&mut work=>panic!("Expected pending history")};
        if mode == 0 {
            op.cancellation.cancel();
            assert_eq!(work.await.unwrap_err().code, "CANCELLED");
        } else if mode == 1 {
            assert_eq!(work.await.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            drop(work);
        }
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
    }
}
