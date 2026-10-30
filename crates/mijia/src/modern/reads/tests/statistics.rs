use super::*;
use crate::{
    DailyEnergyWindow,
    statistics::{DAILY_ENERGY_MODEL, DAILY_ENERGY_SPEC},
};

fn outlet(session: &ModernMijiaSession) -> ModernDirectoryEntry {
    session
        .directory_entry(EntryWire {
            did: PrivateString::new("FICTIONAL-ROOT.s7".into()),
            name: "Outlet".into(),
            model: DAILY_ENERGY_MODEL.into(),
            spec_type: Some(DAILY_ENERGY_SPEC.into()),
            online: None,
            pid: Some(DirectoryKind(0)),
            parent_id: None,
        })
        .unwrap()
}
fn window() -> DailyEnergyWindow {
    DailyEnergyWindow::new(1700000000, 1700086400).unwrap()
}
struct StatisticsPeer {
    mode: u8,
    calls: std::sync::atomic::AtomicUsize,
}
impl StatisticsPeer {
    fn new(mode: u8) -> Self {
        Self {
            mode,
            calls: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}
#[async_trait]
impl ModernReadTransport for StatisticsPeer {
    async fn send_modern_read(
        &self,
        request: &ModernReadRequest,
        operation: &Operation,
    ) -> Result<ModernReadResponse> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(request.endpoint(), ModernReadEndpoint::Statistics);
        assert_eq!(
            request.endpoint().url(),
            "https://api.mijia.tech/app/v2/user/statistics"
        );
        assert!(
            request
                .cookie()
                .starts_with("userId=12345;cUserId=fictional-c-user;")
        );
        assert!(
            request
                .cookie()
                .contains("serviceToken=fictional-service;locale=zh_CN;timezone=GMT+08:00;")
        );
        assert!(request.cookie().contains("PassportDeviceId="));
        assert!(!request.cookie().contains("passToken"));
        assert!(
            request
                .agent
                .as_str()
                .starts_with("Android-15-11.0.701-Xiaomi-")
        );
        assert_eq!(format!("{request:?}"), "ModernReadRequest([REDACTED])");
        let fields: BTreeMap<_, _> = Url::parse(&format!(
            "https://example.invalid/?{}",
            std::str::from_utf8(request.body()).unwrap()
        ))
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"]).unwrap();
        let data = codec::decrypt(&key, fields["data"].as_bytes()).unwrap();
        let hash = codec::decrypt(&key, fields["rc4_hash__"].as_bytes()).unwrap();
        use sha1::Digest;
        let expected = format!(
            "POST&/v2/user/statistics&data={}&{}",
            std::str::from_utf8(data.expose()).unwrap(),
            codec::base64(&key)
        );
        assert_eq!(
            hash.expose(),
            codec::base64(&sha1::Sha1::digest(expected.as_bytes())).as_bytes()
        );
        let signature = format!(
            "POST&/v2/user/statistics&data={}&rc4_hash__={}&{}",
            fields["data"],
            fields["rc4_hash__"],
            codec::base64(&key)
        );
        assert_eq!(
            fields["signature"],
            codec::base64(&sha1::Sha1::digest(signature.as_bytes()))
        );
        let value: Value = serde_json::from_slice(data.expose()).unwrap();
        assert_eq!(
            value,
            json!({"did":"FICTIONAL-ROOT.s7","key":"11.1","data_type":"stat_day_v3","time_start":1700000000,"time_end":1700086400,"limit":31})
        );
        let reply = match self.mode {
            5 | 15 => json!({"code": -10001}),
            6 => json!({"code": 0, "result":[{"did":"foreign", "time":1700000000,"value":"[1]"}]}),
            7 => json!({"code":0,"result":vec![json!({"time":1700000000,"value":"[0]"});32]}),
            _ => {
                json!({"code":0,"result":[{"did":"FICTIONAL-ROOT.s7","key":"11.1","time":1700000000,"value":"[125]"},{"time":1700000001,"value":"[0]"},{"time":1700000002,"value":"bad"}]})
            }
        };
        let plain = serde_json::to_vec(&reply).unwrap();
        let raw = match self.mode {
            2 | 3 => gzip(&plain),
            8 => gzip(&vec![b'x'; crate::MAX_RESPONSE_BYTES + 1]),
            9 => {
                let mut bytes = gzip(&plain);
                bytes.push(0);
                bytes
            }
            10 => vec![b'x'; crate::MAX_RESPONSE_BYTES + 1],
            13 => {
                let mut bytes = gzip(&plain);
                bytes.extend(gzip(&plain));
                bytes
            }
            11 => vec![31, 139, 0, 0],
            _ => plain,
        };
        if self.mode == 4 {
            operation.cancellation.cancel();
        }
        if self.mode == 12 {
            std::future::pending::<()>().await;
        }
        Ok(ModernReadResponse {
            body: SecretBytes::new(if self.mode == 1 || self.mode == 5 || self.mode == 14 {
                raw
            } else {
                codec::encrypt(&key, &raw).as_str().as_bytes().to_vec()
            }),
            encoding: if self.mode == 2 || self.mode == 14 {
                ModernContentEncoding::Gzip
            } else {
                ModernContentEncoding::Plain
            },
        })
    }
}
#[tokio::test]
async fn modern_statistics_selects_community_codec_and_preserves_root_and_historical_units() {
    for mode in 0..4 {
        let s = session();
        let p = StatisticsPeer::new(mode);
        let result = s
            .daily_energy_history(&p, &outlet(&s), window(), &operation())
            .await
            .unwrap();
        assert_eq!(
            result
                .records
                .iter()
                .map(|r| r.kilowatt_hours)
                .collect::<Vec<_>>(),
            vec![1.25, 0.0]
        );
        assert_eq!(result.rejected_records, 1);
        assert!(!result.limit_reached);
        assert_eq!(result.completeness, crate::StatisticsCompleteness::Unproven);
        assert_eq!(p.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn modern_statistics_rejects_bad_envelopes_and_late_lifecycle_without_retry() {
    for mode in (4..12).chain(13..16) {
        let s = session();
        let p = StatisticsPeer::new(mode);
        let err = s
            .daily_energy_history(&p, &outlet(&s), window(), &operation())
            .await
            .unwrap_err();
        if mode == 4 {
            assert_eq!(err.code, "CANCELLED");
        }
        if mode == 5 || mode == 15 {
            assert_eq!(err.code, "AUTH_REQUIRED");
        }
        assert_eq!(p.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn modern_statistics_refuses_foreign_subject_model_window_and_cancel_before_send() {
    let s = session();
    let p = StatisticsPeer::new(0);
    for mode in 0..8 {
        let mut e = outlet(&s);
        let mut w = window();
        let op = operation();
        match mode {
            0 => e.subject = PrivateString::new("54321".into()),
            1 => e.encrypted_subject = PrivateString::new("foreign".into()),
            2 => e.model = "cuco.plug.v2".into(),
            3 => e.mapping = None,
            4 => w = DailyEnergyWindow::new(4102444800, 4102531200).unwrap(),
            5 => e.kind = Some(DirectoryKind(21)),
            6 => e.kind = None,
            _ => op.cancellation.cancel(),
        }
        assert!(s.daily_energy_history(&p, &e, w, &op).await.is_err());
    }
    assert_eq!(p.calls.load(std::sync::atomic::Ordering::SeqCst), 0);
}
#[tokio::test]
async fn modern_statistics_deadline_drops_pending_transport() {
    let s = session();
    let p = StatisticsPeer::new(12);
    let e = outlet(&s);
    let op = Operation::new(25, CancellationToken::new()).unwrap();
    assert_eq!(
        s.daily_energy_history(&p, &e, window(), &op)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(p.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
}
