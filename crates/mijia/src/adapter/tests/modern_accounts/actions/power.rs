//! Elevated toggles need exact grants and trusted approval on both protocols.
use super::*;
use homeverb_domain::Risk;

const MODELS: [(&str, usize); 4] = [
    ("cuco.plug.v3", 1),
    ("yutai.plug.jdls1", 1),
    ("opple.switch.0010d5", 1),
    ("opple.switch.0010d6", 2),
];

#[tokio::test]
async fn every_power_toggle_requires_confirmation_and_sends_only_its_exact_service() {
    for modern in [false, true] {
        for (model, count) in MODELS {
            let t = setup(model, modern).await;
            assert_eq!(t.options.len(), count);
            let example: Value = serde_json::from_str(include_str!(
                "../../../../../../../examples/capability.mijia-power-action.json"
            ))
            .unwrap();
            assert_eq!(
                serde_json::to_value(&t.options[0].definition).unwrap(),
                example["actions"]["toggle"]
            );
            let commands: Vec<_> = t
                .options
                .iter()
                .enumerate()
                .map(|(i, o)| command(&t, o, &format!("approved-toggle-{i}")))
                .collect();
            let a = enabled(&t, &commands).await;
            let core = core(&t, a, &commands);
            let before = bytes(&t.f, &t.account);
            let reads = t.p.reads.lock().unwrap().len();
            let legacy_reads = t.f.cloud.calls.lock().unwrap().len();
            for (i, c) in commands.iter().enumerate() {
                assert_eq!(t.options[i].definition.risk, Risk::Elevated);
                assert_eq!(
                    t.options[i].definition.idempotency,
                    Idempotency::NonIdempotent
                );
                assert!(core.plan(c).is_err());
                assert_eq!(
                    core.execute(c.clone(), CancellationToken::new())
                        .await
                        .status,
                    Status::Failed
                );
                assert_eq!(t.w.count(), i);
                let preview = core.prepare_user_approval(c.clone()).unwrap();
                let r = core.execute_user_approved(preview, &operation()).await;
                assert_eq!(r.status, Status::Accepted, "{:?}", r.error);
                assert!(r.verification.is_none());
                assert_eq!(t.w.count(), i + 1);
                let writes = t.w.writes.lock().unwrap();
                let siid: u32 = c
                    .component_id
                    .strip_prefix("switch-")
                    .unwrap()
                    .parse()
                    .unwrap();
                assert_eq!(writes[i].0, modern);
                assert_eq!(writes[i].1, json!({"did":DID,"siid":siid,"aiid":1,"in":[]}));
                drop(writes);
                assert!(core.prepare_user_approval(c.clone()).is_err());
            }
            assert_eq!(t.w.count(), count);
            assert_eq!(bytes(&t.f, &t.account), before);
            assert_eq!(t.p.reads.lock().unwrap().len(), reads);
            assert_eq!(t.f.cloud.calls.lock().unwrap().len(), legacy_reads);
        }
    }
}

#[tokio::test]
async fn forged_public_low_risk_never_bypasses_protected_power_toggle_approval() {
    for modern in [false, true] {
        for (model, _) in MODELS {
            let t = setup(model, modern).await;
            let commands: Vec<_> = t
                .options
                .iter()
                .enumerate()
                .map(|(i, o)| command(&t, o, &format!("forged-{i}")))
                .collect();
            let a = enabled(&t, &commands).await;
            let core = core(&t, a, &commands);
            let mut descriptor = core.describe(&t.device.id).unwrap();
            for cap in &mut descriptor.capabilities {
                for action in cap.actions.values_mut() {
                    action.risk = Risk::Low;
                }
            }
            t.f.store.save_device(&descriptor).unwrap();
            for c in commands {
                assert!(core.plan(&c).is_ok());
                let r = core.execute(c, CancellationToken::new()).await;
                assert_eq!(r.status, Status::Failed);
                assert_eq!(r.error.unwrap().code, "POLICY_DENIED");
                assert!(r.attempts.iter().all(|a| a.phase == Phase::NotSent));
            }
            assert_eq!(t.w.count(), 0);
        }
    }
}

