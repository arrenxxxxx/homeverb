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

#[tokio::test]
async fn reference_fan_settings_keep_exact_grants_wire_ids_and_uncertain_receipts() {
    let settings = [
        (
            "horizontal_swing_included_angle_6",
            6,
            vec![json!(60), json!(90), json!(120)],
        ),
        ("vertical_swing_7", 7, vec![json!(false), json!(true)]),
        (
            "vertical_swing_included_angle_8",
            8,
            vec![json!(60), json!(105), json!(120)],
        ),
        ("anion_9", 9, vec![json!(false), json!(true)]),
    ];
    for (name, piid, values) in settings {
        for value in values {
            let f = Fixture::new();
            *f.cloud.devices.lock().unwrap() = vec![
                json!({"did":"PRIVATE-FAN-IDENTITY","name":"Reference fan","model":"jdjz.fan.2072fc"}),
            ];
            let report = f.connect().await;
            f.publish(&report).await;
            let fan = f.core.list().unwrap().into_iter().next().unwrap();
            let mut request = command(&fan);
            request.action = format!("set_{name}");
            request.arguments = json!({name:value});
            let policy = Policy {
                grants: vec![Grant {
                    device_id: fan.id.clone(),
                    component_id: "fan-2".into(),
                    capability: request.capability.clone(),
                    action: request.action.clone(),
                    transports: vec![Transport::Cloud],
                    argument_schema: json!({"const":request.arguments}),
                }],
            };
            let default = Core::new(
                f.store.clone(),
                vec![f.adapter.clone()],
                Arc::new(SystemClock),
                policy.clone(),
            );
            assert_eq!(
                default
                    .execute(request.clone(), CancellationToken::new())
                    .await
                    .status,
                Status::Failed
            );
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
            let adapter = controlled(&f, report.account.id).await;
            let denied = Core::new(
                f.store.clone(),
                vec![adapter.clone()],
                Arc::new(SystemClock),
                Policy::default(),
            );
            request.request_id = "no-grant".into();
            assert_eq!(
                denied
                    .execute(request.clone(), CancellationToken::new())
                    .await
                    .status,
                Status::Failed
            );
            let core = Core::new(
                f.store.clone(),
                vec![adapter],
                Arc::new(SystemClock),
                policy,
            );
            let mut wrong = request.clone();
            wrong.request_id = "wrong-domain".into();
            wrong.arguments = json!({name:180});
            assert_eq!(
                core.execute(wrong, CancellationToken::new()).await.status,
                Status::Failed
            );
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
            request.request_id = "admitted-reference".into();
            let receipt = core
                .execute(request.clone(), CancellationToken::new())
                .await;
            assert_eq!(receipt.status, Status::Accepted);
            assert_eq!(
                f.cloud.last_write.lock().unwrap().as_ref().unwrap(),
                &json!({"did":"PRIVATE-FAN-IDENTITY","siid":2,"piid":piid,"value":value})
            );
            core.execute(request.clone(), CancellationToken::new())
                .await;
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 1);
            request.request_id = "reference-lost-reply".into();
            f.cloud.write_mode.store(1, Ordering::SeqCst);
            assert_eq!(
                core.execute(request.clone(), CancellationToken::new())
                    .await
                    .status,
                Status::Unknown
            );
            core.execute(request, CancellationToken::new()).await;
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 2);
        }
    }
}

