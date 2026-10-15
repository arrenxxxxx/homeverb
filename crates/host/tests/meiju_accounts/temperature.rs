use super::*;
use homeverb_host::{MeijuTemperatureGrant, meiju_temperature_policy};

fn request(device: &str) -> Value {
    let mut value = power_command(device, true);
    value["capability"] = json!("climate");
    value["action"] = json!("set");
    value["arguments"] = json!({"target_temperature":26.5});
    value
}

async fn connected() -> (Fixture, String, Uuid) {
    let mut f = Fixture::new();
    *f.wire.product_fields.lock().unwrap() = Some(json!({"modelNumber":1,"sn8":"22397091"}));
    let result = f.connect(None).await;
    let account = Uuid::parse_str(result["data"]["account"]["id"].as_str().unwrap()).unwrap();
    let device = f
        .store
        .devices()
        .unwrap()
        .into_iter()
        .find(|d| d.device.model == "KFR-35G/WF2")
        .unwrap()
        .device
        .id;
    let before = f.calls();
    let reads = f.secrets.reads.load(Ordering::Relaxed);
    f.session.close();
    let adapter = Arc::new(
        MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock))
            .with_ac_temperature_control(),
    );
    let policy = meiju_temperature_policy(&[MeijuTemperatureGrant {
        device_id: device.clone(),
        target_temperature: 26.5,
    }])
    .unwrap();
    let core = Core::new(
        f.store.clone(),
        vec![adapter.clone()],
        Arc::new(SystemClock),
        policy,
    );
    f.session = Arc::new(Session::with_accounts(core, adapter));
    assert_eq!(f.calls(), before);
    assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
    f.refresh(&account.to_string()).await;
    (f, device, account)
}

#[tokio::test]
async fn temperature_sessions_keep_permissions_outcomes_and_receipts_separate() {
    let (mut f, device, _) = connected().await;
    assert!(
        f.call(json!({"method":"plan","command":request(&device)}))
            .await["error"]
            .is_null()
    );
    let mut other = request(&device);
    other["arguments"]["target_temperature"] = json!(27);
    assert_eq!(
        f.call(json!({"method":"plan","command":other})).await["error"]["code"],
        "POLICY_DENIED"
    );
    assert!(
        f.call(json!({"method":"plan","command":power_command(&device,true)}))
            .await["error"]
            .is_object()
    );
    let mut receipts = Vec::new();
    for (code, status) in [(0, "accepted"), (1306, "unknown"), (7400, "failed")] {
        f.wire.control_code.store(code, Ordering::Relaxed);
        let command = request(&device);
        let before = f.calls().len();
        let result = f.call(json!({"method":"execute","command":command})).await;
        assert_eq!(result["status"], status);
        assert!(result["verification"].is_null());
        assert_eq!(
            f.call(json!({"method":"execute","command":command})).await,
            result
        );
        assert_eq!(f.calls().len(), before + 1);
        receipts.push((command, result));
    }
    f.session.close();
    let adapter = Arc::new(MeijuAdapter::new(
        f.vault.clone(),
        f.wire.clone(),
        Arc::new(SystemClock),
    ));
    f.session = Arc::new(Session::with_accounts(
        Core::new(
            f.store.clone(),
            vec![adapter.clone()],
            Arc::new(SystemClock),
            Policy::default(),
        ),
        adapter,
    ));
    assert_eq!(
        f.call(json!({"method":"plan","command":request(&device)}))
            .await["error"]["code"],
        "POLICY_DENIED"
    );
    let before = f.calls();
    for (command, result) in receipts {
        assert_eq!(
            f.call(json!({"method":"get_execution","request_id":command["request_id"]}))
                .await["data"],
            result
        );
    }
    assert_eq!(f.calls(), before);
}

#[tokio::test(start_paused = true)]
async fn temperature_interruption_does_not_restore_power_or_replay_and_releases_leases() {
    for stop in ["cancel", "background", "close", "deadline"] {
        let (f, device, account) = connected().await;
        *f.wire.delay.lock().unwrap() = Some(CloudEndpoint::ApplianceControl);
        let work = f.call_id(
            "temperature",
            json!({"method":"execute","command":request(&device)}),
        );
        tokio::pin!(work);
        tokio::select! {
            _=f.wire.started.notified()=>(),
            result=&mut work=>panic!("Expected an in-flight temperature change: {result}"),
        }
        assert!(f.vault.lease(account, &operation()).await.is_err());
        let before = f.calls();
        match stop {
            "cancel" => {
                assert!(f.session.cancel("temperature"));
            }
            "background" => f.session.set_foreground(false),
            "close" => f.session.close(),
            _ => tokio::time::advance(std::time::Duration::from_secs(2)).await,
        }
        let result = work.await;
        assert_eq!(result["status"], "unknown");
        assert_eq!(result["attempts"][0]["phase"], "possibly_sent");
        drop(f.vault.lease(account, &operation()).await.unwrap());
        assert_eq!(f.calls(), before);
    }
}
