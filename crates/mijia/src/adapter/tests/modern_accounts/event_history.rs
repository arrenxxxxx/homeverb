use super::*;
use homeverb_domain::EventHistoryRequest;

#[tokio::test]
async fn all_78_event_histories_keep_exact_protected_protocol_and_never_mutate_state_or_secrets() {
    for modern in [false, true] {
        let f = Fixture::new();
        let p = Peer::new();
        let catalog: Vec<Value> =
            serde_json::from_str(include_str!("../../../reference_profiles.json")).unwrap();
        let devices:Vec<Value>=catalog.iter().map(|profile|json!({"did":format!("PRIVATE-{}",profile["model"].as_str().unwrap()),"model":profile["model"],"name":"Appliance","pid":0})).collect();
        *f.cloud.devices.lock().unwrap() = devices.clone();
        *p.devices.lock().unwrap() = devices;
        let a = Arc::new(adapter(&f, &p).with_event_history());
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
                .inspect_event_history_options(&device.id, &device.connections[0].id, &operation())
                .await
                .unwrap();
            assert_eq!(
                p.reads.lock().unwrap().len() + f.cloud.calls.lock().unwrap().len(),
                n
            );
            for option in options.options {
                count += 1;
                let query = EventHistoryRequest {
                    schema_version: "1.0".into(),
                    device_id: device.id.clone(),
                    connection_id: device.connections[0].id.clone(),
                    target: option.target.clone(),
                    projection: option.projection,
                    start: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
                    end: chrono::DateTime::from_timestamp(1700086400, 0).unwrap(),
                };
                let result = core.read_event_history(&query, &operation()).await.unwrap();
                assert_eq!(result.records.len(), 2);
                assert_eq!(result.records[0], result.records[1]);
                assert!(!serde_json::to_string(&result).unwrap().contains("PRIVATE"));
                assert!(
                    f.store
                        .snapshot(&device.id, &option.target.component_id)
                        .unwrap()
                        .is_none()
                );
            }
        }
        assert_eq!(count, 78);
        let mut scalar_count = 0;
        for descriptor in f.store.devices().unwrap() {
            for capability in &descriptor.capabilities {
                scalar_count += capability.properties.len();
            }
        }
        assert_eq!(scalar_count, 264);
        assert_eq!(bytes(&f, &record), before);
        assert_eq!(puts(&f), writes);
        assert_eq!(p.auth.lock().unwrap().len(), auth);
        assert!(core.recent_executions().unwrap().is_empty());
        if modern {
            assert!(f.cloud.calls.lock().unwrap().is_empty());
            assert_eq!(*p.reads.lock().unwrap(), vec![Read::EventHistory; 78]);
        } else {
            assert!(p.reads.lock().unwrap().is_empty());
            assert_eq!(
                *f.cloud.calls.lock().unwrap(),
                vec![CloudEndpoint::EventHistory; 78]
            );
        }
    }
}