#[tokio::test]
async fn exact_light_and_purifier_settings_require_independent_admission_and_keep_wire_values() {
    let cases = [
        (
            "abhome.light.wy0a01",
            "light-2",
            "power",
            "state",
            2,
            1,
            vec![json!(false), json!(true)],
        ),
        (
            "abhome.light.wy0a01",
            "light-2",
            "vendor.xiaomi.mijia.light",
            "brightness_2",
            2,
            2,
            vec![json!(1), json!(50), json!(100)],
        ),
        (
            "abhome.light.wy0a01",
            "light-2",
            "vendor.xiaomi.mijia.light",
            "color_temperature_3",
            2,
            3,
            vec![json!(2700), json!(4321), json!(6500)],
        ),
        (
            "abhome.light.wy0a01",
            "light-2",
            "vendor.xiaomi.mijia.light",
            "mode_7",
            2,
            7,
            vec![json!(0), json!(4), json!(23)],
        ),
        (
            "abhome.light.wy0a01",
            "light-2",
            "vendor.xiaomi.mijia.light",
            "default_power_on_state_9",
            2,
            9,
            vec![json!(0), json!(1), json!(2)],
        ),
        (
            "abhome.light.wy0a01",
            "light-2",
            "vendor.xiaomi.mijia.light",
            "flex_switch_12",
            2,
            12,
            vec![json!(false), json!(true)],
        ),
        (
            "abhome.light.wy0a01",
            "light-2",
            "vendor.xiaomi.mijia.light",
            "night_light_switch_13",
            2,
            13,
            vec![json!(false), json!(true)],
        ),
        (
            "zhimi.airpurifier.v6",
            "air_purifier-2",
            "power",
            "state",
            2,
            1,
            vec![json!(false), json!(true)],
        ),
        (
            "zhimi.airpurifier.v6",
            "air_purifier-2",
            "vendor.xiaomi.mijia.air_purifier",
            "fan_level_2",
            2,
            2,
            vec![json!(0), json!(1), json!(2)],
        ),
        (
            "zhimi.airpurifier.v6",
            "air_purifier-2",
            "vendor.xiaomi.mijia.air_purifier",
            "mode_3",
            2,
            3,
            vec![json!(0), json!(1), json!(2)],
        ),
        (
            "zhimi.airpurifier.v6",
            "indicator_light-5",
            "vendor.xiaomi.mijia.indicator_light",
            "on_1",
            5,
            1,
            vec![json!(false), json!(true)],
        ),
        (
            "zhimi.airpurifier.v6",
            "alarm-6",
            "vendor.xiaomi.mijia.alarm",
            "alarm_1",
            6,
            1,
            vec![json!(false), json!(true)],
        ),
        (
            "zhimi.airpurifier.v6",
            "physical_controls_locked-7",
            "vendor.xiaomi.mijia.physical_controls_locked",
            "physical_controls_locked_1",
            7,
            1,
            vec![json!(false), json!(true)],
        ),
        (
            "zhimi.airpurifier.v6",
            "air_purifier_favorite-8",
            "vendor.xiaomi.mijia.air_purifier_favorite",
            "favorite_fan_level_1",
            8,
            1,
            vec![json!(0), json!(7), json!(16)],
        ),
    ];
    for (model, component, capability, property, siid, piid, values) in cases {
        let f = Fixture::new();
        *f.cloud.devices.lock().unwrap() = vec![
            json!({"did":"FICTIONAL-REFERENCE-DEVICE","name":"Reference appliance","model":model}),
        ];
        let report = f.connect().await;
        f.publish(&report).await;
        let device = f.core.list().unwrap().into_iter().next().unwrap();
        let adapter = Arc::new(
            MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
                .with_reference_controls(),
        );
        let refresh = adapter
            .refresh_devices(report.account.id, &operation())
            .await
            .unwrap();
        f.publish(&refresh).await;
        for (index, value) in values.iter().enumerate() {
            let mut request = command(&device);
            request.request_id = format!("setting-{index}");
            request.component_id = component.into();
            request.capability = capability.into();
            request.action = if capability == "power" {
                "set".into()
            } else {
                format!("set_{property}")
            };
            request.arguments = json!({property:value});
            let policy = Policy {
                grants: vec![Grant {
                    device_id: device.id.clone(),
                    component_id: component.into(),
                    capability: capability.into(),
                    action: request.action.clone(),
                    transports: vec![Transport::Cloud],
                    argument_schema: json!({"const":request.arguments}),
                }],
            };
            for (admitted, grants) in [
                (f.adapter.clone(), policy.clone()),
                (
                    Arc::new(
                        MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
                            .with_fan_controls(),
                    ),
                    policy.clone(),
                ),
                (adapter.clone(), Policy::default()),
            ] {
                let denied = Core::new(
                    f.store.clone(),
                    vec![admitted],
                    Arc::new(SystemClock),
                    grants,
                );
                assert_eq!(
                    denied
                        .execute(request.clone(), CancellationToken::new())
                        .await
                        .status,
                    Status::Failed
                );
            }
            let before = f.cloud.count(CloudEndpoint::SetProperty);
            let core = Core::new(
                f.store.clone(),
                vec![adapter.clone()],
                Arc::new(SystemClock),
                policy,
            );
            let mut wrong = request.clone();
            wrong.request_id = format!("invalid-{index}");
            wrong.arguments = json!({property:"invalid"});
            assert_eq!(
                core.execute(wrong, CancellationToken::new()).await.status,
                Status::Failed
            );
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), before);
            request.request_id = format!("accepted-{index}");
            assert_eq!(
                core.execute(request.clone(), CancellationToken::new())
                    .await
                    .status,
                Status::Accepted
            );
            assert_eq!(
                f.cloud.last_write.lock().unwrap().as_ref().unwrap(),
                &json!({"did":"FICTIONAL-REFERENCE-DEVICE","siid":siid,"piid":piid,"value":value})
            );
            core.execute(request.clone(), CancellationToken::new())
                .await;
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), before + 1);
            request.request_id = format!("lost-{index}");
            f.cloud.write_mode.store(1, Ordering::SeqCst);
            assert_eq!(
                core.execute(request.clone(), CancellationToken::new())
                    .await
                    .status,
                Status::Unknown
            );
            core.execute(request, CancellationToken::new()).await;
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), before + 2);
            f.cloud.write_mode.store(0, Ordering::SeqCst);
        }
    }
}

#[tokio::test]
async fn reference_admission_cannot_enable_fan_writes() {
    let f = Fixture::new();
    let report = f.connect().await;
    f.publish(&report).await;
    controlled(&f, report.account.id).await;
    let fan = f.fan();
    let adapter = Arc::new(
        MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
            .with_reference_controls(),
    );
    let core = Core::new(
        f.store.clone(),
        vec![adapter],
        Arc::new(SystemClock),
        policy(&fan),
    );
    assert_eq!(
        core.execute(command(&fan), CancellationToken::new())
            .await
            .status,
        Status::Failed
    );
    assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
}
