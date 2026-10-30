use super::*;
use crate::{CloudRequest, CloudResponse, PrivateString, codec};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_util::sync::CancellationToken;

fn window() -> PropertyHistoryWindow {
    PropertyHistoryWindow::new(1700000000, 1700086400).unwrap()
}
fn operation() -> Operation {
    Operation::new(1000, CancellationToken::new()).unwrap()
}
fn mapping(model: &str) -> ModelMapping {
    ModelMapping::pinned(model, None).unwrap().unwrap()
}
fn selected(model: &str, siid: u32, piid: u32) -> MappedProperty {
    let m = mapping(model);
    field(model, &m.spec_type, &PropertyQuery { siid, piid }).unwrap()
}
fn temperature() -> MappedProperty {
    selected("linp.sensor_ht.ks1bp", 2, 1001)
}
fn decode(field: &MappedProperty, records: Vec<Value>) -> Result<NumericPropertyHistory> {
    decode_history(
        &serde_json::to_vec(&json!({"code":0,"result":records})).unwrap(),
        "fictional.s7",
        field,
        window(),
        Utc::now(),
    )
}
fn record(time: i64, value: &str) -> Value {
    json!({"time":time,"value":value})
}
#[test]
fn property_history_preserves_signed_samples_units_and_raw_enum_codes() {
    let r=decode(&temperature(),vec![record(1700000002,"[12.3]"),json!({"time":"1700000000","value":"[-12.3]","uid":"PRIVATE-ACTOR","name":"PRIVATE-NAME"}),record(1700000001,"[-0]")]).unwrap();
    assert_eq!(
        r.records.iter().map(|r| r.value).collect::<Vec<_>>(),
        vec![-12.3, 0.0, 12.3]
    );
    assert_eq!(r.unit.as_deref(), Some("celsius"));
    assert_eq!(r.rejected_records, 0);
    assert_eq!(r.completeness, crate::StatisticsCompleteness::Unproven);
    assert!(!format!("{r:?}").contains("PRIVATE"));
    let r = decode(
        &selected("jdjz.fan.2072fc", 2, 4),
        vec![record(1700000000, "[12]")],
    )
    .unwrap();
    assert_eq!(r.records[0].value, 12.0);
    assert_eq!(r.unit, None);
    let r = decode(
        &selected("cuco.plug.v3", 11, 1),
        vec![record(1700000000, "[125]")],
    )
    .unwrap();
    assert_eq!(r.records[0].value, 125.0); // No daily-energy conversion.
}
#[test]
fn property_history_rejects_bad_values_times_and_every_ambiguous_duplicate() {
    let bad = [
        "[true]",
        "[null]",
        "[\"2\"]",
        "[1,2]",
        "[]",
        "[1e9999]",
        "[1e-9999]",
        "[-1e-9999]",
        "[100.1]",
        "[-30.1]",
        "[0.15]",
        "oops",
    ];
    let mut records = bad
        .iter()
        .enumerate()
        .map(|(i, v)| record(1700000000 + i as i64, v))
        .collect::<Vec<_>>();
    records.extend([
        record(1699999999, "[0]"),
        record(1700086400, "[0]"),
        json!({"time":1700000000.5,"value":"[0]"}),
        json!({"time":"1700000000ms","value":"[0]"}),
    ]);
    let n = records.len();
    let r = decode(&temperature(), records).unwrap();
    assert_eq!(r.rejected_records, n);
    assert!(r.records.is_empty());
    let r = decode(
        &temperature(),
        vec![
            record(1700000000, "[1]"),
            record(1700000000, "bad"),
            record(1700000000, "[2]"),
            record(1700000001, "[3]"),
        ],
    )
    .unwrap();
    assert_eq!(r.rejected_records, 3);
    assert_eq!(r.records.len(), 1);
    assert_eq!(
        decode(
            &selected("jdjz.fan.2072fc", 2, 4),
            vec![record(1700000000, "[13]")]
        )
        .unwrap()
        .rejected_records,
        1
    );
}
#[test]
fn property_history_refuses_foreign_correlations_and_oversized_envelopes() {
    for (key, value) in [
        ("did", json!("foreign")),
        ("key", json!("2.1008")),
        ("type", json!("event")),
        ("did", Value::Null),
    ] {
        let mut r = record(1700000000, "[1]");
        r[key] = value;
        assert!(decode(&temperature(), vec![r]).is_err());
    }
    assert!(decode(&temperature(), vec![record(1700000000, "[1]"); 32]).is_err());
    let r = decode(
        &temperature(),
        (0..31).map(|i| record(1700000000 + i, "[1]")).collect(),
    )
    .unwrap();
    assert!(r.limit_reached);
    assert_eq!(r.records.len(), 31);
    assert!(decode(&temperature(), vec![]).unwrap().records.is_empty());
    for bytes in [
        b"{\"code\":1,\"result\":[]}".to_vec(),
        b"{\"code\":0,\"result\":{}}".to_vec(),
        vec![b' '; crate::MAX_RESPONSE_BYTES + 1],
    ] {
        assert!(
            decode_history(&bytes, "fictional.s7", &temperature(), window(), Utc::now()).is_err()
        );
    }
    for (a, b) in [
        (0, 1),
        (1, 1),
        (2, 1),
        (1, 32 * 86400 + 2),
        (i64::MAX - 1, i64::MAX),
    ] {
        assert!(PropertyHistoryWindow::new(a, b).is_err());
    }
}
#[test]
fn property_history_all_pinned_numeric_fields_keep_exact_domains() {
    let profiles: Vec<Value> =
        serde_json::from_str(include_str!("../reference_profiles.json")).unwrap();
    let mut count = 0;
    for profile in profiles {
        let m = mapping(profile["model"].as_str().unwrap());
        for p in &m.properties {
            let selected = field(
                &m.model,
                &m.spec_type,
                &PropertyQuery {
                    siid: p.siid,
                    piid: p.piid,
                },
            );
            if p.definition["type"] == "boolean" {
                assert!(selected.is_err());
                continue;
            }
            let p = selected.unwrap();
            count += 1;
            let values = if let Some(e) = p.definition["enum"].as_array() {
                e.clone()
            } else {
                vec![
                    p.definition["minimum"].clone(),
                    p.definition["maximum"].clone(),
                ]
            };
            for v in values {
                let text = serde_json::to_string(&vec![v.clone()]).unwrap();
                let r = decode(&p, vec![record(1700000000, &text)]).unwrap();
                assert_eq!(
                    r.records.len(),
                    1,
                    "{} {}.{} {text}",
                    m.model,
                    p.siid,
                    p.piid
                );
                assert_eq!(r.records[0].value, v.as_f64().unwrap());
            }
        }
        assert!(field(&m.model, &m.spec_type, &PropertyQuery { siid: 1, piid: 1 }).is_err());
        assert!(
            field(
                &m.model,
                "urn:miot-spec-v2:device:fan:0000A005:foreign:1",
                &PropertyQuery { siid: 2, piid: 1 }
            )
            .is_err()
        );
    }
    assert_eq!(count, 161);
}
fn session() -> MijiaSession {
    MijiaSession {
        region: Region::Cn,
        user_id: PrivateString::new("12345".into()),
        security: PrivateString::new("AAECAwQFBgcICQoLDA0ODw==".into()),
        token: PrivateString::new("fictional-legacy-token".into()),
    }
}
fn device() -> CloudDevice {
    let m = mapping("linp.sensor_ht.ks1bp");
    CloudDevice {
        did: PrivateString::new("fictional.s7".into()),
        name: "Sensor".into(),
        model: m.model,
        spec_type: Some(m.spec_type),
        online_hint: None,
    }
}
struct Peer {
    mode: u8,
    calls: AtomicUsize,
}
impl Peer {
    fn new(mode: u8) -> Self {
        Self {
            mode,
            calls: AtomicUsize::new(0),
        }
    }
}
#[async_trait]
impl CloudTransport for Peer {
    async fn send(&self, r: &CloudRequest, op: &Operation) -> Result<CloudResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(r.endpoint(), CloudEndpoint::PropertyHistory);
        assert!(r.cookie().contains("fictional-legacy-token"));
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
            json!({"did":"fictional.s7","key":"2.1001","type":"prop","time_start":1700000000,"time_end":1700086400,"limit":31})
        );
        if self.mode == 1 {
            std::future::pending::<()>().await;
        }
        if self.mode == 2 {
            op.cancellation.cancel();
        }
        let reply = if self.mode == 3 {
            json!({"code":-10001})
        } else {
            json!({"code":0,"result":[{"did":"fictional.s7","key":"2.1001","type":"prop","time":1700000000,"value":"[-1.2]"}]})
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
async fn legacy_property_history_exact_request_and_failure_lifecycle() {
    for mode in 0..4 {
        let p = Peer::new(mode);
        let op =
            Operation::new(if mode == 1 { 50 } else { 1000 }, CancellationToken::new()).unwrap();
        let r = session()
            .numeric_property_history(
                &p,
                &device(),
                &PropertyQuery {
                    siid: 2,
                    piid: 1001,
                },
                window(),
                &op,
            )
            .await;
        if mode == 0 {
            assert_eq!(r.unwrap().records[0].value, -1.2);
        } else {
            assert!(r.is_err());
        }
        assert_eq!(p.calls.load(Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn legacy_property_history_refuses_targets_and_future_windows_before_io() {
    let p = Peer::new(0);
    for mode in 0..5 {
        let mut s = session();
        let mut d = device();
        let mut q = PropertyQuery {
            siid: 2,
            piid: 1001,
        };
        let mut w = window();
        match mode {
            0 => d.spec_type = None,
            1 => d.model = "unknown.sensor".into(),
            2 => q.piid = 999,
            3 => s.region = Region::Us,
            _ => {
                w = PropertyHistoryWindow::new(Utc::now().timestamp(), Utc::now().timestamp() + 60)
                    .unwrap()
            }
        };
        assert!(
            s.numeric_property_history(&p, &d, &q, w, &operation())
                .await
                .is_err()
        );
    }
    assert_eq!(p.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn legacy_property_history_pending_cancel_and_drop_do_not_retry() {
    for cancel in [true, false] {
        let peer = Peer::new(1);
        let session = session();
        let device = device();
        let query = PropertyQuery {
            siid: 2,
            piid: 1001,
        };
        let operation = operation();
        let mut pending = Box::pin(session.numeric_property_history(
            &peer,
            &device,
            &query,
            window(),
            &operation,
        ));
        loop {
            tokio::select! {
                _ = &mut pending => panic!("Expected pending property history"),
                _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => {
                    if peer.calls.load(Ordering::SeqCst) == 1 { break; }
                }
            }
        }
        if cancel {
            operation.cancellation.cancel();
            assert_eq!(pending.await.unwrap_err().code, "CANCELLED");
        } else {
            drop(pending);
        }
        assert_eq!(peer.calls.load(Ordering::SeqCst), 1);
    }
}
