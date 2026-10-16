use super::dishwasher::inventory;
use super::*;
use homeverb_domain::{Idempotency, Phase, Risk, Status, Transport};

const CAPABILITY: &str = "vendor.midea.dishwasher";

#[tokio::test]
async fn explicit_rediscovery_upgrades_legacy_metadata_without_losing_cached_evidence() {
    let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":"76006481"}))).await;
    let device = f.discover().await.remove(0);
    *f.wire.state.lock().unwrap() = Some(json!({"work_status":"work"}));
    let before = f.read(&device, "main", false).await.unwrap();
    let mut legacy = f.core.describe(&device.id).unwrap();
    legacy.capabilities[0].revision = 1;
    legacy.capabilities[0].properties.remove("operator");
    f.store.save_device(&legacy).unwrap();
    *f.wire.state.lock().unwrap() = Some(json!({"work_status":"work", "operator":"pause"}));
    assert!(f.read(&device, "main", false).await.is_err());
    assert_eq!(
        json!(f.store.snapshot(&device.id, "main").unwrap().unwrap()),
        json!(before)
    );
    f.discover().await;
    assert_eq!(
        f.core.describe(&device.id).unwrap().capabilities[0].revision,
        3
    );
    let after = f.read(&device, "main", false).await.unwrap();
    assert_eq!(
        after.observations["vendor.midea.dishwasher.operator"].value,
        "pause"
    );
    assert_eq!(after.device_id, device.id);
}

fn core(f: &Fixture, device: &Device, enabled: bool) -> Core {
    let mut adapter = MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock));
    if enabled {
        adapter = adapter.with_dishwasher_cycle_control();
    }
    Core::new(
        f.store.clone(),
        vec![Arc::new(adapter)],
        Arc::new(SystemClock),
        Policy {
            grants: DISHWASHER_CYCLE_ACTIONS
                .into_iter()
                .map(|action| Grant {
                    device_id: device.id.clone(),
                    component_id: "main".into(),
                    capability: CAPABILITY.into(),
                    action: action.action_name().into(),
                    transports: vec![Transport::Cloud],
                    argument_schema: json!({"type":"object"}),
                })
                .collect(),
        },
    )
}

fn command(device: &Device, action: DishwasherCycleAction) -> Command {
    let mut command = power_command(device, false);
    command.capability = CAPABILITY.into();
    command.action = action.action_name().into();
    command.arguments = json!({});
    command
}

#[tokio::test]
async fn reviewed_cycle_payloads_require_approval_and_preserve_observations_and_replay() {
    for (product, additional) in [
        ("7600020L", None),
        ("76006481", None),
        ("7600649Q", None),
        ("7600V1E7", None),
        ("760Y0026", Some(0)),
        ("760Y0026", Some(9)),
        ("760Y0026", Some(10)),
        ("760Y0026", Some(13)),
        ("760Y0026", Some(14)),
    ] {
        let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":product}))).await;
        let device = f.discover().await.remove(0);
        *f.wire.state.lock().unwrap() = Some(json!({"work_status":"work", "operator":"pause"}));
        let before = f.read(&device, "main", false).await.unwrap();
        let observed = &before.observations["vendor.midea.dishwasher.operator"];
        assert_eq!(observed.value, "pause");
        assert!(observed.stale && observed.observed_at.is_none());
        let enabled = core(&f, &device, true);
        enabled.discover(&operation()).await.unwrap();
        let descriptor = enabled.describe(&device.id).unwrap();
        let cap = descriptor.capability("main", CAPABILITY).unwrap();
        assert_eq!(
            json!(cap),
            serde_json::from_str::<Value>(if additional.is_some() {
                include_str!(
                    "../../../../examples/capability.meiju-dishwasher-cycle-additional.json"
                )
            } else {
                include_str!("../../../../examples/capability.meiju-dishwasher-cycle.json")
            })
            .unwrap()
        );
        assert!(!cap.actions.contains_key("start_cycle"));
        for action in DISHWASHER_CYCLE_ACTIONS {
            let contract = &cap.actions[action.action_name()];
            assert_eq!(contract.risk, Risk::Elevated);
            assert_eq!(contract.idempotency, Idempotency::NonIdempotent);
            let mut c = command(&device, action);
            if let Some(code) = additional {
                c.arguments = json!({"additional":code});
            }
            assert_eq!(enabled.plan(&c).unwrap_err().code, "POLICY_DENIED");
            assert_eq!(
                enabled
                    .execute(c.clone(), CancellationToken::new())
                    .await
                    .error
                    .unwrap()
                    .code,
                "POLICY_DENIED"
            );
            let reads = f.calls();
            let sends = f.wire.controls.lock().unwrap().len();
            let preview = enabled.prepare_user_approval(c.clone()).unwrap();
            let duplicate = enabled.prepare_user_approval(c.clone()).unwrap();
            let result = enabled.execute_user_approved(preview, &operation()).await;
            assert_eq!(result.status, Status::Accepted);
            assert_eq!(result.error.as_ref().unwrap().code, "VERIFICATION_PENDING");
            assert!(result.verification.is_none());
            let mut body = match action {
                DishwasherCycleAction::PauseCycle => json!({"operator":"pause"}),
                DishwasherCycleAction::ResumeCycle => json!({"operator":"start"}),
                DishwasherCycleAction::CancelCycle => json!({"work_status":"cancel"}),
            };
            if let Some(code) = additional {
                body["additional"] = json!(code);
            }
            assert_eq!(
                f.wire.controls.lock().unwrap().last().unwrap()["command"],
                json!({"control":body})
            );
            assert_eq!(
                json!(enabled.execute_user_approved(duplicate, &operation()).await),
                json!(result)
            );
            assert_eq!(f.wire.controls.lock().unwrap().len(), sends + 1);
            assert_eq!(f.calls(), reads);
            assert_eq!(
                json!(f.store.snapshot(&device.id, "main").unwrap().unwrap()),
                json!(before)
            );
            assert_eq!(
                json!(
                    enabled
                        .execution(&c.request_id, &operation())
                        .await
                        .unwrap()
                ),
                json!(result)
            );
        }
        core(&f, &device, false)
            .discover(&operation())
            .await
            .unwrap();
        assert!(
            f.core
                .describe(&device.id)
                .unwrap()
                .capability("main", CAPABILITY)
                .unwrap()
                .actions
                .is_empty()
        );
    }
}