#[tokio::test]
async fn power_toggle_approval_cannot_transfer_property_or_foreign_action_authority() {
    for modern in [false, true] {
        let t = setup("opple.switch.0010d6", modern).await;
        let c = command(&t, &t.options[0], "power-scope");
        let good = enabled(&t, std::slice::from_ref(&c)).await;
        for mode in 0..5 {
            let mut c = c.clone();
            c.request_id = format!("power-scope-{mode}");
            let mut p = policy(std::slice::from_ref(&c));
            let mut a = base(&t)
                .with_outlet_controls()
                .with_fan_controls()
                .with_reference_controls();
            match mode {
                0 => a = a.with_control_policies(p.clone(), p),
                1 => a = a.with_action_controls(Policy::default()),
                2 => {
                    p.grants[0].component_id = t.options[1].component_id.clone();
                    a = a.with_action_controls(p);
                }
                3 => {
                    p.grants[0].action = "set".into();
                    a = a.with_action_controls(p);
                }
                _ => {
                    p.grants[0].device_id = Uuid::new_v4().to_string();
                    a = a.with_action_controls(p);
                }
            }
            let core = core(&t, Arc::new(a), std::slice::from_ref(&c));
            let preview = core.prepare_user_approval(c).unwrap();
            let r = core.execute_user_approved(preview, &operation()).await;
            assert_eq!(r.status, Status::Failed);
            assert!(r.attempts.iter().all(|a| a.phase == Phase::NotSent));
        }
        let denied = core(&t, good, &[]);
        assert!(denied.prepare_user_approval(c).is_err());
        assert_eq!(t.w.count(), 0);
    }
}

#[tokio::test]
async fn power_toggle_lifecycle_and_reopen_preserve_unknown_without_another_approval() {
    for modern in [false, true] {
        for mode in [0, 2, 3, 4, 5, 6, 7, 8] {
            let t = setup("cuco.plug.v3", modern).await;
            let c = command(&t, &t.options[0], "power-durable");
            let a = enabled(&t, std::slice::from_ref(&c)).await;
            let admitted = core(&t, a, std::slice::from_ref(&c));
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
                assert_eq!(t.w.count(), 0);
                continue;
            }
            t.w.mode.store(mode, Ordering::SeqCst);
            let op = operation();
            let mut work = Box::pin(admitted.execute_user_approved(preview, &op));
            if mode == 3 {
                tokio::select! {_=t.w.started.notified()=>{},_=&mut work=>panic!("Expected pending toggle")}
                op.cancellation.cancel();
            }
            let r = work.await;
            assert_eq!(
                r.status,
                if mode == 4 {
                    Status::Failed
                } else {
                    Status::Unknown
                }
            );
            assert_eq!(t.w.count(), 1);
            let store = Arc::new(SqliteStore::open(&t.f.directory.0).unwrap());
            let fresh = Core::new(
                store.clone(),
                vec![Arc::new(base(&t))],
                Arc::new(SystemClock),
                policy(std::slice::from_ref(&c)),
            );
            assert_eq!(
                store.execution(&c.request_id).unwrap().unwrap().1.status,
                r.status
            );
            assert!(fresh.prepare_user_approval(c.clone()).is_err());
            assert_eq!(
                fresh
                    .execute(c.clone(), CancellationToken::new())
                    .await
                    .status,
                Status::Failed
            );
            assert_eq!(
                store.execution(&c.request_id).unwrap().unwrap().1.status,
                r.status
            );
            assert_eq!(store.clear_execution_history().unwrap(), 1);
            assert!(fresh.prepare_user_approval(c).is_err());
            assert_eq!(t.w.count(), 1);
        }
    }
}

