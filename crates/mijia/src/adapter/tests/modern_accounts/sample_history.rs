use super::*;
use homeverb_domain::{HistoryAggregation, HistoryRequest};

#[tokio::test]
async fn every_pinned_sample_history_uses_protected_protocol_without_state_or_secret_mutation() {
    for modern in [false, true] {
        let f = Fixture::new();
        let p = Peer::new();
        let catalog: Vec<Value> =
            serde_json::from_str(include_str!("../../../reference_profiles.json")).unwrap();
        let devices:Vec<Value>=catalog.iter().map(|profile|json!({"did":format!("PRIVATE-{}",profile["model"].as_str().unwrap()),"model":profile["model"],"name":"Appliance","pid":0})).collect();
        *f.cloud.devices.lock().unwrap() = devices.clone();
        *p.devices.lock().unwrap() = devices;
        let a = Arc::new(adapter(&f, &p));
        let legacy = a
            .connect_devices(credentials(), None, "Home", &operation())
            .await
            .unwrap();
        f.publish(&legacy).await;
        let record = connect(&a).await.account;
        if modern {
            let report = a
                .refresh_modern_devices(record.id, &operation())
                .await
                .unwrap();
            f.publish(&report).await;
        }
        let core = Core::new(
            f.store.clone(),
            vec![a.clone()],
            Arc::new(SystemClock),
            Policy::default(),
        );
        let before = bytes(&f, &record);
        let writes = puts(&f);
        let auth = p.auth.lock().unwrap().len();
        p.reads.lock().unwrap().clear();
        f.cloud.calls.lock().unwrap().clear();
        let mut count = 0;
        for device in core.list().unwrap() {
            let n = p.reads.lock().unwrap().len() + f.cloud.calls.lock().unwrap().len();
            let options = core
                .inspect_history_options(&device.id, &device.connections[0].id, &operation())
                .await
                .unwrap();
            assert_eq!(
                p.reads.lock().unwrap().len() + f.cloud.calls.lock().unwrap().len(),
                n
            );
            for option in options
                .options
                .into_iter()
                .filter(|o| o.aggregation == HistoryAggregation::ReportedSamples)
            {
                count += 1;
                let query = HistoryRequest {
                    schema_version: "1.0".into(),
                    device_id: device.id.clone(),
                    connection_id: device.connections[0].id.clone(),
                    target: option.target.clone(),
                    aggregation: option.aggregation,
                    start: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
                    end: chrono::DateTime::from_timestamp(1700086400, 0).unwrap(),
                };
                let result = core.read_history(&query, &operation()).await.unwrap();
                assert_eq!(result.unit, option.unit);
                assert_eq!(result.records.len(), 1);
                assert!(!serde_json::to_string(&result).unwrap().contains("PRIVATE"));
                assert!(
                    f.store
                        .snapshot(&device.id, &option.target.component_id)
                        .unwrap()
                        .is_none()
                );
            }
        }
        assert_eq!(count, 161);
        assert_eq!(bytes(&f, &record), before);
        assert_eq!(puts(&f), writes);
        assert_eq!(p.auth.lock().unwrap().len(), auth);
        assert!(core.recent_executions().unwrap().is_empty());
        if modern {
            assert!(f.cloud.calls.lock().unwrap().is_empty());
            assert_eq!(*p.reads.lock().unwrap(), vec![Read::PropertyHistory; 161]);
        } else {
            assert!(p.reads.lock().unwrap().is_empty());
            assert_eq!(
                *f.cloud.calls.lock().unwrap(),
                vec![CloudEndpoint::PropertyHistory; 161]
            );
        }
    }
}
