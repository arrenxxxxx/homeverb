use super::*;
use homeverb_domain::{HistoryAggregation, HistoryCompleteness, HistoryRequest};

struct HistoryFixture {
    f: Fixture,
    p: Arc<Peer>,
    a: Arc<MijiaAdapter>,
    c: Core,
    record: AccountRecord,
    device: Device,
}
async fn fixture(mixed: bool, modern: bool) -> HistoryFixture {
    let f = Fixture::new();
    let p = Peer::new();
    *p.devices.lock().unwrap() = vec![
        json!({"did":"FICTIONAL-ENERGY-DEVICE","name":"Outlet","model":crate::statistics::DAILY_ENERGY_MODEL,"pid":0}),
    ];
    let a = Arc::new(adapter(&f, &p));
    if mixed {
        *f.cloud.devices.lock().unwrap() = p.devices.lock().unwrap().clone();
        let old = a
            .connect_devices(credentials(), None, "Home", &operation())
            .await
            .unwrap();
        f.publish(&old).await;
    }
    let record = connect(&a).await.account;
    if modern {
        let report = a
            .refresh_modern_devices(record.id, &operation())
            .await
            .unwrap();
        f.publish(&report).await;
    }
    let c = Core::new(
        f.store.clone(),
        vec![a.clone()],
        Arc::new(SystemClock),
        Policy::default(),
    );
    let device = c.list().unwrap().remove(0);
    HistoryFixture {
        f,
        p,
        a,
        c,
        record,
        device,
    }
}
async fn query(t: &HistoryFixture) -> HistoryRequest {
    let options =
        t.c.inspect_history_options(&t.device.id, &t.device.connections[0].id, &operation())
            .await
            .unwrap();
    assert_eq!(options.options.len(), 1);
    HistoryRequest {
        schema_version: "1.0".into(),
        device_id: t.device.id.clone(),
        connection_id: t.device.connections[0].id.clone(),
        target: options.options[0].target.clone(),
        aggregation: HistoryAggregation::ReportedDaily,
        start: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
        end: chrono::DateTime::from_timestamp(1700086400, 0).unwrap(),
    }
}
#[tokio::test]
async fn modern_history_works_after_reopen_without_legacy_slot_cache_secret_or_receipt_mutation() {
    for mixed in [false, true] {
        let t = fixture(mixed, true).await;
        let q = query(&t).await;
        t.p.reads.lock().unwrap().clear();
        t.f.cloud.calls.lock().unwrap().clear();
        let before = bytes(&t.f, &t.record);
        let writes = puts(&t.f);
        let auth = t.p.auth.lock().unwrap().len();
        let c = Core::new(
            t.f.store.clone(),
            vec![Arc::new(adapter(&t.f, &t.p))],
            Arc::new(SystemClock),
            Policy::default(),
        );
        let result = c.read_history(&q, &operation()).await.unwrap();
        assert_eq!(result.records[0].value, 1.25);
        assert_eq!(result.completeness, HistoryCompleteness::Unproven);
        assert_eq!(result.bucket_timezone, None);
        assert_eq!(*t.p.reads.lock().unwrap(), vec![Read::Statistics]);
        assert!(t.f.cloud.calls.lock().unwrap().is_empty());
        assert_eq!(t.p.auth.lock().unwrap().len(), auth);
        assert_eq!(bytes(&t.f, &t.record), before);
        assert_eq!(puts(&t.f), writes);
        assert!(c.recent_executions().unwrap().is_empty());
        assert!(
            t.f.store
                .snapshot(&t.device.id, &q.target.component_id)
                .unwrap()
                .is_none()
        );
        let public = serde_json::to_string(&result).unwrap();
        for private in [
            "FICTIONAL-ENERGY-DEVICE",
            "PRIVATE-MODERN",
            "999001",
            "11.1",
        ] {
            assert!(!public.contains(private));
        }
    }
}
#[tokio::test]
async fn legacy_history_stays_legacy_after_modern_login_and_modern_failure_never_falls_back() {
    let t = fixture(true, false).await;
    let q = query(&t).await;
    t.p.reads.lock().unwrap().clear();
    t.f.cloud.calls.lock().unwrap().clear();
    t.c.read_history(&q, &operation()).await.unwrap();
    assert!(t.p.reads.lock().unwrap().is_empty());
    assert_eq!(
        *t.f.cloud.calls.lock().unwrap(),
        vec![CloudEndpoint::Statistics]
    );
    let t = fixture(true, true).await;
    let q = query(&t).await;
    t.p.reads.lock().unwrap().clear();
    t.f.cloud.calls.lock().unwrap().clear();
    let auth = t.p.auth.lock().unwrap().len();
    t.p.fail_read.store(1, Ordering::SeqCst);
    assert!(t.c.read_history(&q, &operation()).await.is_err());
    assert_eq!(*t.p.reads.lock().unwrap(), vec![Read::Statistics]);
    assert!(t.f.cloud.calls.lock().unwrap().is_empty());
    assert_eq!(t.p.auth.lock().unwrap().len(), auth);
    let absent = MijiaAdapter::new(t.f.vault.clone(), t.f.cloud.clone(), Arc::new(SystemClock));
    assert!(
        absent
            .history_options(&t.device, &t.device.connections[0], &operation())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        absent
            .read_history(&t.device, &t.device.connections[0], &q, &operation())
            .await
            .is_err()
    );
    assert!(t.f.cloud.calls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn modern_history_rechecks_exact_target_identity_and_protected_membership() {
    let t = fixture(false, true).await;
    let q = query(&t).await;
    t.p.reads.lock().unwrap().clear();
    for mode in 0..6 {
        let mut q = q.clone();
        let mut device = t.device.clone();
        match mode {
            0 => q.target.property = "unreviewed".into(),
            1 => q.device_id = "foreign".into(),
            2 => q.connection_id = "foreign".into(),
            3 => q.start += chrono::Duration::milliseconds(1),
            4 => q.end = q.start + chrono::Duration::days(33),
            _ => device.model = "cuco.plug.v2".into(),
        }
        assert!(
            t.a.read_history(&device, &device.connections[0], &q, &operation())
                .await
                .is_err()
        );
    }
    let mut saved = payload(&t.f, &t.record);
    saved["bindings"][0]["listed"] = json!(false);
    t.f.secrets.values.lock().unwrap().insert(
        t.record.credential_ref.to_string(),
        serde_json::to_vec(&saved).unwrap(),
    );
    assert!(
        t.c.inspect_history_options(&t.device.id, &t.device.connections[0].id, &operation())
            .await
            .unwrap()
            .options
            .is_empty()
    );
    assert!(t.c.read_history(&q, &operation()).await.is_err());
    assert!(t.p.reads.lock().unwrap().is_empty());
}
#[tokio::test]
async fn modern_history_cancellation_deadline_drop_and_late_reply_release_leases() {
    for mode in 0..4 {
        let t = fixture(false, true).await;
        let q = query(&t).await;
        t.p.reads.lock().unwrap().clear();
        let mut op = operation();
        if mode == 3 {
            let cancellation = op.cancellation.clone();
            *t.p.hook.lock().unwrap() = Some(Box::new(move || cancellation.cancel()));
            assert_eq!(
                t.c.read_history(&q, &op).await.unwrap_err().code,
                "CANCELLED"
            );
        } else {
            t.p.pause.store(2, Ordering::SeqCst);
            if mode == 1 {
                op.deadline = tokio::time::Instant::now() + Duration::from_millis(100);
            }
            let mut work = Box::pin(t.c.read_history(&q, &op));
            tokio::select! {_=t.p.started.notified()=>{},result=&mut work=>panic!("Unexpected {result:?}")}
            assert!(t.f.vault.lease(t.record.id, &operation()).await.is_err());
            assert!(t.f.store.lock_device(&t.device.id).is_err());
            match mode {
                0 => {
                    op.cancellation.cancel();
                    assert_eq!(work.await.unwrap_err().code, "CANCELLED");
                }
                1 => assert_eq!(work.await.unwrap_err().code, "DEADLINE_EXCEEDED"),
                _ => drop(work),
            }
        }
        assert!(t.f.vault.lease(t.record.id, &operation()).await.is_ok());
        assert!(t.f.store.lock_device(&t.device.id).is_ok());
        assert_eq!(*t.p.reads.lock().unwrap(), vec![Read::Statistics]);
        assert!(t.f.cloud.calls.lock().unwrap().is_empty());
    }
}
