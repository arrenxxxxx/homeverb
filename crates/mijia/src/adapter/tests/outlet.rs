use super::*;
use homeverb_core::Grant;
use homeverb_domain::{Command, Risk, Status, Transport, WaitFor};

async fn outlet() -> (Fixture, Arc<MijiaAdapter>, Device) {
    let f = Fixture::new();
    *f.cloud.devices.lock().unwrap() =
        vec![json!({"did":"FICTIONAL-OUTLET", "name":"Fictional outlet", "model":"cuco.plug.v3"})];
    let report = f.connect().await;
    f.publish(&report).await;
    {
        let values = f.secrets.values.lock().unwrap();
        let payload: Value = serde_json::from_slice(values.values().next().unwrap()).unwrap();
        assert!(
            payload["bindings"][0]["mapping"]["properties"]
                .as_array()
                .unwrap()
                .iter()
                .all(|p| p["write_domain"].is_null())
        );
    }
    let first = f.core.list().unwrap().pop().unwrap();
    let local = f
        .adapter
        .inspect_control_options_with_outlets(&first, &first.connections[0], &operation())
        .await
        .unwrap();
    assert_eq!(local.len(), 4);
    let adapter = Arc::new(
        MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
            .with_outlet_controls(),
    );
    let report = adapter
        .refresh_devices(report.account.id, &operation())
        .await
        .unwrap();
    f.publish(&report).await;
    let device = f.core.list().unwrap().pop().unwrap();
    (f, adapter, device)
}
fn command(device: &Device, component: &str, property: &str, value: Value, id: &str) -> Command {
    let option = crate::outlet_control_option(component, property).unwrap();
    Command {
        schema_version: "1.0".into(),
        request_id: id.into(),
        device_id: device.id.clone(),
        component_id: component.into(),
        capability: option.capability.clone(),
        action: option.action.clone(),
        arguments: json!({property:value}),
        connection: Route::Cloud,
        timeout_ms: 5000,
        wait_for: WaitFor::Accepted,
    }
}
fn core(f: &Fixture, adapter: Arc<MijiaAdapter>, c: &Command) -> Core {
    Core::new(
        f.store.clone(),
        vec![adapter],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: c.device_id.clone(),
                component_id: c.component_id.clone(),
                capability: c.capability.clone(),
                action: c.action.clone(),
                transports: vec![Transport::Cloud],
                argument_schema: json!({"const":c.arguments}),
            }],
        },
    )
}

#[tokio::test]
async fn outlet_inspection_is_explicit_local_and_keeps_legacy_native_scopes() {
    let (f, adapter, device) = outlet().await;
    let before = f.cloud.calls.lock().unwrap().len();
    assert!(
        adapter
            .inspect_control_options(&device, &device.connections[0], &operation())
            .await
            .unwrap()
            .is_empty()
    );
    let options = f
        .adapter
        .inspect_control_options_with_outlets(&device, &device.connections[0], &operation())
        .await
        .unwrap();
    assert_eq!(options.len(), 4);
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../../examples/capability.mijia-outlet-power.json"
    ))
    .unwrap();
    let descriptor = f.core.describe(&device.id).unwrap();
    let power = descriptor
        .capabilities
        .iter()
        .find(|c| c.id == "power" && c.component_id.as_deref() == Some("switch-2"))
        .unwrap();
    assert_eq!(serde_json::to_value(power).unwrap(), expected);
    assert_eq!(
        options
            .iter()
            .filter(|o| o.definition.risk == Risk::Elevated)
            .count(),
        3
    );
    assert_eq!(f.cloud.calls.lock().unwrap().len(), before);
    assert!(crate::reference_setting_option("switch-2", "state").is_none());
    assert!(crate::reference_setting_option("indicator_light-13", "on_1").is_none());
}

#[tokio::test]
async fn every_outlet_target_has_independent_grants_and_exact_wire_values() {
    for (component, property, siid, piid, values) in [
        ("switch-2", "state", 2, 1, vec![json!(false), json!(true)]),
        (
            "switch-2",
            "default_power_on_state_2",
            2,
            2,
            vec![json!(0), json!(1), json!(2)],
        ),
        (
            "physical_controls_locked-7",
            "physical_controls_locked_1",
            7,
            1,
            vec![json!(false), json!(true)],
        ),
        (
            "indicator_light-13",
            "on_1",
            13,
            1,
            vec![json!(false), json!(true)],
        ),
    ] {
        for value in values {
            let (f, adapter, device) = outlet().await;
            let mut c = command(&device, component, property, value.clone(), "send");
            let elevated = crate::outlet_control_option(component, property)
                .unwrap()
                .definition
                .risk
                == Risk::Elevated;
            let admitted = core(&f, adapter.clone(), &c);
            let no_grant = Core::new(
                f.store.clone(),
                vec![adapter.clone()],
                Arc::new(SystemClock),
                Policy::default(),
            );
            assert_eq!(
                no_grant
                    .execute(c.clone(), CancellationToken::new())
                    .await
                    .status,
                Status::Failed
            );
            for other in [
                f.adapter.clone(),
                Arc::new(
                    MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
                        .with_fan_controls()
                        .with_reference_controls(),
                ),
            ] {
                let denied = core(&f, other, &c);
                c.request_id = Uuid::new_v4().to_string();
                let result = if elevated {
                    denied
                        .execute_user_approved(
                            denied.prepare_user_approval(c.clone()).unwrap(),
                            &operation(),
                        )
                        .await
                } else {
                    denied.execute(c.clone(), CancellationToken::new()).await
                };
                assert_eq!(result.status, Status::Failed);
            }
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
            c.request_id = "ordinary".into();
            if elevated {
                assert!(admitted.plan(&c).is_err());
                assert_eq!(
                    admitted
                        .execute(c.clone(), CancellationToken::new())
                        .await
                        .status,
                    Status::Failed
                );
            }
            c.request_id = "accepted".into();
            let result = if elevated {
                let preview = admitted.prepare_user_approval(c.clone()).unwrap();
                assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
                admitted.execute_user_approved(preview, &operation()).await
            } else {
                admitted.execute(c.clone(), CancellationToken::new()).await
            };
            assert_eq!(result.status, Status::Accepted);
            assert_eq!(
                f.cloud.last_write.lock().unwrap().as_ref().unwrap(),
                &json!({"did":"FICTIONAL-OUTLET","siid":siid,"piid":piid,"value":value})
            );
            admitted.execute(c.clone(), CancellationToken::new()).await;
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 1);
            assert!(admitted.prepare_user_approval(c).is_err());
        }
    }
}

