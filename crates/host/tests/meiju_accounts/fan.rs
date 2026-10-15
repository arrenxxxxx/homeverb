use super::*;
use homeverb_host::{MeijuFanGrant, meiju_fan_policy};

fn reopen(f: &mut Fixture, grants: &[MeijuFanGrant]) {
    f.session.close();
    let policy = meiju_fan_policy(grants).unwrap();
    let mut adapter = MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock));
    if !grants.is_empty() {
        adapter = adapter.with_ac_fan_control();
    }
    let adapter = Arc::new(adapter);
    let core = Core::new(
        f.store.clone(),
        vec![adapter.clone()],
        Arc::new(SystemClock),
        policy,
    );
    f.session = Arc::new(Session::with_accounts(core, adapter));
}

fn command(device: &str, speed: u8) -> Value {
    let mut value = power_command(device, false);
    value["capability"] = json!("vendor.midea.ac");
    value["action"] = json!("set_fan_speed");
    value["arguments"] = json!({"wind_speed":speed});
    value
}

async fn connected() -> (Fixture, String, Uuid) {
    let f = Fixture::new();
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
    (f, device, account)
}

#[tokio::test(start_paused = true)]
async fn fan_sessions_keep_exact_permissions_receipts_and_read_only_reopening() {
    let (mut f, device, account) = connected().await;
    let before = f.calls();
    let reads = f.secrets.reads.load(Ordering::Relaxed);
    reopen(
        &mut f,
        &[MeijuFanGrant {
            device_id: device.clone(),
            wind_speed: 40,
        }],
    );
    assert_eq!(f.calls(), before);
    assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
    f.refresh(&account.to_string()).await;
    assert!(
        f.call(json!({"method":"plan","command":command(&device,40)}))
            .await["error"]
            .is_null()
    );
    assert_eq!(
        f.call(json!({"method":"plan","command":command(&device,60)}))
            .await["error"]["code"],
        "POLICY_DENIED"
    );
    assert_eq!(
        f.call(json!({"method":"plan","command":power_command(&device,true)}))
            .await["error"]["code"],
        "CAPABILITY_UNSUPPORTED"
    );
    let mut lan = command(&device, 40);
    lan["connection"] = json!("lan");
    assert_eq!(
        f.call(json!({"method":"plan","command":lan})).await["error"]["code"],
        "POLICY_DENIED"
    );
    let mut receipts = Vec::new();
    for (code, status) in [(0, "accepted"), (1306, "unknown"), (7400, "failed")] {
        f.wire.control_code.store(code, Ordering::Relaxed);
        let request = command(&device, 40);
        let calls = f.calls().len();
        let result = f.call(json!({"method":"execute","command":request})).await;
        assert_eq!(result["status"], status);
        assert!(result["verification"].is_null());
        assert_eq!(
            f.call(json!({"method":"execute","command":request})).await,
            result
        );
        assert_eq!(f.calls().len(), calls + 1);
        receipts.push((request, result));
    }
    let before = f.calls();
    reopen(&mut f, &[]);
    assert_eq!(
        f.call(json!({"method":"plan","command":command(&device,40)}))
            .await["error"]["code"],
        "POLICY_DENIED"
    );
    for (request, result) in receipts {
        assert_eq!(
            f.call(json!({"method":"get_execution","request_id":request["request_id"]}))
                .await["data"],
            result
        );
    }
    assert_eq!(f.calls(), before);
}

#[tokio::test(start_paused = true)]
async fn interrupted_fan_writes_stay_unknown_and_release_account_leases() {
    for stop in ["cancel", "background", "close", "deadline"] {
        let (mut f, device, account) = connected().await;
        reopen(
            &mut f,
            &[MeijuFanGrant {
                device_id: device.clone(),
                wind_speed: 40,
            }],
        );
        f.refresh(&account.to_string()).await;
        *f.wire.delay.lock().unwrap() = Some(CloudEndpoint::ApplianceControl);
        let work = f.call_id(
            "fan",
            json!({"method":"execute","command":command(&device,40)}),
        );
        tokio::pin!(work);
        tokio::select! {
            _ = f.wire.started.notified() => (),
            result = &mut work => panic!("Expected an in-flight control: {result}"),
        }
        assert!(f.vault.lease(account, &operation()).await.is_err());
        let calls = f.calls();
        match stop {
            "cancel" => {
                assert!(f.session.cancel("fan"));
            }
            "background" => f.session.set_foreground(false),
            "close" => f.session.close(),
            _ => tokio::time::advance(std::time::Duration::from_secs(2)).await,
        }
        let result = work.await;
        assert_eq!(result["status"], "unknown");
        assert_eq!(result["attempts"][0]["phase"], "possibly_sent");
        drop(f.vault.lease(account, &operation()).await.unwrap());
        assert_eq!(f.calls(), calls);
    }
}
