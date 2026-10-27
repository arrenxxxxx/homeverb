use super::*;
use crate::{CloudRequest, CloudResponse, PrivateString, codec};
use async_trait::async_trait;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use tokio_util::sync::CancellationToken;

fn session() -> MijiaSession {
    MijiaSession {
        region: Region::Cn,
        user_id: PrivateString::new("2".into()),
        security: PrivateString::new("AAECAwQFBgcICQoLDA0ODw==".into()),
        token: PrivateString::new("fictional-token".into()),
    }
}
fn device() -> CloudDevice {
    CloudDevice {
        did: PrivateString::new("fictional-device".into()),
        name: "Fictional outlet".into(),
        model: DAILY_ENERGY_MODEL.into(),
        spec_type: Some(DAILY_ENERGY_SPEC.into()),
        online_hint: None,
    }
}
fn window() -> DailyEnergyWindow {
    DailyEnergyWindow::new(1700000000, 1700086400).unwrap()
}
fn operation() -> Operation {
    Operation::new(10000, CancellationToken::new()).unwrap()
}
fn record(time: i64, value: &str) -> Value {
    json!({"time":time,"value":value})
}
fn history(records: Vec<Value>) -> Result<DailyEnergyHistory> {
    decode_history(
        &serde_json::to_vec(&json!({"code":0,"result":records})).unwrap(),
        "fictional-device",
        window(),
        Utc::now(),
    )
}
struct Transport {
    reply: Value,
    calls: AtomicUsize,
    data: Mutex<Option<Value>>,
}
impl Transport {
    fn new(reply: Value) -> Self {
        Self {
            reply,
            calls: AtomicUsize::new(0),
            data: Mutex::new(None),
        }
    }
}
#[async_trait]
impl CloudTransport for Transport {
    async fn send(&self, request: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.endpoint(), CloudEndpoint::Statistics);
        assert_eq!(request.region(), Region::Cn);
        let url = reqwest::Url::parse(&format!(
            "https://localhost/?{}",
            std::str::from_utf8(request.body()).unwrap()
        ))
        .unwrap();
        let fields: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"])?;
        let data = codec::decrypt(&key, fields["data"].as_bytes())?;
        *self.data.lock().unwrap() = Some(serde_json::from_slice(data.expose()).unwrap());
        let body = codec::encrypt(&key, &serde_json::to_vec(&self.reply).unwrap());
        Ok(CloudResponse {
            body: SecretBytes::new(body.as_str().as_bytes().to_vec()),
            service_token: None,
        })
    }
}

#[test]
fn windows_are_bounded_and_whole_seconds_without_calendar_assumptions() {
    for (start, end) in [
        (0, 1),
        (-1, 1),
        (100, 100),
        (100, 99),
        (1, MAX_WINDOW_SECONDS + 2),
        (i64::MAX - 1, i64::MAX),
    ] {
        assert!(DailyEnergyWindow::new(start, end).is_err());
    }
    assert!(DailyEnergyWindow::new(1, MAX_WINDOW_SECONDS + 1).is_ok());
}

#[test]
fn records_scale_only_the_exact_energy_unit_and_keep_original_timestamps() {
    let result = history(vec![
        record(1700000002, "[123.5]"),
        json!({"time":"1700000000","value":"[0]"}),
    ])
    .unwrap();
    assert_eq!(result.records[0].kilowatt_hours, 0.0);
    assert!((result.records[1].kilowatt_hours - 1.235).abs() < 1e-12);
    assert_eq!(result.records[1].reported_at.timestamp(), 1700000002);
    assert_eq!(result.rejected_records, 0);
    assert!(!result.limit_reached);
    assert_eq!(result.completeness, StatisticsCompleteness::Unproven);
    assert!(history(vec![]).unwrap().records.is_empty());
}

#[test]
fn invalid_values_and_dates_never_replace_missing_energy_with_zero() {
    for bad in [
        json!(null),
        json!(true),
        json!(123),
        json!([1]),
        json!(""),
        json!("{}"),
        json!("[]"),
        json!("[null]"),
        json!("[true]"),
        json!("[\"1\"]"),
        json!("[-1]"),
        json!("[NaN]"),
        json!("[Infinity]"),
        json!("[1e999]"),
        json!("[1,2]"),
        json!("[1e-323]"),
        json!("[1e-999]"),
        json!("[-1e-999]"),
        json!("1"),
        json!(" ".repeat(129)),
    ] {
        let result = history(vec![
            json!({"time":1700000000,"value":bad}),
            record(1700000001, "[25]"),
        ])
        .unwrap();
        assert_eq!(result.records.len(), 1, "{bad:?}");
        assert_eq!(result.rejected_records, 1);
        assert_eq!(result.records[0].kilowatt_hours, 0.25);
    }
    for bad in [
        json!(null),
        json!(true),
        json!(-1),
        json!(1700000000.5),
        json!("1700000000.0"),
        json!(" 1700000000"),
        json!("+1700000000"),
        json!(1700086400),
        json!(1699999999),
        json!("99999999999999999999"),
    ] {
        let result = history(vec![
            json!({"time":bad,"value":"[1]"}),
            record(1700000001, "[25]"),
        ])
        .unwrap();
        assert_eq!(result.records.len(), 1, "{bad:?}");
        assert_eq!(result.rejected_records, 1);
    }
    let result = history(vec![
        json!(null),
        json!({}),
        json!("private"),
        record(1700000001, "[25]"),
    ])
    .unwrap();
    assert_eq!(result.rejected_records, 3);
    assert_eq!(result.records.len(), 1);
}