#[tokio::test]
async fn public_low_risk_metadata_cannot_bypass_protected_outlet_approval() {
    for (component, property, value) in [
        ("switch-2", "state", json!(true)),
        ("switch-2", "default_power_on_state_2", json!(1)),
        (
            "physical_controls_locked-7",
            "physical_controls_locked_1",
            json!(true),
        ),
    ] {
        let (f, adapter, device) = outlet().await;
        let c = command(&device, component, property, value, "forged-risk");
        let mut descriptor = f.core.describe(&device.id).unwrap();
        let capability = descriptor
            .capabilities
            .iter_mut()
            .find(|cap| cap.id == c.capability && cap.component_id.as_deref() == Some(component))
            .unwrap();
        capability.actions.get_mut(&c.action).unwrap().risk = Risk::Low;
        f.store.save_device(&descriptor).unwrap();
        let admitted = core(&f, adapter, &c);
        assert!(admitted.plan(&c).is_ok());
        let result = admitted.execute(c, CancellationToken::new()).await;
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
        assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
    }
}

#[tokio::test]
async fn outlet_approval_lifecycle_and_lost_replies_never_replay() {
    for mode in [0, 1, 2, 3] {
        let (f, adapter, device) = outlet().await;
        let c = command(&device, "switch-2", "state", json!(true), "outlet-outcome");
        let admitted = core(&f, adapter, &c);
        let preview = admitted.prepare_user_approval(c.clone()).unwrap();
        if mode == 0 {
            admitted.invalidate_user_approvals();
            assert_eq!(
                admitted
                    .execute_user_approved(preview, &operation())
                    .await
                    .status,
                Status::Failed
            );
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
            continue;
        }
        f.cloud.write_mode.store(mode, Ordering::SeqCst);
        let op = operation();
        let work = admitted.execute_user_approved(preview, &op);
        tokio::pin!(work);
        if mode == 3 {
            tokio::select! { _=f.cloud.read_gate.notified()=>{}, _=&mut work=>panic!("Expected an in-flight write") }
            op.cancellation.cancel();
        }
        let result = work.await;
        assert_eq!(
            result.status,
            if mode == 2 {
                Status::Failed
            } else {
                Status::Unknown
            }
        );
        admitted.execute(c.clone(), CancellationToken::new()).await;
        assert!(admitted.prepare_user_approval(c).is_err());
        assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 1);
    }
}

#[tokio::test]
async fn outlet_argument_and_protected_binding_changes_refuse_before_send() {
    for mutation in [
        "argument",
        "connection",
        "missing_grant",
        "source",
        "binding",
        "unlisted",
    ] {
        let (f, adapter, device) = outlet().await;
        let mut c = command(
            &device,
            "switch-2",
            "default_power_on_state_2",
            json!(2),
            "refused",
        );
        let admitted = core(&f, adapter, &c);
        if mutation == "argument" {
            c.arguments = json!({"default_power_on_state_2":3});
        }
        if mutation == "connection" {
            c.connection = Route::Lan;
        }
        if mutation == "missing_grant" {
            c.arguments = json!({"default_power_on_state_2":1});
        }
        if matches!(mutation, "argument" | "connection" | "missing_grant") {
            assert!(admitted.prepare_user_approval(c).is_err());
        } else {
            let preview = admitted.prepare_user_approval(c).unwrap();
            {
                let mut values = f.secrets.values.lock().unwrap();
                let bytes = values.values_mut().next().unwrap();
                let mut payload: Value = serde_json::from_slice(bytes).unwrap();
                match mutation {
                    "source" => {
                        payload["bindings"][0]["mapping"]["properties"][0]["piid"] = json!(999)
                    }
                    "binding" => {
                        payload["bindings"][0]["connection_id"] = json!(Uuid::new_v4().to_string())
                    }
                    _ => payload["bindings"][0]["listed"] = json!(false),
                }
                *bytes = serde_json::to_vec(&payload).unwrap();
            }
            assert_eq!(
                admitted
                    .execute_user_approved(preview, &operation())
                    .await
                    .status,
                Status::Failed
            );
        }
        assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
    }
}
