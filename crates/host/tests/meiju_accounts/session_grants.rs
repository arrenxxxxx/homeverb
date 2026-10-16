use super::*;

#[tokio::test(start_paused = true)]
async fn revoked_sessions_report_empty_grants_without_refresh_or_secret_access() {
    let mut f = Fixture::new();
    f.connect(None).await;
    let account = f.store.accounts().unwrap().remove(0);
    let device = f
        .store
        .devices()
        .unwrap()
        .into_iter()
        .find(|d| d.device.model == "KFR-35G/WF2")
        .unwrap()
        .device;
    let request = json!({"method":"inspect_session_grants","device_id":device.id,"connection_id":device.connections[0].id});
    let expected = json!({"device_id":device.id,"connection_id":device.connections[0].id,
        "transport":"cloud","source":"session_policy","provider_authorization":"not_checked",
        "grants":[{"component_id":"main","capability":"power","action":"set","argument_schema":{"const":{"state":false}}}]});
    assert_eq!(f.call(request.clone()).await["data"]["grants"], json!([]));
    f.reopen_with_power_grants(&[MeijuPowerGrant {
        device_id: device.id.clone(),
        state: false,
    }]);
    // Session policy changes before inventory refresh or descriptor publication.
    assert_eq!(f.call(request.clone()).await["data"], expected);
    f.refresh(&account.id.to_string()).await;
    let command = power_command(&device.id, false);
    let receipt = f.call(json!({"method":"execute","command":command})).await;
    assert_eq!(receipt["status"], "accepted");
    let descriptors = json!(f.store.devices().unwrap());
    let saved = f
        .store
        .devices()
        .unwrap()
        .into_iter()
        .find(|descriptor| descriptor.device.id == device.id)
        .unwrap();
    assert!(
        saved
            .capability("main", "power")
            .unwrap()
            .actions
            .contains_key("set")
    );
    let calls = f.calls();
    let secrets = f.secrets.values.lock().unwrap().clone();
    let reads = f.secrets.reads.load(Ordering::Relaxed);
    f.secrets.unavailable.store(true, Ordering::Relaxed);
    let busy = f.vault.lease(account.id, &operation()).await.unwrap();
    assert_eq!(f.call(request.clone()).await["data"], expected);
    f.reopen_with_power_grants(&[]);
    let revoked = f.call(request.clone()).await;
    assert_eq!(revoked["data"]["grants"], json!([]));
    assert_eq!(json!(f.store.devices().unwrap()), descriptors);
    assert_eq!(
        f.call(json!({"method":"plan","command":power_command(&device.id,false)}))
            .await["error"]["code"],
        "POLICY_DENIED"
    );
    assert_eq!(
        f.call(json!({"method":"get_execution","request_id":command["request_id"]}))
            .await["data"],
        receipt
    );
    assert_eq!(f.secrets.reads.load(Ordering::Relaxed), reads);
    assert_eq!(*f.secrets.values.lock().unwrap(), secrets);
    assert_eq!(f.calls(), calls);
    f.session.set_foreground(false);
    assert_eq!(
        f.call(request.clone()).await["error"]["code"],
        "HOST_BACKGROUND"
    );
    f.session.set_foreground(true);
    assert_eq!(f.call(request.clone()).await, revoked);
    f.session.close();
    assert_eq!(f.call(request).await["error"]["code"], "HOST_CLOSED");
    drop(busy);
}