struct EventFixture {
    f: Fixture,
    p: Arc<Peer>,
    a: Arc<MijiaAdapter>,
    c: Core,
    record: AccountRecord,
    device: Device,
}
async fn fixture(modern: bool, enabled: bool) -> EventFixture {
    let f = Fixture::new();
    let p = Peer::new();
    let devices = vec![
        json!({"did":"FICTIONAL-EVENT-DEVICE","name":"Camera","model":"xiaomi.camera.083ac1","pid":0}),
    ];
    *p.devices.lock().unwrap() = devices.clone();
    *f.cloud.devices.lock().unwrap() = devices;
    let a = adapter(&f, &p);
    let a = Arc::new(if enabled { a.with_event_history() } else { a });
    let legacy = a
        .connect_devices(credentials(), None, "Home", &operation())
        .await
        .unwrap();
    f.publish(&legacy).await;
    let record = connect(&a).await.account;
    if modern {
        f.publish(
            &a.refresh_modern_devices(record.id, &operation())
                .await
                .unwrap(),
        )
        .await;
    }
    let c = Core::new(
        f.store.clone(),
        vec![a.clone()],
        Arc::new(SystemClock),
        Policy::default(),
    );
    let device = c.list().unwrap().remove(0);
    p.reads.lock().unwrap().clear();
    f.cloud.calls.lock().unwrap().clear();
    EventFixture {
        f,
        p,
        a,
        c,
        record,
        device,
    }
}
async fn query(t: &EventFixture) -> EventHistoryRequest {
    let options =
        t.c.inspect_event_history_options(&t.device.id, &t.device.connections[0].id, &operation())
            .await
            .unwrap();
    assert_eq!(options.options.len(), 20);
    EventHistoryRequest {
        schema_version: "1.0".into(),
        device_id: t.device.id.clone(),
        connection_id: t.device.connections[0].id.clone(),
        target: options.options[0].target.clone(),
        projection: options.options[0].projection,
        start: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
        end: chrono::DateTime::from_timestamp(1700086400, 0).unwrap(),
    }
}
#[tokio::test]
async fn event_history_requires_explicit_composition_and_refresh_without_breaking_old_reads() {
    for modern in [false, true] {
        let t = fixture(modern, false).await;
        let old = t.c.describe(&t.device.id).unwrap();
        assert!(old.capabilities.iter().all(|c| c.events.is_empty()));
        let enabled = Arc::new(adapter(&t.f, &t.p).with_event_history());
        let c = Core::new(
            t.f.store.clone(),
            vec![enabled.clone()],
            Arc::new(SystemClock),
            Policy::default(),
        );
        assert!(
            c.inspect_event_history_options(
                &t.device.id,
                &t.device.connections[0].id,
                &operation()
            )
            .await
            .unwrap()
            .options
            .is_empty()
        );
        assert!(t.p.reads.lock().unwrap().is_empty());
        assert!(t.f.cloud.calls.lock().unwrap().is_empty());
        c.read(
            &t.device.id,
            &t.device.components[0].id,
            Route::Cloud,
            false,
            &operation(),
        )
        .await
        .unwrap();
        let report = if modern {
            enabled
                .refresh_modern_devices(t.record.id, &operation())
                .await
                .unwrap()
        } else {
            enabled
                .refresh_devices(t.record.id, &operation())
                .await
                .unwrap()
        };
        t.f.publish(&report).await;
        let new = c.describe(&t.device.id).unwrap();
        assert_eq!(old.device.connections, new.device.connections);
        for old_cap in &old.capabilities {
            assert_eq!(
                serde_json::to_value(old_cap).unwrap(),
                serde_json::to_value(
                    new.capability(old_cap.component_id.as_deref().unwrap(), &old_cap.id)
                        .unwrap()
                )
                .unwrap()
            );
        }
        assert_eq!(
            new.capabilities
                .iter()
                .map(|c| c.events.len())
                .sum::<usize>(),
            20
        );
        assert!(
            new.device
                .components
                .iter()
                .any(|c| c.id == "family_guard-16")
        );
        assert!(
            new.capabilities
                .iter()
                .any(|c| !c.events.is_empty() && c.properties.is_empty())
        );
        let device = new.device;
        let without = adapter(&t.f, &t.p);
        assert!(
            without
                .event_history_options(&device, &device.connections[0], &operation())
                .await
                .unwrap()
                .is_empty()
        );
        let options = c
            .inspect_event_history_options(&device.id, &device.connections[0].id, &operation())
            .await
            .unwrap();
        let request = EventHistoryRequest {
            schema_version: "1.0".into(),
            device_id: device.id.clone(),
            connection_id: device.connections[0].id.clone(),
            target: options.options[0].target.clone(),
            projection: options.options[0].projection,
            start: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            end: chrono::DateTime::from_timestamp(1700086400, 0).unwrap(),
        };
        assert!(
            without
                .read_event_history(&device, &device.connections[0], &request, &operation())
                .await
                .is_err()
        );
        // Reopening without event opt-in still admits existing scalar reads on the complete enriched layout.
        without
            .read(
                &device,
                &t.device.components[0].id,
                &device.connections[0],
                &operation(),
            )
            .await
            .unwrap();
    }
}
#[tokio::test]
async fn event_history_independently_rejects_identity_target_window_and_partial_component_changes()
{
    for modern in [false, true] {
        let t = fixture(modern, true).await;
        let q = query(&t).await;
        for mode in 0..12 {
            let mut q = q.clone();
            let mut device = t.device.clone();
            let mut connection = device.connections[0].clone();
            match mode {
                0 => q.target.event = "foreign".into(),
                1 => q.target.capability = "power".into(),
                2 => q.device_id = "foreign".into(),
                3 => q.connection_id = "foreign".into(),
                4 => q.start += chrono::Duration::milliseconds(1),
                5 => q.end = q.start + chrono::Duration::days(33),
                6 => device.model = "foreign".into(),
                7 => connection.identity_ref = Uuid::new_v4().to_string(),
                8 => {
                    device.components.pop();
                }
                9 => q.schema_version = "2.0".into(),
                10 => q.end = chrono::Utc::now() + chrono::Duration::days(1),
                _ => q.end = q.start,
            }
            assert!(
                t.a.read_event_history(&device, &connection, &q, &operation())
                    .await
                    .is_err(),
                "mode {mode}"
            );
        }
        let mut saved = payload(&t.f, &t.record);
        saved["bindings"][0]["listed"] = json!(false);
        t.f.secrets.values.lock().unwrap().insert(
            t.record.credential_ref.to_string(),
            serde_json::to_vec(&saved).unwrap(),
        );
        assert!(
            t.c.inspect_event_history_options(
                &t.device.id,
                &t.device.connections[0].id,
                &operation()
            )
            .await
            .unwrap()
            .options
            .is_empty()
        );
        assert!(t.c.read_event_history(&q, &operation()).await.is_err());
        assert!(t.p.reads.lock().unwrap().is_empty());
        assert!(t.f.cloud.calls.lock().unwrap().is_empty());
    }
}
#[tokio::test]
async fn event_history_releases_account_lease_and_device_lock_on_cancel_deadline_and_drop() {
    for modern in [false, true] {
        for mode in 0..3 {
            let t = fixture(modern, true).await;
            let q = query(&t).await;
            let before = bytes(&t.f, &t.record);
            let writes = puts(&t.f);
            let mut op = operation();
            t.p.pause.store(2, Ordering::SeqCst);
            t.f.cloud.stall_read.store(1, Ordering::SeqCst);
            if mode == 1 {
                op.deadline = tokio::time::Instant::now() + Duration::from_secs(1);
            }
            let mut work = Box::pin(t.c.read_event_history(&q, &op));
            if modern {
                tokio::select! {_=t.p.started.notified()=>{},r=&mut work=>panic!("Unexpected {r:?}")}
            } else {
                tokio::select! {_=t.f.cloud.read_gate.notified()=>{},r=&mut work=>panic!("Unexpected {r:?}")}
            }
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
            assert!(t.f.vault.lease(t.record.id, &operation()).await.is_ok());
            assert!(t.f.store.lock_device(&t.device.id).is_ok());
            assert_eq!(bytes(&t.f, &t.record), before);
            assert_eq!(puts(&t.f), writes);
            assert_eq!(
                t.p.reads.lock().unwrap().len() + t.f.cloud.calls.lock().unwrap().len(),
                1
            );
        }
    }
}
#[tokio::test]
async fn event_history_missing_modern_transport_and_provider_failure_never_fall_back_or_renew() {
    let t = fixture(true, true).await;
    let q = query(&t).await;
    let absent = MijiaAdapter::new(t.f.vault.clone(), t.f.cloud.clone(), Arc::new(SystemClock))
        .with_event_history();
    assert!(
        absent
            .event_history_options(&t.device, &t.device.connections[0], &operation())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        absent
            .read_event_history(&t.device, &t.device.connections[0], &q, &operation())
            .await
            .is_err()
    );
    let before = bytes(&t.f, &t.record);
    let auth = t.p.auth.lock().unwrap().len();
    t.p.fail_read.store(1, Ordering::SeqCst);
    assert!(t.c.read_event_history(&q, &operation()).await.is_err());
    assert_eq!(*t.p.reads.lock().unwrap(), vec![Read::EventHistory]);
    assert!(t.f.cloud.calls.lock().unwrap().is_empty());
    assert_eq!(t.p.auth.lock().unwrap().len(), auth);
    assert_eq!(bytes(&t.f, &t.record), before);
}

