//! Exact additional outlet/switch setters, with fictional legacy transport.
use super::*;

type Case = (&'static str, &'static str, &'static str, u32, u32, usize);
const CASES: [Case; 7] = [
    ("opple.switch.0010d5", "switch-3", "state", 3, 1, 1),
    ("opple.switch.0010d6", "switch-11", "state", 11, 1, 2),
    ("opple.switch.0010d6", "switch-12", "state", 12, 1, 2),
    ("yutai.plug.jdls1", "switch-2", "state", 2, 1, 4),
    (
        "yutai.plug.jdls1",
        "switch-2",
        "default_power_on_state_2",
        2,
        2,
        4,
    ),
    ("yutai.plug.jdls1", "indicator_light-4", "on_1", 4, 1, 4),
    ("yutai.plug.jdls1", "custom-7", "child_lock_7", 7, 7, 4),
];
fn targets(case: Case) -> Vec<Value> {
    if case.2 == "default_power_on_state_2" {
        vec![json!(0), json!(1), json!(2)]
    } else {
        vec![json!(false), json!(true)]
    }
}

#[tokio::test]
async fn exact_additional_targets_keep_local_inspection_grants_and_single_property_writes() {
    for case in CASES {
        for value in targets(case) {
            let (f, adapter, device) = outlet_for(case.0, case.5).await;
            let before = f.cloud.calls.lock().unwrap().len();
            assert!(
                adapter
                    .inspect_control_options(&device, &device.connections[0], &operation())
                    .await
                    .unwrap()
                    .is_empty()
            );
            assert_eq!(f.cloud.calls.lock().unwrap().len(), before);
            let c = command(
                &device,
                case.1,
                case.2,
                value.clone(),
                "exact-outlet-target",
            );
            let elevated = case.1 != "indicator_light-4";
            assert_eq!(
                crate::outlet_control_option(case.1, case.2)
                    .unwrap()
                    .definition
                    .risk,
                if elevated { Risk::Elevated } else { Risk::Low }
            );
            let admitted = core(&f, adapter.clone(), &c);
            let denied = Core::new(
                f.store.clone(),
                vec![adapter],
                Arc::new(SystemClock),
                Policy::default(),
            );
            assert_eq!(
                denied
                    .execute(c.clone(), CancellationToken::new())
                    .await
                    .status,
                Status::Failed
            );
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
            let result = if elevated {
                assert!(admitted.plan(&c).is_err());
                assert_eq!(
                    admitted
                        .execute(c.clone(), CancellationToken::new())
                        .await
                        .status,
                    Status::Failed
                );
                let preview = admitted.prepare_user_approval(c.clone()).unwrap();
                assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
                admitted.execute_user_approved(preview, &operation()).await
            } else {
                admitted.execute(c.clone(), CancellationToken::new()).await
            };
            assert_eq!(result.status, Status::Accepted);
            assert_eq!(
                f.cloud.last_write.lock().unwrap().as_ref().unwrap(),
                &json!({"did":"FICTIONAL-OUTLET","siid":case.3,"piid":case.4,"value":value})
            );
            admitted.execute(c.clone(), CancellationToken::new()).await;
            assert!(admitted.prepare_user_approval(c).is_err());
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 1);
        }
    }
}

#[tokio::test]
async fn added_outlet_targets_refuse_wrong_family_and_forged_public_risk() {
    for case in CASES {
        for mutation in [
            "disabled",
            "wrong_family",
            "public_risk",
            "protected_identity",
        ] {
            if case.1 == "indicator_light-4" && mutation == "public_risk" {
                continue;
            }
            let (f, adapter, device) = outlet_for(case.0, case.5).await;
            let c = command(&device, case.1, case.2, targets(case)[0].clone(), mutation);
            let elevated = case.1 != "indicator_light-4";
            let a = if mutation == "disabled" {
                f.adapter.clone()
            } else if mutation == "wrong_family" {
                Arc::new(
                    MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
                        .with_outlet_controls()
                        .with_control_policies(
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
                            Policy::default(),
                        ),
                )
            } else {
                adapter
            };
            if mutation == "public_risk" {
                let mut d = f.core.describe(&device.id).unwrap();
                d.capabilities
                    .iter_mut()
                    .find(|cap| {
                        cap.id == c.capability && cap.component_id.as_deref() == Some(case.1)
                    })
                    .unwrap()
                    .actions
                    .get_mut(&c.action)
                    .unwrap()
                    .risk = Risk::Low;
                f.store.save_device(&d).unwrap();
            }
            let admitted = core(&f, a, &c);
            let preview = if elevated && mutation != "public_risk" {
                Some(admitted.prepare_user_approval(c.clone()).unwrap())
            } else {
                None
            };
            if mutation == "protected_identity" {
                let mut secrets = f.secrets.values.lock().unwrap();
                let bytes = secrets.values_mut().next().unwrap();
                let mut data: Value = serde_json::from_slice(bytes).unwrap();
                data["bindings"][0]["connection_id"] = json!(Uuid::new_v4().to_string());
                *bytes = serde_json::to_vec(&data).unwrap();
            }
            let result = if let Some(preview) = preview {
                admitted.execute_user_approved(preview, &operation()).await
            } else {
                admitted.execute(c, CancellationToken::new()).await
            };
            assert_eq!(result.status, Status::Failed, "{} {mutation}", case.1);
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 0);
        }
    }
}

#[tokio::test]
async fn additional_outlet_lost_rejected_and_cancelled_replies_preserve_receipts_without_replay() {
    for case in CASES {
        for mode in [1, 2, 3] {
            let (f, adapter, device) = outlet_for(case.0, case.5).await;
            let c = command(
                &device,
                case.1,
                case.2,
                targets(case)[0].clone(),
                "extended-outlet-uncertain",
            );
            let admitted = core(&f, adapter.clone(), &c);
            f.cloud.write_mode.store(mode, Ordering::SeqCst);
            let op = operation();
            let preview = if case.1 != "indicator_light-4" {
                Some(admitted.prepare_user_approval(c.clone()).unwrap())
            } else {
                None
            };
            let work = async {
                if let Some(preview) = preview {
                    admitted.execute_user_approved(preview, &op).await
                } else {
                    admitted.execute(c.clone(), op.cancellation.clone()).await
                }
            };
            tokio::pin!(work);
            if mode == 3 {
                tokio::select! {_=f.cloud.read_gate.notified()=>{},_=&mut work=>panic!("Expected pending write")}
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
            let reopened = core(&f, adapter, &c);
            assert!(reopened.prepare_user_approval(c.clone()).is_err());
            reopened.execute(c.clone(), CancellationToken::new()).await;
            assert_eq!(
                f.store.execution(&c.request_id).unwrap().unwrap().1.status,
                result.status
            );
            assert_eq!(f.cloud.count(CloudEndpoint::SetProperty), 1);
        }
    }
}