#[tokio::test]
async fn dropped_or_expired_approved_power_toggle_releases_locks_and_never_replays() {
    for modern in [false, true] {
        for expire in [false, true] {
            let t = setup("yutai.plug.jdls1", modern).await;
            let c = command(&t, &t.options[0], "stopped-approved-toggle");
            let a = enabled(&t, std::slice::from_ref(&c)).await;
            let admitted = core(&t, a, std::slice::from_ref(&c));
            let preview = admitted.prepare_user_approval(c.clone()).unwrap();
            t.w.mode.store(3, Ordering::SeqCst);
            let op =
                Operation::new(if expire { 100 } else { 5000 }, CancellationToken::new()).unwrap();
            let mut work = Box::pin(admitted.execute_user_approved(preview, &op));
            tokio::select! {_=t.w.started.notified()=>{},_=&mut work=>panic!("Expected pending approved toggle")}
            assert!(t.f.vault.lease(t.account.id, &operation()).await.is_err());
            assert!(t.f.store.lock_device(&t.device.id).is_err());
            if expire {
                assert_eq!(work.await.status, Status::Unknown);
            } else {
                drop(work);
            }
            assert!(t.f.vault.lease(t.account.id, &operation()).await.is_ok());
            assert!(t.f.store.lock_device(&t.device.id).is_ok());
            assert_eq!(
                t.f.store
                    .execution(&c.request_id)
                    .unwrap()
                    .unwrap()
                    .1
                    .status,
                Status::Unknown
            );
            assert!(admitted.prepare_user_approval(c).is_err());
            assert_eq!(t.w.count(), 1);
        }
    }
}

#[tokio::test]
async fn trusted_power_approval_cannot_widen_protected_empty_arguments() {
    for modern in [false, true] {
        for (model, _) in MODELS {
            for args in [
                json!({"state":true}),
                json!({"in":[]}),
                json!({"confirmed":true}),
                json!({"siid":2,"aiid":1}),
            ] {
                let t = setup(model, modern).await;
                let mut c = command(&t, &t.options[0], "forged-approved-arguments");
                c.arguments = args;
                let a = enabled(&t, std::slice::from_ref(&c)).await;
                let core = core(&t, a, std::slice::from_ref(&c));
                let mut d = core.describe(&t.device.id).unwrap();
                d.capabilities
                    .iter_mut()
                    .find(|cap| {
                        cap.id == c.capability
                            && cap.component_id.as_deref() == Some(&c.component_id)
                    })
                    .unwrap()
                    .actions
                    .get_mut("toggle")
                    .unwrap()
                    .parameters = json!({"type":"object"});
                t.f.store.save_device(&d).unwrap();
                let preview = core.prepare_user_approval(c).unwrap();
                let r = core.execute_user_approved(preview, &operation()).await;
                assert_eq!(r.status, Status::Failed);
                assert!(r.attempts.iter().all(|a| a.phase == Phase::NotSent));
                assert_eq!(t.w.count(), 0);
            }
        }
    }
}

#[tokio::test]
async fn trusted_power_approval_rechecks_protected_membership_model_and_root() {
    for modern in [false, true] {
        for (model, _) in MODELS {
            for mode in 0..4 {
                let t = setup(model, modern).await;
                let c = command(&t, &t.options[0], "changed-approved-root");
                let a = enabled(&t, std::slice::from_ref(&c)).await;
                let core = core(&t, a, std::slice::from_ref(&c));
                let preview = core.prepare_user_approval(c).unwrap();
                let mut saved = payload(&t.f, &t.account);
                match mode {
                    0 => saved["bindings"][0]["listed"] = json!(false),
                    1 => saved["bindings"][0]["model"] = json!("foreign.switch"),
                    2 => saved["bindings"][0]["reported_spec"] = json!("foreign-specification"),
                    _ => {
                        if modern {
                            saved["bindings"][0]["modern"]["parent"] =
                                json!("PRIVATE-UNADMITTED-PARENT");
                        } else {
                            saved["session"] = Value::Null;
                        }
                    }
                }
                t.f.secrets.values.lock().unwrap().insert(
                    t.account.credential_ref.to_string(),
                    serde_json::to_vec(&saved).unwrap(),
                );
                let r = core.execute_user_approved(preview, &operation()).await;
                assert_eq!(r.status, Status::Failed, "{model}/{modern}/{mode}");
                assert!(r.attempts.iter().all(|a| a.phase == Phase::NotSent));
                assert_eq!(t.w.count(), 0);
            }
        }
    }
}
