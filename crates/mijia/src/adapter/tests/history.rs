use super::*;
use homeverb_domain::{HistoryAggregation, HistoryCompleteness, HistoryRequest};

async fn fixture(reported: bool) -> (Fixture, AccountDiscovery, Device) {
    let f = Fixture::new();
    let mut entry = json!({"did":"FICTIONAL-ENERGY-DEVICE","name":"Fictional outlet","model":crate::statistics::DAILY_ENERGY_MODEL});
    if reported {
        entry["spec_type"] = json!(crate::statistics::DAILY_ENERGY_SPEC);
    }
    *f.cloud.devices.lock().unwrap() = vec![entry];
    let report = f.connect().await;
    f.publish(&report).await;
    let device = f.core.list().unwrap().remove(0);
    (f, report, device)
}
async fn request(f: &Fixture, device: &Device) -> HistoryRequest {
    let options = f
        .core
        .inspect_history_options(&device.id, &device.connections[0].id, &operation())
        .await
        .unwrap();
    assert_eq!(options.options.len(), 1);
    assert_eq!(options.options[0].unit, "kWh");
    HistoryRequest {
        schema_version: "1.0".into(),
        device_id: device.id.clone(),
        connection_id: device.connections[0].id.clone(),
        target: options.options[0].target.clone(),
        aggregation: HistoryAggregation::ReportedDaily,
        start: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
        end: chrono::DateTime::from_timestamp(1700086400, 0).unwrap(),
    }
}
#[tokio::test]
async fn exact_history_runs_through_core_with_protected_reference_fallback_and_no_state_mutation() {
    for reported in [true, false] {
        let (f, _, device) = fixture(reported).await;
        f.cloud.calls.lock().unwrap().clear();
        let puts = f.secrets.puts.load(Ordering::SeqCst);
        let query = request(&f, &device).await;
        assert!(f.cloud.calls.lock().unwrap().is_empty());
        let result = f.core.read_history(&query, &operation()).await.unwrap();
        assert_eq!(
            result.records.iter().map(|r| r.value).collect::<Vec<_>>(),
            vec![0.0, 1.25]
        );
        assert_eq!(result.rejected_records, 1);
        assert_eq!(result.completeness, HistoryCompleteness::Unproven);
        assert_eq!(
            *f.cloud.calls.lock().unwrap(),
            vec![CloudEndpoint::Statistics]
        );
        assert_eq!(f.secrets.puts.load(Ordering::SeqCst), puts);
        assert!(
            f.store
                .snapshot(&device.id, &query.target.component_id)
                .unwrap()
                .is_none()
        );
        assert!(f.core.recent_executions().unwrap().is_empty());
        let public = serde_json::to_string(&result).unwrap();
        for secret in [
            "FICTIONAL-ENERGY-DEVICE",
            "PRIVATE-SERVICE-TOKEN",
            "999001",
            "11.1",
        ] {
            assert!(!public.contains(secret));
        }
    }
}
#[tokio::test]
async fn direct_adapter_history_repeats_target_window_and_public_identity_admission() {
    let (f, _, device) = fixture(true).await;
    let query = request(&f, &device).await;
    f.cloud.calls.lock().unwrap().clear();
    for mode in 0..7 {
        let mut request = query.clone();
        let mut device = device.clone();
        let mut connection = device.connections[0].clone();
        match mode {
            0 => request.target.property = "unreviewed".into(),
            1 => request.device_id = "foreign".into(),
            2 => request.connection_id = "foreign".into(),
            3 => request.start += chrono::Duration::milliseconds(1),
            4 => request.end = request.start + chrono::Duration::days(33),
            5 => device.model = "cuco.plug.v2".into(),
            _ => connection.identity_ref = Uuid::new_v4().to_string(),
        }
        assert!(
            f.adapter
                .read_history(&device, &connection, &request, &operation())
                .await
                .is_err()
        );
    }
    assert!(f.cloud.calls.lock().unwrap().is_empty());
    let mut foreign = device.clone();
    foreign.model = "unreviewed".into();
    assert!(
        f.adapter
            .history_options(&foreign, &foreign.connections[0], &operation())
            .await
            .is_err()
    );
    assert!(f.cloud.calls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn changed_protected_membership_and_unrelated_models_never_read_history() {
    let (f, _, device) = fixture(true).await;
    let query = request(&f, &device).await;
    f.cloud.calls.lock().unwrap().clear();
    {
        let mut secrets = f.secrets.values.lock().unwrap();
        let bytes = secrets.values_mut().next().unwrap();
        let mut payload: Value = serde_json::from_slice(bytes).unwrap();
        payload["bindings"][0]["listed"] = json!(false);
        *bytes = serde_json::to_vec(&payload).unwrap();
    }
    assert!(
        f.core
            .inspect_history_options(&device.id, &device.connections[0].id, &operation())
            .await
            .unwrap()
            .options
            .is_empty()
    );
    assert!(f.core.read_history(&query, &operation()).await.is_err());
    assert!(f.cloud.calls.lock().unwrap().is_empty());
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let device = f.fan();
    f.cloud.calls.lock().unwrap().clear();
    assert!(
        f.core
            .inspect_history_options(&device.id, &device.connections[0].id, &operation())
            .await
            .unwrap()
            .options
            .is_empty()
    );
    assert!(f.cloud.calls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn historical_reads_hold_and_release_account_and_device_leases() {
    for mode in ["cancel", "deadline", "drop"] {
        let (f, account, device) = fixture(true).await;
        let query = request(&f, &device).await;
        f.cloud.stall_read.store(1, Ordering::SeqCst);
        let mut op = operation();
        if mode == "deadline" {
            op.deadline = tokio::time::Instant::now() + Duration::from_millis(100);
        }
        let mut work = Box::pin(f.core.read_history(&query, &op));
        tokio::select! {result=&mut work=>panic!("Unexpected result: {result:?}"),_=f.cloud.read_gate.notified()=>{}}
        assert!(
            f.vault
                .lease(account.account.id, &operation())
                .await
                .is_err()
        );
        assert!(f.store.lock_device(&device.id).is_err());
        match mode {
            "cancel" => {
                op.cancellation.cancel();
                assert_eq!(work.await.unwrap_err().code, "CANCELLED");
            }
            "deadline" => assert_eq!(work.await.unwrap_err().code, "DEADLINE_EXCEEDED"),
            _ => drop(work),
        }
        assert!(
            f.vault
                .lease(account.account.id, &operation())
                .await
                .is_ok()
        );
        assert!(f.store.lock_device(&device.id).is_ok());
        assert_eq!(f.cloud.count(CloudEndpoint::Statistics), 1);
        assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
    }
}