#[test]
fn duplicate_dates_are_all_excluded_including_when_one_record_is_invalid() {
    for values in [
        ["[1]", "[2]", "[3]"],
        ["bad", "[2]", "[3]"],
        ["[1]", "bad", "[3]"],
    ] {
        let mut records: Vec<_> = values.into_iter().map(|v| record(1700000000, v)).collect();
        records.push(record(1700000001, "[4]"));
        let result = history(records).unwrap();
        assert_eq!(result.rejected_records, 3);
        assert_eq!(result.records.len(), 1);
        assert_eq!(result.records[0].kilowatt_hours, 0.04);
    }
}

#[test]
fn record_limits_and_optional_correlation_never_claim_complete_history() {
    let records: Vec<_> = (0..31).map(|i| record(1700000000 + i, "[1]")).collect();
    let result = history(records.clone()).unwrap();
    assert!(result.limit_reached);
    let mut too_many = records;
    too_many.push(record(1700000031, "[1]"));
    assert!(history(too_many).is_err());
    for (key, value) in [
        ("did", json!("foreign-private-device")),
        ("did", json!(2)),
        ("key", json!("11.2")),
    ] {
        let mut rec = record(1700000000, "[1]");
        rec[key] = value;
        let error = history(vec![rec]).unwrap_err();
        assert_eq!(error.code, "MIJIA_INVALID_RESPONSE");
        assert!(!format!("{error:?}").contains("private-device"));
    }
    let mut rec = record(1700000000, "[1]");
    rec["did"] = json!("fictional-device");
    rec["key"] = json!("11.1");
    assert_eq!(history(vec![rec]).unwrap().records.len(), 1);
    for bytes in [
        b"{}".as_slice(),
        b"{\"code\":0,\"result\":null}",
        b"{\"code\":1,\"result\":[]}",
        b"[]",
    ] {
        assert!(decode_history(bytes, "fictional-device", window(), Utc::now()).is_err());
    }
    assert!(
        decode_history(
            &vec![b' '; crate::MAX_RESPONSE_BYTES + 1],
            "fictional-device",
            window(),
            Utc::now()
        )
        .is_err()
    );
}

#[tokio::test]
async fn fixed_encrypted_wire_contract_preserves_history_and_sanitizes_provider_errors() {
    let transport = Transport::new(json!({"code":0,"result":[record(1700000000,"[25]")]}));
    let result = session()
        .daily_energy_history(&transport, &device(), window(), &operation())
        .await
        .unwrap();
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        transport.data.lock().unwrap().as_ref().unwrap(),
        &json!({
        "did":"fictional-device","key":"11.1","data_type":"stat_day_v3",
        "time_start":1700000000,"time_end":1700086400,"limit":31})
    );
    assert_eq!(result.records[0].kilowatt_hours, 0.25);
    assert!(!format!("{result:?}").contains("fictional-device"));
    for (code, expected) in [(-3, "AUTH_REQUIRED"), (999, "MIJIA_REQUEST_REJECTED")] {
        let transport = Transport::new(json!({"code":code,"message":"private-response"}));
        let error = session()
            .daily_energy_history(&transport, &device(), window(), &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, expected);
        assert!(!format!("{error:?}").contains("private-response"));
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn foreign_profiles_future_windows_and_cancelled_operations_never_dispatch() {
    for mode in 0..7 {
        let mut session = session();
        let mut device = device();
        let mut window = window();
        let op = operation();
        match mode {
            0 => session.region = Region::Us,
            1 => device.model = "cuco.plug.v2".into(),
            2 => device.spec_type = None,
            3 => device.spec_type = Some(format!("{DAILY_ENERGY_SPEC}:extra")),
            4 => device.did = PrivateString::new("bad\nidentity".into()),
            5 => {
                window = DailyEnergyWindow::new(
                    Utc::now().timestamp() + 100,
                    Utc::now().timestamp() + 200,
                )
                .unwrap()
            }
            _ => op.cancellation.cancel(),
        }
        let transport = Transport::new(json!({}));
        assert!(
            session
                .daily_energy_history(&transport, &device, window, &op)
                .await
                .is_err()
        );
        assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
    }
}

struct Hanging {
    active: Arc<AtomicUsize>,
    calls: Arc<AtomicUsize>,
}
struct Guard(Arc<AtomicUsize>);
impl Drop for Guard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
#[async_trait]
impl CloudTransport for Hanging {
    async fn send(&self, _: &CloudRequest, _: &Operation) -> Result<CloudResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.active.fetch_add(1, Ordering::SeqCst);
        let _guard = Guard(self.active.clone());
        std::future::pending().await
    }
}
#[tokio::test]
async fn cancellation_deadline_and_drop_release_pending_read_without_retry() {
    for mode in 0..3 {
        let transport = Hanging {
            active: Arc::new(AtomicUsize::new(0)),
            calls: Arc::new(AtomicUsize::new(0)),
        };
        let mut op = operation();
        if mode == 1 {
            op.deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(100);
        }
        let session = session();
        let device = device();
        let mut work = Box::pin(session.daily_energy_history(&transport, &device, window(), &op));
        tokio::select! {result=&mut work=>panic!("Unexpected result: {result:?}"),_=tokio::time::sleep(std::time::Duration::from_millis(10))=>{}}
        assert_eq!(transport.active.load(Ordering::SeqCst), 1);
        if mode == 0 {
            op.cancellation.cancel();
            assert_eq!(work.await.unwrap_err().code, "CANCELLED");
        } else if mode == 1 {
            assert_eq!(work.await.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            drop(work);
        }
        assert_eq!(transport.active.load(Ordering::SeqCst), 0);
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    }
}
