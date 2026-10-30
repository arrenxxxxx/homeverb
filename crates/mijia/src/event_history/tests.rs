use super::*;
use crate::{CloudRequest, CloudResponse, PrivateString, codec};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_util::sync::CancellationToken;
fn window() -> EventHistoryWindow {
    EventHistoryWindow::new(1700000000, 1700086400).unwrap()
}
fn query() -> EventHistoryQuery {
    EventHistoryQuery { siid: 2, eiid: 1 }
}
fn row(time: Value) -> Value {
    json!({"did":"fictional.s7","key":"2.1","type":"event","time":time,"value":"PRIVATE-OPAQUE","uid":"PRIVATE-ACTOR"})
}
fn decode(rows: Vec<Value>) -> Result<ReportedEventHistory> {
    decode_history(
        &serde_json::to_vec(&json!({"code":0,"result":rows})).unwrap(),
        "fictional.s7",
        &query(),
        window(),
        Utc::now(),
    )
}
fn op() -> Operation {
    Operation::new(5000, CancellationToken::new()).unwrap()
}
#[test]
fn exact_catalog_has_all_78_events_and_rejects_foreign_queries() {
    let catalog = profiles().unwrap();
    assert_eq!(catalog.len(), 78);
    let mut seen = std::collections::HashSet::new();
    for p in catalog {
        assert!(seen.insert((p.model.clone(), p.siid, p.eiid)));
        let q = EventHistoryQuery {
            siid: p.siid,
            eiid: p.eiid,
        };
        admit(&p.model, &p.spec_type, &q).unwrap();
        assert!(admit("foreign.model", &p.spec_type, &q).is_err());
        assert!(
            admit(
                &p.model,
                "urn:miot-spec-v2:device:switch:0000A003:foreign:1",
                &q
            )
            .is_err()
        );
        assert!(
            admit(
                &p.model,
                &p.spec_type,
                &EventHistoryQuery {
                    siid: 0,
                    eiid: p.eiid
                }
            )
            .is_err()
        );
        assert!(
            admit(
                &p.model,
                &p.spec_type,
                &EventHistoryQuery {
                    siid: p.siid,
                    eiid: 999999
                }
            )
            .is_err()
        );
    }
}
#[test]
fn pinned_custom_namespaces_keep_exact_identity_without_expanding_shared_urn_admission() {
    let profiles = profiles().unwrap();
    let namespaces: std::collections::BTreeSet<_> = profiles
        .iter()
        .flat_map(|p| [&p.service_type, &p.event_type])
        .map(|urn| urn.split(':').nth(1).unwrap())
        .collect();
    assert_eq!(
        namespaces,
        ["cuco-spec", "madv-spec", "miot-spec-v2", "xiaomi-spec"].into()
    );
    for p in profiles {
        for (value, category) in [(&p.service_type, "service"), (&p.event_type, "event")] {
            let original = value.clone();
            validate_event_urn(value, category).unwrap();
            assert_eq!(*value, original);
            if !value.starts_with("urn:miot-spec-v2:") {
                assert!(crate::spec::urn(value, category).is_err());
            }
            let mut parts: Vec<_> = value.split(':').collect();
            parts[1] = "foreign-spec";
            assert!(validate_event_urn(&parts.join(":"), category).is_err());
        }
    }
    for value in [
        "urn:cuco-spec:event:open-event:00005001:cuco-v3:0",
        "urn:cuco-spec:event:open-event:invalid:cuco-v3:1",
        "urn:cuco-spec:event:Open:00005001:cuco-v3:1",
        "urn:cuco-spec:property:open-event:00005001:cuco-v3:1",
        "urn:CUCO-SPEC:event:open-event:00005001:cuco-v3:1",
        "urn:cuco-spec:event:open-event:00005001:cuco-v3:1:extra",
    ] {
        assert!(validate_event_urn(value, "event").is_err());
    }
}
#[test]
fn occurrences_omit_payloads_and_keep_equal_timestamps_without_identity_claims() {
    let r = decode(vec![
        row(json!(1700000002)),
        row(json!("1700000000")),
        row(json!(1700000000)),
    ])
    .unwrap();
    assert_eq!(
        r.records
            .iter()
            .map(|v| v.reported_at.timestamp())
            .collect::<Vec<_>>(),
        vec![1700000000, 1700000000, 1700000002]
    );
    assert_eq!(r.rejected_records, 0);
    assert!(!r.limit_reached);
    assert_eq!(r.completeness, crate::StatisticsCompleteness::Unproven);
    assert!(!format!("{r:?}").contains("PRIVATE"));
    let r = decode(vec![]).unwrap();
    assert!(r.records.is_empty());
    assert_eq!(r.completeness, crate::StatisticsCompleteness::Unproven);
}
#[test]
fn malformed_times_missing_values_and_nonrecords_are_rejected_without_promoting_state() {
    let mut rows = vec![Value::Null, json!([]), json!(7)];
    for time in [
        json!(0),
        json!(-1),
        json!(1700086400),
        json!(1700000000.1),
        json!("1e9"),
        json!("+1700000000"),
        json!("999999999999999999"),
        Value::Null,
    ] {
        rows.push(row(time));
    }
    let mut missing = row(json!(1700000000));
    missing.as_object_mut().unwrap().remove("value");
    rows.push(missing);
    let rejected = rows.len();
    rows.push(row(json!(1700000001)));
    let result = decode(rows).unwrap();
    assert_eq!(result.rejected_records, rejected);
    assert_eq!(result.records.len(), 1);
}
#[test]
fn missing_or_foreign_correlations_refuse_the_whole_page() {
    for key in ["did", "key", "type"] {
        for value in [
            None,
            Some(Value::Null),
            Some(json!("foreign")),
            Some(json!(0)),
        ] {
            let mut r = row(json!(1700000000));
            if let Some(value) = value {
                r[key] = value;
            } else {
                r.as_object_mut().unwrap().remove(key);
            }
            assert!(decode(vec![row(json!(1700000000)), r]).is_err());
        }
    }
    let mut prop = row(json!(1700000000));
    prop["type"] = json!("prop");
    assert!(decode(vec![prop]).is_err());
}
#[test]
fn windows_response_size_status_and_record_limits_are_closed() {
    for (a, b) in [
        (0, 1),
        (-1, 1),
        (1, 1),
        (2, 1),
        (1, 1 + 32 * 86400 + 1),
        (1, i64::MAX),
    ] {
        assert!(EventHistoryWindow::new(a, b).is_err());
    }
    assert!(decode(vec![row(json!(1700000000)); 32]).is_err());
    let full = decode(vec![row(json!(1700000000)); 31]).unwrap();
    assert!(full.limit_reached);
    assert_eq!(full.records.len(), 31);
    for bytes in [
        b"[]".to_vec(),
        br#"{"code":0,"result":{},"message":"PRIVATE"}"#.to_vec(),
        br#"{"code":1,"result":[]}"#.to_vec(),
        vec![b'x'; crate::MAX_RESPONSE_BYTES + 1],
    ] {
        assert!(decode_history(&bytes, "fictional.s7", &query(), window(), Utc::now()).is_err());
    }
}
fn session() -> MijiaSession {
    MijiaSession {
        region: Region::Cn,
        user_id: PrivateString::new("12345".into()),
        security: PrivateString::new("AAECAwQFBgcICQoLDA0ODw==".into()),
        token: PrivateString::new("fictional-token".into()),
    }
}
fn device(p: &Profile) -> CloudDevice {
    CloudDevice {
        did: PrivateString::new("fictional.s7".into()),
        name: "Fictional".into(),
        model: p.model.clone(),
        spec_type: Some(p.spec_type.clone()),
        online_hint: None,
    }
}
struct Peer {
    mode: u8,
    key: String,
    calls: AtomicUsize,
    started: tokio::sync::Notify,
}
impl Peer {
    fn new(mode: u8, key: String) -> Self {
        Self {
            mode,
            key,
            calls: AtomicUsize::new(0),
            started: tokio::sync::Notify::new(),
        }
    }
}
#[async_trait]
impl CloudTransport for Peer {
    async fn send(&self, r: &CloudRequest, operation: &Operation) -> Result<CloudResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(r.endpoint(), CloudEndpoint::EventHistory);
        assert!(r.cookie().contains("fictional-token"));
        let fields: std::collections::HashMap<_, _> = reqwest::Url::parse(&format!(
            "https://example.invalid/?{}",
            std::str::from_utf8(r.body()).unwrap()
        ))
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"])?;
        let data = codec::decrypt(&key, fields["data"].as_bytes())?;
        assert_eq!(
            serde_json::from_slice::<Value>(data.expose()).unwrap(),
            json!({"did":"fictional.s7","key":self.key,"type":"event","time_start":1700000000,"time_end":1700086400,"limit":31})
        );
        self.started.notify_one();
        if self.mode == 1 {
            std::future::pending::<()>().await;
        }
        if self.mode == 2 {
            operation.cancellation.cancel();
        }
        let mut r = row(json!(1700000000));
        r["key"] = json!(self.key);
        let reply = if self.mode == 3 {
            json!({"code":-10001})
        } else {
            json!({"code":0,"result":[r]})
        };
        Ok(CloudResponse {
            body: SecretBytes::new(
                codec::encrypt(&key, &serde_json::to_vec(&reply).unwrap())
                    .as_str()
                    .as_bytes()
                    .to_vec(),
            ),
            service_token: None,
        })
    }
}
#[tokio::test]
async fn all_78_legacy_queries_send_one_exact_event_history_request() {
    for p in profiles().unwrap() {
        let q = EventHistoryQuery {
            siid: p.siid,
            eiid: p.eiid,
        };
        let peer = Peer::new(0, format!("{}.{}", q.siid, q.eiid));
        let result = session()
            .reported_event_history(&peer, &device(&p), &q, window(), &op())
            .await
            .unwrap();
        assert_eq!(result.records[0].reported_at.timestamp(), 1700000000);
        assert!(!format!("{result:?}").contains("PRIVATE"));
        assert_eq!(peer.calls.load(Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn legacy_event_history_presend_refusal_auth_late_cancel_deadline_and_drop_do_not_retry() {
    let p = profiles().unwrap().remove(0);
    let q = EventHistoryQuery {
        siid: p.siid,
        eiid: p.eiid,
    };
    let peer = Peer::new(0, format!("{}.{}", q.siid, q.eiid));
    for mode in 0..6 {
        let mut s = session();
        let mut d = device(&p);
        let mut w = window();
        let o = op();
        let mut query = q;
        match mode {
            0 => d.model = "foreign".into(),
            1 => d.spec_type = None,
            2 => s.region = Region::Us,
            3 => query.eiid = 999999,
            4 => {
                w = EventHistoryWindow::new(Utc::now().timestamp(), Utc::now().timestamp() + 60)
                    .unwrap()
            }
            _ => o.cancellation.cancel(),
        };
        assert!(
            s.reported_event_history(&peer, &d, &query, w, &o)
                .await
                .is_err()
        );
    }
    assert_eq!(peer.calls.load(Ordering::SeqCst), 0);
    for mode in [2, 3] {
        let peer = Peer::new(mode, format!("{}.{}", q.siid, q.eiid));
        assert!(
            session()
                .reported_event_history(&peer, &device(&p), &q, window(), &op())
                .await
                .is_err()
        );
        assert_eq!(peer.calls.load(Ordering::SeqCst), 1);
    }
    for mode in 0..3 {
        let peer = Peer::new(1, format!("{}.{}", q.siid, q.eiid));
        let s = session();
        let d = device(&p);
        let operation =
            Operation::new(if mode == 1 { 100 } else { 5000 }, CancellationToken::new()).unwrap();
        let mut work = Box::pin(s.reported_event_history(&peer, &d, &q, window(), &operation));
        tokio::select! {_=peer.started.notified()=>{},_=&mut work=>panic!("Expected pending event history")}
        if mode == 0 {
            operation.cancellation.cancel();
            assert_eq!(work.await.unwrap_err().code, "CANCELLED");
        } else if mode == 1 {
            assert_eq!(work.await.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            drop(work);
        }
        assert_eq!(peer.calls.load(Ordering::SeqCst), 1);
    }
}