#[tokio::test]
async fn centralized_missing_or_forged_profiles_and_host_opt_out_refuse_before_send() {
    for (profile, enabled) in [
        (json!({}), true),
        (json!({"sn8":"76006481"}), true),
        (json!({"modelNumber":1,"sn8":"TEST1234"}), true),
        (json!({"modelNumber":1,"sn8":"7600016L"}), true),
        (
            json!({"modelNumber":1,"sn8":"76006481","productModel":"Unreviewed"}),
            true,
        ),
        (json!({"modelNumber":1,"sn8":"76006481","type":"AC"}), true),
        (json!({"modelNumber":1,"sn8":"76006481"}), false),
    ] {
        let f = Fixture::new(inventory(profile)).await;
        let device = f.discover().await.remove(0);
        let enabled = core(&f, &device, enabled);
        enabled.discover(&operation()).await.unwrap();
        assert!(
            enabled
                .describe(&device.id)
                .unwrap()
                .capabilities
                .iter()
                .all(|cap| cap.actions.is_empty())
        );
        let mut forged = enabled.describe(&device.id).unwrap();
        forged.device.model = MODELS[5].0.into();
        forged.device.support_status = "experimental".into();
        forged.device.components = vec![homeverb_domain::Component {
            id: "main".into(),
            capabilities: vec![CAPABILITY.into()],
        }];
        forged.capabilities = vec![
            serde_json::from_str(include_str!(
                "../../../../examples/capability.meiju-dishwasher-cycle.json"
            ))
            .unwrap(),
        ];
        f.store.save_device(&forged).unwrap();
        for action in DISHWASHER_CYCLE_ACTIONS {
            let preview = enabled
                .prepare_user_approval(command(&device, action))
                .unwrap();
            let result = enabled.execute_user_approved(preview, &operation()).await;
            assert_eq!(result.status, Status::Failed);
            assert_eq!(result.attempts[0].phase, Phase::NotSent);
            assert_eq!(result.error.unwrap().code, "CAPABILITY_UNSUPPORTED");
        }
        assert!(f.wire.controls.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn private_cycle_contract_and_approval_survive_public_metadata_tampering() {
    for action in DISHWASHER_CYCLE_ACTIONS {
        let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":"76006481"}))).await;
        let device = f.discover().await.remove(0);
        let enabled = core(&f, &device, true);
        enabled.discover(&operation()).await.unwrap();
        let original = enabled.describe(&device.id).unwrap();
        for arguments in [
            json!({"operator":"start"}),
            json!({"confirmed":true}),
            json!({"program":"eco_wash"}),
        ] {
            let mut c = command(&device, action);
            c.arguments = arguments;
            assert!(enabled.prepare_user_approval(c.clone()).is_err());
            let mut forged = original.clone();
            forged.capabilities[0]
                .actions
                .get_mut(action.action_name())
                .unwrap()
                .parameters = json!({"type":"object"});
            f.store.save_device(&forged).unwrap();
            let preview = enabled.prepare_user_approval(c).unwrap();
            let result = enabled.execute_user_approved(preview, &operation()).await;
            assert_eq!(result.status, Status::Failed);
            assert_eq!(result.attempts[0].phase, Phase::NotSent);
            f.store.save_device(&original).unwrap();
        }
        let mut forged = original;
        forged.capabilities[0]
            .actions
            .get_mut(action.action_name())
            .unwrap()
            .risk = Risk::Low;
        f.store.save_device(&forged).unwrap();
        f.secrets.unavailable.store(true, Ordering::Relaxed);
        let result = enabled
            .execute(command(&device, action), CancellationToken::new())
            .await;
        assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
        assert_eq!(result.attempts[0].phase, Phase::NotSent);
        assert!(f.wire.controls.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn cycle_outcomes_remain_conservative_and_membership_is_rechecked_after_preview() {
    for action in DISHWASHER_CYCLE_ACTIONS {
        for (fault, status, phase) in [
            (0, Status::Accepted, Phase::Accepted),
            (1, Status::Failed, Phase::NotSent),
            (2, Status::Unknown, Phase::PossiblySent),
            (4, Status::Failed, Phase::NotSent),
            (5, Status::Failed, Phase::Rejected),
        ] {
            let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":"76006481"}))).await;
            let device = f.discover().await.remove(0);
            let enabled = core(&f, &device, true);
            enabled.discover(&operation()).await.unwrap();
            f.wire.control_fault.store(fault, Ordering::Relaxed);
            let c = command(&device, action);
            let preview = enabled.prepare_user_approval(c.clone()).unwrap();
            let result = enabled.execute_user_approved(preview, &operation()).await;
            assert_eq!(result.status, status);
            assert_eq!(result.attempts[0].phase, phase);
            assert_eq!(
                json!(
                    enabled
                        .execution(&c.request_id, &operation())
                        .await
                        .unwrap()
                ),
                json!(result)
            );
            assert!(!json!(result).to_string().contains("PRIVATE"));
            let preview = enabled
                .prepare_user_approval(command(&device, action))
                .unwrap();
            f.wire
                .inventories
                .lock()
                .unwrap()
                .get_mut("FIRST")
                .unwrap()
                .clear();
            enabled.discover(&operation()).await.unwrap();
            let sends = f.wire.controls.lock().unwrap().len();
            let result = enabled.execute_user_approved(preview, &operation()).await;
            assert_eq!(result.error.unwrap().code, "MIDEA_DEVICE_NOT_LISTED");
            assert_eq!(result.attempts[0].phase, Phase::NotSent);
            assert_eq!(f.wire.controls.lock().unwrap().len(), sends);
        }
    }
}

#[tokio::test(start_paused = true)]
async fn cancelled_or_expired_cycle_send_stays_unknown_without_retry_and_releases_lease() {
    for action in DISHWASHER_CYCLE_ACTIONS {
        for deadline in [false, true] {
            let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":"76006481"}))).await;
            let device = f.discover().await.remove(0);
            let enabled = core(&f, &device, true);
            enabled.discover(&operation()).await.unwrap();
            f.wire.control_fault.store(3, Ordering::Relaxed);
            let c = command(&device, action);
            let preview = enabled.prepare_user_approval(c.clone()).unwrap();
            let op = operation();
            let work = enabled.execute_user_approved(preview, &op);
            tokio::pin!(work);
            tokio::select! { _ = f.wire.control_started.notified() => (), result = &mut work => panic!("Expected pending: {result:?}") }
            if deadline {
                tokio::time::advance(std::time::Duration::from_secs(2)).await;
            } else {
                op.cancellation.cancel();
            }
            let result = work.await;
            assert_eq!(result.status, Status::Unknown);
            assert_eq!(result.attempts[0].phase, Phase::PossiblySent);
            assert_eq!(f.wire.controls.lock().unwrap().len(), 1);
            assert_eq!(
                json!(
                    enabled
                        .execution(&c.request_id, &operation())
                        .await
                        .unwrap()
                ),
                json!(result)
            );
            let account = f.vault.list(&operation()).await.unwrap()[0].id;
            drop(f.vault.lease(account, &operation()).await.unwrap());
        }
    }
}

#[tokio::test]
async fn additional_cycle_options_are_required_and_rechecked_after_public_schema_tampering() {
    let f = Fixture::new(inventory(json!({"modelNumber":1,"sn8":"760Y0026"}))).await;
    let device = f.discover().await.remove(0);
    let enabled = core(&f, &device, true);
    enabled.discover(&operation()).await.unwrap();
    let original = enabled.describe(&device.id).unwrap();
    for action in DISHWASHER_CYCLE_ACTIONS {
        for arguments in [
            json!({}),
            json!({"additional":null}),
            json!({"additional":true}),
            json!({"additional":1}),
            json!({"additional":9.5}),
            json!({"additional":"9"}),
            json!({"additional":9,"program":"eco_wash"}),
        ] {
            let mut c = command(&device, action);
            c.arguments = arguments;
            assert!(enabled.prepare_user_approval(c.clone()).is_err());
            let mut forged = original.clone();
            forged.capabilities[0]
                .actions
                .get_mut(action.action_name())
                .unwrap()
                .parameters = json!({"type":"object"});
            f.store.save_device(&forged).unwrap();
            let preview = enabled.prepare_user_approval(c).unwrap();
            let result = enabled.execute_user_approved(preview, &operation()).await;
            assert_eq!(result.status, Status::Failed);
            assert_eq!(result.attempts[0].phase, Phase::NotSent);
            f.store.save_device(&original).unwrap();
        }
    }
    assert!(f.wire.controls.lock().unwrap().is_empty());
}
