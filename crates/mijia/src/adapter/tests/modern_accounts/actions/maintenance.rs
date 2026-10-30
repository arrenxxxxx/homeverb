//! Exact maintenance/stop actions retain trusted consent and persistent uncertainty.
use super::*;
use homeverb_domain::Risk;

type Case = (&'static str, &'static str, &'static str, u32, u32);
const CASES: [Case; 4] = [
    (
        "xiaomi.camera.083ac1",
        "camera_control-2",
        "ptz_calibrate",
        2,
        1,
    ),
    (
        "xiaomi.camera.083ac1",
        "camera_control-2",
        "restart_device",
        2,
        2,
    ),
    (
        "xiaomi.camera.083ac1",
        "memory_card_management-4",
        "pop_up",
        4,
        2,
    ),
    (
        "xiaomi.hood.cyp10",
        "hood_smart_dry_wash-10",
        "stop_dry_cleaning",
        10,
        1,
    ),
];
async fn selected(case: Case, modern: bool) -> Setup {
    let mut t = setup(case.0, modern).await;
    t.options
        .retain(|o| o.component_id == case.1 && o.action == case.2);
    assert_eq!(t.options.len(), 1, "{:?}", case);
    t
}

#[tokio::test]
async fn all_maintenance_actions_require_approval_and_send_only_the_exact_empty_action() {
    for modern in [false, true] {
        for case in CASES {
            let t = selected(case, modern).await;
            let o = &t.options[0];
            assert_eq!(o.definition.risk, Risk::Elevated);
            assert_eq!(o.definition.idempotency, Idempotency::NonIdempotent);
            assert_eq!(o.definition.verification, VerificationMethod::Unavailable);
            let mut c = command(&t, o, "maintenance-approved");
            c.wait_for = WaitFor::Verified;
            let a = enabled(&t, std::slice::from_ref(&c)).await;
            let core = core(&t, a, std::slice::from_ref(&c));
            let before = bytes(&t.f, &t.account);
            let reads = t.p.reads.lock().unwrap().len();
            let calls = t.f.cloud.calls.lock().unwrap().len();
            assert!(core.plan(&c).is_err());
            assert_eq!(
                core.execute(c.clone(), CancellationToken::new())
                    .await
                    .status,
                Status::Failed
            );
            let preview = core.prepare_user_approval(c.clone()).unwrap();
            assert_eq!(t.w.count(), 0);
            let r = core.execute_user_approved(preview, &operation()).await;
            assert_eq!(r.status, Status::Accepted, "{:?}", r.error);
            assert!(r.verification.is_none());
            assert_eq!(t.w.count(), 1);
            assert_eq!(
                t.w.writes.lock().unwrap()[0],
                (
                    modern,
                    json!({"did":DID,"siid":case.3,"aiid":case.4,"in":[]})
                )
            );
            assert!(core.prepare_user_approval(c).is_err());
            assert_eq!(bytes(&t.f, &t.account), before);
            assert_eq!(t.p.reads.lock().unwrap().len(), reads);
            assert_eq!(t.f.cloud.calls.lock().unwrap().len(), calls);
        }
    }
}

#[tokio::test]
async fn maintenance_grants_cannot_transfer_from_properties_or_other_actions() {
    for modern in [false, true] {
        for case in CASES {
            let t = selected(case, modern).await;
            let c = command(&t, &t.options[0], "maintenance-scope");
            let good = enabled(&t, std::slice::from_ref(&c)).await;
            for mode in 0..5 {
                let mut c = c.clone();
                c.request_id = format!("maintenance-scope-{mode}");
                let mut p = policy(std::slice::from_ref(&c));
                let mut a = base(&t)
                    .with_reference_controls()
                    .with_fan_controls()
                    .with_outlet_controls();
                match mode {
                    0 => a = a.with_control_policies(p.clone(), p),
                    1 => a = a.with_action_controls(Policy::default()),
                    2 => {
                        p.grants[0].component_id = "switch-2".into();
                        a = a.with_action_controls(p);
                    }
                    3 => {
                        p.grants[0].action = "format".into();
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
            assert!(core(&t, good, &[]).prepare_user_approval(c).is_err());
            assert_eq!(t.w.count(), 0);
        }
    }
}

#[tokio::test]
async fn protected_maintenance_risk_and_empty_arguments_survive_forged_public_metadata() {
    for modern in [false, true] {
        for case in CASES {
            let t = selected(case, modern).await;
            let c = command(&t, &t.options[0], "forged-maintenance");
            let a = enabled(&t, std::slice::from_ref(&c)).await;
            let core = core(&t, a, std::slice::from_ref(&c));
            let original = core.describe(&t.device.id).unwrap();
            let mut d = original.clone();
            let action = d
                .capabilities
                .iter_mut()
                .find(|cap| {
                    cap.id == c.capability && cap.component_id.as_deref() == Some(&c.component_id)
                })
                .unwrap()
                .actions
                .get_mut(&c.action)
                .unwrap();
            action.risk = Risk::Low;
            t.f.store.save_device(&d).unwrap();
            assert!(core.plan(&c).is_ok());
            let r = core.execute(c.clone(), CancellationToken::new()).await;
            assert_eq!(r.status, Status::Failed);
            assert_eq!(r.error.unwrap().code, "POLICY_DENIED");
            t.f.store.save_device(&original).unwrap();
            for arguments in [
                json!({"in":[]}),
                json!({"confirmed":true}),
                json!({"format":true}),
                json!({"aiid":1}),
            ] {
                let mut c = c.clone();
                c.arguments = arguments;
                c.request_id = unique_request();
                let a = enabled(&t, std::slice::from_ref(&c)).await;
                let admitted = super::core(&t, a, std::slice::from_ref(&c));
                let mut d = admitted.describe(&t.device.id).unwrap();
                d.capabilities
                    .iter_mut()
                    .find(|cap| {
                        cap.id == c.capability
                            && cap.component_id.as_deref() == Some(&c.component_id)
                    })
                    .unwrap()
                    .actions
                    .get_mut(&c.action)
                    .unwrap()
                    .parameters = json!({"type":"object"});
                t.f.store.save_device(&d).unwrap();
                let preview = admitted.prepare_user_approval(c).unwrap();
                let r = admitted.execute_user_approved(preview, &operation()).await;
                assert_eq!(r.status, Status::Failed);
                assert!(r.attempts.iter().all(|a| a.phase == Phase::NotSent));
            }
            assert_eq!(t.w.count(), 0);
        }
    }
}
fn unique_request() -> String {
    Uuid::new_v4().to_string()
}

#[tokio::test]
async fn maintenance_uncertainty_and_reopen_preserve_receipts_without_replay() {
    for modern in [false, true] {
        for case in CASES {
            for mode in [0, 2, 3, 4, 5, 6, 7, 8] {
                let t = selected(case, modern).await;
                let c = command(&t, &t.options[0], "maintenance-durable");
                let a = enabled(&t, std::slice::from_ref(&c)).await;
                let core = core(&t, a, std::slice::from_ref(&c));
                let preview = core.prepare_user_approval(c.clone()).unwrap();
                if mode == 0 {
                    core.invalidate_user_approvals();
                    assert_eq!(
                        core.execute_user_approved(preview, &operation())
                            .await
                            .status,
                        Status::Failed
                    );
                    assert_eq!(t.w.count(), 0);
                    continue;
                }
                t.w.mode.store(mode, Ordering::SeqCst);
                let op = operation();
                let mut work = Box::pin(core.execute_user_approved(preview, &op));
                if mode == 3 {
                    tokio::select! {_=t.w.started.notified()=>{},_=&mut work=>panic!("Expected pending maintenance")}
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
                assert!(t.f.vault.lease(t.account.id, &operation()).await.is_ok());
                assert!(t.f.store.lock_device(&t.device.id).is_ok());
            }
        }
    }
}

#[tokio::test]
async fn approved_maintenance_revalidates_protected_membership_and_root() {
    for modern in [false, true] {
        for case in CASES {
            for mode in 0..4 {
                let t = selected(case, modern).await;
                let c = command(&t, &t.options[0], "maintenance-changed-root");
                let a = enabled(&t, std::slice::from_ref(&c)).await;
                let core = core(&t, a, std::slice::from_ref(&c));
                let preview = core.prepare_user_approval(c).unwrap();
                let mut saved = payload(&t.f, &t.account);
                match mode {
                    0 => saved["bindings"][0]["listed"] = json!(false),
                    1 => saved["bindings"][0]["model"] = json!("foreign.camera"),
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
                assert_eq!(r.status, Status::Failed);
                assert!(r.attempts.iter().all(|a| a.phase == Phase::NotSent));
                assert_eq!(t.w.count(), 0);
            }
        }
    }
}

#[tokio::test]
async fn dropped_or_expired_maintenance_releases_locks_with_unknown_receipt() {
    for modern in [false, true] {
        for case in CASES {
            for expire in [false, true] {
                let t = selected(case, modern).await;
                let c = command(&t, &t.options[0], "maintenance-stopped");
                let a = enabled(&t, std::slice::from_ref(&c)).await;
                let core = core(&t, a, std::slice::from_ref(&c));
                let preview = core.prepare_user_approval(c.clone()).unwrap();
                t.w.mode.store(3, Ordering::SeqCst);
                let op = Operation::new(if expire { 100 } else { 5000 }, CancellationToken::new())
                    .unwrap();
                let mut work = Box::pin(core.execute_user_approved(preview, &op));
                tokio::select! {_=t.w.started.notified()=>{},_=&mut work=>panic!("Expected pending maintenance")}
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
                assert!(core.prepare_user_approval(c).is_err());
                assert_eq!(t.w.count(), 1);
            }
        }
    }
}
