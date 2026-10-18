use super::*;
use homeverb_core::{Core, Grant, Policy, Store, SystemClock};
use homeverb_domain::{Command, Phase, Route, Status, WaitFor};
use serde_json::json;
use std::sync::atomic::Ordering;
use tokio_util::sync::CancellationToken;

fn command(device: &Device) -> Command {
    Command {
        schema_version: "1.0".into(),
        request_id: "fictional-fan-write".into(),
        device_id: device.id.clone(),
        component_id: "fan-2".into(),
        capability: "vendor.xiaomi.mijia.fan".into(),
        action: "set_level".into(),
        arguments: json!({"level":3}),
        connection: Route::Cloud,
        timeout_ms: 5000,
        wait_for: WaitFor::Accepted,
    }
}
fn policy(device: &Device) -> Policy {
    Policy {
        grants: vec![Grant {
            device_id: device.id.clone(),
            component_id: "fan-2".into(),
            capability: "vendor.xiaomi.mijia.fan".into(),
            action: "set_level".into(),
            transports: vec![Transport::Cloud],
            argument_schema: json!({"const":{"level":3}}),
        }],
    }
}
async fn controlled(f: &Fixture, account: Uuid) -> Arc<MijiaAdapter> {
    let adapter = Arc::new(
        MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
            .with_fan_controls(),
    );
    let report = adapter
        .refresh_devices(account, &operation())
        .await
        .unwrap();
    f.publish(&report).await;
    adapter
}

#[tokio::test]
async fn inspection_is_local_and_never_publishes_or_enables_actions() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let fan = f.fan();
    let before = f.cloud.calls.lock().unwrap().len();
    let options = f
        .adapter
        .inspect_control_options(&fan, &fan.connections[0], &operation())
        .await
        .unwrap();
    assert_eq!(options.len(), 2);
    assert_eq!(
        options[1].definition.parameters["properties"]["level"]["maximum"],
        4.0
    );
    assert_eq!(f.cloud.calls.lock().unwrap().len(), before);
    assert!(
        f.core
            .describe(&fan.id)
            .unwrap()
            .capabilities
            .iter()
            .all(|c| c.actions.is_empty())
    );
    let mut foreign = fan.connections[0].clone();
    foreign.identity_ref = Uuid::new_v4().to_string();
    assert!(
        f.adapter
            .inspect_control_options(&fan, &foreign, &operation())
            .await
            .is_err()
    );
    assert_eq!(f.cloud.calls.lock().unwrap().len(), before);
}

#[tokio::test]
async fn exact_core_grant_is_required_and_acceptance_does_not_verify_or_replay() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let adapter = controlled(&f, report.account.id).await;
    let fan = f.fan();
    let no_grant = Core::new(
        f.store.clone(),
        vec![adapter.clone()],
        Arc::new(SystemClock),
        Policy::default(),
    );
    let receipt = no_grant
        .execute(command(&fan), CancellationToken::new())
        .await;
    assert_eq!(receipt.status, Status::Failed);
    assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
    let core = Core::new(
        f.store.clone(),
        vec![adapter],
        Arc::new(SystemClock),
        policy(&fan),
    );
    let mut request = command(&fan);
    request.request_id = "fictional-admitted".into();
    let receipt = core
        .execute(request.clone(), CancellationToken::new())
        .await;
    assert_eq!(receipt.status, Status::Accepted);
    assert_eq!(receipt.attempts[0].phase, Phase::Accepted);
    core.execute(request, CancellationToken::new()).await;
    assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 1);
    let mut ungranted = command(&fan);
    ungranted.request_id = "fictional-ungranted-value".into();
    ungranted.arguments = json!({"level":4});
    assert_eq!(
        core.execute(ungranted, CancellationToken::new())
            .await
            .status,
        Status::Failed
    );
    assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 1);
}

#[tokio::test]
async fn lost_reply_and_explicit_rejection_retain_distinct_receipts() {
    for (mode, status, phase) in [
        (1, Status::Unknown, Phase::PossiblySent),
        (2, Status::Failed, Phase::Rejected),
    ] {
        let f = Fixture::new();
        let report = f.connect().await;
        f.publish(&report).await;
        let adapter = controlled(&f, report.account.id).await;
        let fan = f.fan();
        f.cloud.write_mode.store(mode, Ordering::SeqCst);
        let core = Core::new(
            f.store.clone(),
            vec![adapter],
            Arc::new(SystemClock),
            policy(&fan),
        );
        let request = command(&fan);
        let receipt = core
            .execute(request.clone(), CancellationToken::new())
            .await;
        assert_eq!(receipt.status, status);
        assert_eq!(receipt.attempts[0].phase, phase);
        core.execute(request, CancellationToken::new()).await;
        assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 1);
    }
}

#[tokio::test]
async fn forged_public_domains_actions_and_connections_cannot_bypass_protected_admission() {
    for mode in 0..4 {
        let f = Fixture::new();
        let report = f.connect().await;
        f.publish(&report).await;
        let adapter = controlled(&f, report.account.id).await;
        let fan = f.fan();
        let mut descriptor = f.core.describe(&fan.id).unwrap();
        let cap = descriptor
            .capabilities
            .iter_mut()
            .find(|c| c.id == "vendor.xiaomi.mijia.fan")
            .unwrap();
        let mut request = command(&fan);
        let mut policy = policy(&fan);
        match mode {
            0 => {
                cap.actions.get_mut("set_level").unwrap().parameters["properties"]["level"]["maximum"] =
                    json!(200);
                request.arguments = json!({"level":100});
                policy.grants[0].argument_schema = json!({"const":{"level":100}});
            }
            1 => {
                let action = cap.actions.remove("set_level").unwrap();
                cap.actions.insert("unlock".into(), action);
                request.action = "unlock".into();
                policy.grants[0].action = "unlock".into();
            }
            2 => {
                descriptor.device.connections[0].identity_ref = Uuid::new_v4().to_string();
            }
            _ => {
                descriptor.device.connections[0].id = Uuid::new_v4().to_string();
            }
        }
        if mode >= 2 {
            assert_eq!(
                f.store.save_device(&descriptor).unwrap_err().code,
                "DEVICE_IDENTITY_CONFLICT"
            );
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
            continue;
        }
        f.store.save_device(&descriptor).unwrap();
        let core = Core::new(
            f.store.clone(),
            vec![adapter],
            Arc::new(SystemClock),
            policy,
        );
        let receipt = core.execute(request, CancellationToken::new()).await;
        assert_eq!(receipt.status, Status::Failed);
        assert_eq!(receipt.attempts[0].phase, Phase::NotSent);
        assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
    }
}

#[tokio::test]
async fn background_cancellation_after_send_keeps_unknown_and_releases_account_lease() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    let adapter = controlled(&f, report.account.id).await;
    let fan = f.fan();
    f.cloud.write_mode.store(3, Ordering::SeqCst);
    let core = Core::new(
        f.store.clone(),
        vec![adapter],
        Arc::new(SystemClock),
        policy(&fan),
    );
    let cancellation = CancellationToken::new();
    let send = core.execute(command(&fan), cancellation.clone());
    tokio::pin!(send);
    tokio::select! { _ = &mut send => panic!("Expected pending write"), _ = f.cloud.read_gate.notified() => {} }
    cancellation.cancel();
    let receipt = send.await;
    assert_eq!(receipt.status, Status::Unknown);
    assert_eq!(receipt.attempts[0].phase, Phase::PossiblySent);
    assert!(f.vault.lease(report.account.id, &operation()).await.is_ok());
    assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 1);
}