#[tokio::test]
async fn event_history_rejects_unconfirmed_accounts_foreign_protected_models_and_child_roots() {
    for mode in 0..4 {
        let t = fixture(true, true).await;
        let q = query(&t).await;
        let before = bytes(&t.f, &t.record);
        let writes = puts(&t.f);
        if mode == 0 {
            t.f.secrets.mode.store(1, Ordering::SeqCst);
        } else {
            let mut saved = payload(&t.f, &t.record);
            match mode {
                1 => saved["bindings"][0]["mapping"]["model"] = json!("foreign"),
                2 => saved["bindings"][0]["modern"]["parent"] = json!("PRIVATE-PARENT"),
                _ => saved["bindings"][0]["modern"]["kind"] = json!(21),
            }
            t.f.secrets.values.lock().unwrap().insert(
                t.record.credential_ref.to_string(),
                serde_json::to_vec(&saved).unwrap(),
            );
        }
        assert!(t.c.read_event_history(&q, &operation()).await.is_err());
        assert!(t.p.reads.lock().unwrap().is_empty());
        assert!(t.f.cloud.calls.lock().unwrap().is_empty());
        assert_eq!(puts(&t.f), writes);
        if mode == 0 {
            assert_eq!(bytes(&t.f, &t.record), before);
        }
    }
}
#[tokio::test]
async fn event_history_late_modern_reply_is_discarded_without_storage_or_retry() {
    let t = fixture(true, true).await;
    let q = query(&t).await;
    let op = operation();
    let before = bytes(&t.f, &t.record);
    let writes = puts(&t.f);
    let cancellation = op.cancellation.clone();
    *t.p.hook.lock().unwrap() = Some(Box::new(move || cancellation.cancel()));
    assert_eq!(
        t.c.read_event_history(&q, &op).await.unwrap_err().code,
        "CANCELLED"
    );
    assert_eq!(*t.p.reads.lock().unwrap(), vec![Read::EventHistory]);
    assert!(t.f.cloud.calls.lock().unwrap().is_empty());
    assert_eq!(bytes(&t.f, &t.record), before);
    assert_eq!(puts(&t.f), writes);
    assert!(t.f.vault.lease(t.record.id, &operation()).await.is_ok());
}
