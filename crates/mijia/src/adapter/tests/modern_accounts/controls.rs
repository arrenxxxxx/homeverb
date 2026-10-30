use super::*;
use crate::{ModernControlRequest, ModernControlResponse, ModernControlTransport};
use homeverb_core::{Grant, WriteProgress};
use homeverb_domain::{Command, Phase, Risk, Status, WaitFor};

#[derive(Default)]
struct Writer {
    writes: Mutex<Vec<Value>>,
    mode: AtomicU8,
    started: tokio::sync::Notify,
}
#[async_trait]
impl ModernControlTransport for Writer {
    async fn send_modern_control(
        &self,
        request: &ModernControlRequest,
        progress: &WriteProgress,
        op: &Operation,
    ) -> Result<ModernControlResponse> {
        assert_eq!(
            request.url(),
            "https://core.api.mijia.tech/app/miotspec/prop/set"
        );
        assert!(request.cookie().contains("PRIVATE-MODERN-TOKEN"));
        assert!(!request.cookie().contains("PRIVATE-SERVICE-TOKEN"));
        assert!(!request.cookie().contains("passToken"));
        let fields: HashMap<_, _> = reqwest::Url::parse(&format!(
            "https://example.invalid/?{}",
            std::str::from_utf8(request.body()).unwrap()
        ))
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"])?;
        let data: Value =
            serde_json::from_slice(codec::decrypt(&key, fields["data"].as_bytes())?.expose())
                .unwrap();
        let mode = self.mode.load(Ordering::SeqCst);
        if mode == 1 {
            return Err(crate::failed());
        }
        progress.mark_sent();
        self.writes.lock().unwrap().push(data["params"][0].clone());
        self.started.notify_one();
        if mode == 2 {
            return Err(crate::failed());
        }
        if mode == 4 {
            std::future::pending::<()>().await;
        }
        let mut ack = data["params"][0].clone();
        ack.as_object_mut().unwrap().remove("value");
        ack["code"] = json!(if mode == 3 { -4001 } else { 0 });
        if mode == 6 {
            ack["did"] = json!("foreign");
        }
        if mode == 5 {
            op.cancellation.cancel();
        }
        Ok(ModernControlResponse {
            body: SecretBytes::new(
                codec::encrypt(
                    &key,
                    &serde_json::to_vec(&json!({"code":0,"result":[ack]})).unwrap(),
                )
                .as_str()
                .as_bytes()
                .to_vec(),
            ),
            encoding: ModernContentEncoding::Plain,
        })
    }
}
struct Setup {
    f: Fixture,
    p: Arc<Peer>,
    w: Arc<Writer>,
    a: Arc<MijiaAdapter>,
    device: Device,
    account: AccountRecord,
}
async fn setup(model: &str, mixed: bool, modern: bool, enabled: bool) -> Setup {
    let f = Fixture::new();
    let p = Peer::new();
    let w = Arc::new(Writer::default());
    *p.devices.lock().unwrap() = vec![
        json!({"did":"PRIVATE-CONTROL-ROOT.s7","name":"Modern appliance","model":model,"pid":0}),
    ];
    let a = adapter(&f, &p).with_modern_control_transport(w.clone());
    let a = Arc::new(if enabled {
        a.with_fan_controls()
            .with_reference_controls()
            .with_outlet_controls()
    } else {
        a
    });
    if mixed {
        *f.cloud.devices.lock().unwrap() = p.devices.lock().unwrap().clone();
        let r = a
            .connect_devices(credentials(), None, "Home", &operation())
            .await
            .unwrap();
        f.publish(&r).await;
    }
    let account = connect(&a).await.account;
    if modern {
        let r = a
            .refresh_modern_devices(account.id, &operation())
            .await
            .unwrap();
        f.publish(&r).await;
    }
    let device = f.core.list().unwrap().remove(0);
    p.reads.lock().unwrap().clear();
    f.cloud.calls.lock().unwrap().clear();
    Setup {
        f,
        p,
        w,
        a,
        device,
        account,
    }
}
fn make_command(t: &Setup, field: &crate::MappedProperty, value: Value, id: &str) -> Command {
    let mapping = ModelMapping::pinned(&t.device.model, None)
        .unwrap()
        .unwrap();
    let option = mapping.control_option(field).unwrap();
    Command {
        schema_version: "1.0".into(),
        request_id: id.into(),
        device_id: t.device.id.clone(),
        component_id: option.component_id,
        capability: option.capability,
        action: option.action,
        arguments: json!({field.property.clone():value}),
        connection: Route::Cloud,
        timeout_ms: 5000,
        wait_for: WaitFor::Accepted,
    }
}
fn grant(c: &Command) -> Grant {
    Grant {
        device_id: c.device_id.clone(),
        component_id: c.component_id.clone(),
        capability: c.capability.clone(),
        action: c.action.clone(),
        transports: vec![Transport::Cloud],
        argument_schema: json!({"const":c.arguments}),
    }
}
fn admitted(t: &Setup, c: &Command) -> Core {
    Core::new(
        t.f.store.clone(),
        vec![t.a.clone()],
        Arc::new(SystemClock),
        Policy {
            grants: vec![grant(c)],
        },
    )
}
fn values(field: &crate::MappedProperty) -> Vec<Value> {
    let d = field.write_domain.as_ref().unwrap();
    if let Some(values) = d["enum"].as_array() {
        return values.clone();
    }
    if d["type"] == "boolean" {
        return vec![json!(false), json!(true)];
    }
    let mut result = vec![d["minimum"].clone(), d["maximum"].clone()];
    if d["type"] == "integer" {
        result = result
            .into_iter()
            .map(|v| json!(v.as_f64().unwrap() as i64))
            .collect();
    }
    result.dedup();
    result
}
fn first(t: &Setup) -> crate::MappedProperty {
    ModelMapping::pinned(&t.device.model, None)
        .unwrap()
        .unwrap()
        .properties
        .into_iter()
        .find(|p| p.write_domain.is_some())
        .unwrap()
}
#[tokio::test]
async fn every_modern_exact_control_keeps_grants_wire_values_and_outlet_approval() {
    let mut fields = 0;
    for model in [
        "jdjz.fan.2072fc",
        "abhome.light.wy0a01",
        "zhimi.airpurifier.v6",
        "cuco.plug.v3",
        "xiaomi.hood.cyp10",
        "yutai.plug.jdls1",
        "opple.switch.0010d5",
        "opple.switch.0010d6",
    ] {
        let t = setup(model, false, true, true).await;
        let mapping = ModelMapping::pinned(model, None).unwrap().unwrap();
        let inspected = t
            .a
            .inspect_control_options_with_outlets(&t.device, &t.device.connections[0], &operation())
            .await
            .unwrap();
        assert_eq!(inspected.len(), mapping.control_options().len());
        let auth = t.p.auth.lock().unwrap().len();
        let writes = puts(&t.f);
        for field in mapping
            .properties
            .iter()
            .filter(|p| p.write_domain.is_some())
        {
            fields += 1;
            for (i, value) in values(field).into_iter().enumerate() {
                let c = make_command(&t, field, value.clone(), &format!("modern-{fields}-{i}"));
                let denied = Core::new(
                    t.f.store.clone(),
                    vec![t.a.clone()],
                    Arc::new(SystemClock),
                    Policy::default(),
                );
                let before = t.w.writes.lock().unwrap().len();
                let mut no_grant = c.clone();
                no_grant.request_id.push_str("-denied");
                assert_eq!(
                    denied
                        .execute(no_grant, CancellationToken::new())
                        .await
                        .status,
                    Status::Failed
                );
                let core = admitted(&t, &c);
                let elevated = mapping.control_option(field).unwrap().definition.risk != Risk::Low;
                let receipt = if elevated {
                    let mut ordinary = c.clone();
                    ordinary.request_id.push_str("-ordinary");
                    assert_eq!(
                        core.execute(ordinary, CancellationToken::new())
                            .await
                            .status,
                        Status::Failed
                    );
                    let preview = core.prepare_user_approval(c.clone()).unwrap();
                    assert_eq!(t.w.writes.lock().unwrap().len(), before);
                    core.execute_user_approved(preview, &operation()).await
                } else {
                    core.execute(c.clone(), CancellationToken::new()).await
                };
                assert_eq!(
                    receipt.status,
                    Status::Accepted,
                    "{model} {}",
                    field.property
                );
                assert_eq!(receipt.attempts[0].phase, Phase::Accepted);
                assert_eq!(
                    t.w.writes.lock().unwrap().last().unwrap(),
                    &json!({"did":"PRIVATE-CONTROL-ROOT.s7","siid":field.siid,"piid":field.piid,"value":value})
                );
                core.execute(c, CancellationToken::new()).await;
                assert_eq!(t.w.writes.lock().unwrap().len(), before + 1);
            }
        }
        assert!(t.p.reads.lock().unwrap().is_empty());
        assert!(t.f.cloud.calls.lock().unwrap().is_empty());
        assert_eq!(t.p.auth.lock().unwrap().len(), auth);
        assert_eq!(puts(&t.f), writes);
    }
    assert_eq!(fields, 40);
}
#[tokio::test]
async fn modern_controls_remain_read_only_without_opt_in_and_legacy_selection_stays_legacy() {
    let t = setup("jdjz.fan.2072fc", false, true, false).await;
    let field = first(&t);
    let c = make_command(&t, &field, values(&field)[0].clone(), "read-only");
    assert!(
        !t.a.inspect_control_options(&t.device, &t.device.connections[0], &operation())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        t.f.core
            .describe(&t.device.id)
            .unwrap()
            .capabilities
            .iter()
            .all(|c| c.actions.is_empty())
    );
    assert_eq!(
        admitted(&t, &c)
            .execute(c, CancellationToken::new())
            .await
            .status,
        Status::Failed
    );
    assert!(t.w.writes.lock().unwrap().is_empty());
    let t = setup("jdjz.fan.2072fc", true, false, true).await;
    let field = first(&t);
    let c = make_command(&t, &field, values(&field)[0].clone(), "legacy-selected");
    assert_eq!(
        admitted(&t, &c)
            .execute(c, CancellationToken::new())
            .await
            .status,
        Status::Accepted
    );
    assert!(t.w.writes.lock().unwrap().is_empty());
    assert_eq!(t.f.cloud.count(CloudEndpoint::SetProperty), 1);
}
#[tokio::test]
async fn modern_unknown_rejected_and_not_sent_receipts_never_fallback_or_replay() {
    for (mode, status, phase) in [
        (1, Status::Failed, Phase::NotSent),
        (2, Status::Unknown, Phase::PossiblySent),
        (3, Status::Failed, Phase::Rejected),
        (5, Status::Unknown, Phase::PossiblySent),
        (6, Status::Unknown, Phase::PossiblySent),
    ] {
        let t = setup("jdjz.fan.2072fc", true, true, true).await;
        let field = first(&t);
        let c = make_command(&t, &field, values(&field)[0].clone(), "modern-outcome");
        t.w.mode.store(mode, Ordering::SeqCst);
        let core = admitted(&t, &c);
        let result = core.execute(c.clone(), CancellationToken::new()).await;
        assert_eq!(result.status, status);
        assert_eq!(result.attempts[0].phase, phase);
        let before = t.w.writes.lock().unwrap().len();
        core.execute(c, CancellationToken::new()).await;
        assert_eq!(t.w.writes.lock().unwrap().len(), before);
        assert!(t.f.cloud.calls.lock().unwrap().is_empty());
        assert!(t.p.reads.lock().unwrap().is_empty());
    }
}
#[tokio::test]
async fn modern_write_cancel_deadline_drop_keep_unknown_and_release_leases() {
    for mode in 0..3 {
        let t = setup("jdjz.fan.2072fc", false, true, true).await;
        let field = first(&t);
        let mut c = make_command(&t, &field, values(&field)[0].clone(), "modern-pending");
        if mode == 1 {
            c.timeout_ms = 100;
        }
        let core = admitted(&t, &c);
        t.w.mode.store(4, Ordering::SeqCst);
        let cancellation = CancellationToken::new();
        let mut work = Box::pin(core.execute(c.clone(), cancellation.clone()));
        tokio::select! {_=t.w.started.notified()=>{},_=&mut work=>panic!("Expected pending write")}
        assert!(t.f.vault.lease(t.account.id, &operation()).await.is_err());
        assert!(t.f.store.lock_device(&t.device.id).is_err());
        if mode == 0 {
            cancellation.cancel();
            assert_eq!(work.await.status, Status::Unknown);
        } else if mode == 1 {
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
        assert_eq!(
            core.execute(c, CancellationToken::new()).await.status,
            Status::Unknown
        );
        assert_eq!(t.w.writes.lock().unwrap().len(), 1);
    }
}
#[tokio::test]
async fn modern_outlet_public_risk_and_cross_family_policy_cannot_bypass_approval() {
    for wrong_family in [false, true] {
        let t = setup("cuco.plug.v3", false, true, true).await;
        let field = first(&t);
        let c = make_command(&t, &field, json!(true), "outlet-protected");
        let core = if wrong_family {
            let a = adapter(&t.f, &t.p)
                .with_modern_control_transport(t.w.clone())
                .with_outlet_controls()
                .with_control_policies(
                    Policy {
                        grants: vec![grant(&c)],
                    },
                    Policy::default(),
                );
            Core::new(
                t.f.store.clone(),
                vec![Arc::new(a)],
                Arc::new(SystemClock),
                Policy {
                    grants: vec![grant(&c)],
                },
            )
        } else {
            let mut d = t.f.core.describe(&t.device.id).unwrap();
            d.capabilities
                .iter_mut()
                .find(|cap| {
                    cap.id == c.capability && cap.component_id.as_deref() == Some(&c.component_id)
                })
                .unwrap()
                .actions
                .get_mut(&c.action)
                .unwrap()
                .risk = Risk::Low;
            t.f.store.save_device(&d).unwrap();
            admitted(&t, &c)
        };
        let result = if wrong_family {
            let preview = core.prepare_user_approval(c).unwrap();
            core.execute_user_approved(preview, &operation()).await
        } else {
            core.execute(c, CancellationToken::new()).await
        };
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.attempts[0].phase, Phase::NotSent);
        assert!(t.w.writes.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn legacy_unknown_receipt_is_not_replayed_after_explicit_modern_adoption() {
    let t = setup("jdjz.fan.2072fc", true, false, true).await;
    let field = first(&t);
    let c = make_command(
        &t,
        &field,
        values(&field)[0].clone(),
        "cross-protocol-receipt",
    );
    let core = admitted(&t, &c);
    t.f.cloud.write_mode.store(1, Ordering::SeqCst);
    assert_eq!(
        core.execute(c.clone(), CancellationToken::new())
            .await
            .status,
        Status::Unknown
    );
    let report =
        t.a.refresh_modern_devices(t.account.id, &operation())
            .await
            .unwrap();
    t.f.publish(&report).await;
    assert_eq!(
        core.execute(c, CancellationToken::new()).await.status,
        Status::Unknown
    );
    assert!(t.w.writes.lock().unwrap().is_empty());
    assert_eq!(t.f.cloud.count(CloudEndpoint::SetProperty), 1);
}

#[tokio::test]
async fn additional_modern_outlet_targets_keep_family_and_protected_risk_checks() {
    for model in [
        "yutai.plug.jdls1",
        "opple.switch.0010d5",
        "opple.switch.0010d6",
    ] {
        let mapping = ModelMapping::pinned(model, None).unwrap().unwrap();
        for field in mapping
            .properties
            .iter()
            .filter(|p| p.write_domain.is_some())
        {
            for mutation in ["disabled", "family", "risk"] {
                let elevated =
                    mapping.control_option(field).unwrap().definition.risk == Risk::Elevated;
                if mutation == "risk" && !elevated {
                    continue;
                }
                let t = setup(model, false, true, mutation != "disabled").await;
                let c = make_command(
                    &t,
                    field,
                    values(field)[0].clone(),
                    "modern-extended-denial",
                );
                let core = if mutation == "family" {
                    let a = adapter(&t.f, &t.p)
                        .with_modern_control_transport(t.w.clone())
                        .with_outlet_controls()
                        .with_control_policies(
                            Policy {
                                grants: vec![grant(&c)],
                            },
                            Policy::default(),
                        );
                    Core::new(
                        t.f.store.clone(),
                        vec![Arc::new(a)],
                        Arc::new(SystemClock),
                        Policy {
                            grants: vec![grant(&c)],
                        },
                    )
                } else {
                    if mutation == "risk" {
                        let mut d = t.f.core.describe(&t.device.id).unwrap();
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
                            .risk = Risk::Low;
                        t.f.store.save_device(&d).unwrap();
                    }
                    admitted(&t, &c)
                };
                if elevated && mutation == "family" {
                    let preview = core.prepare_user_approval(c).unwrap();
                    assert_eq!(
                        core.execute_user_approved(preview, &operation())
                            .await
                            .status,
                        Status::Failed
                    );
                } else {
                    assert_eq!(
                        core.execute(c, CancellationToken::new()).await.status,
                        Status::Failed
                    );
                }
                assert!(t.w.writes.lock().unwrap().is_empty());
                assert!(t.f.cloud.calls.lock().unwrap().is_empty());
            }
        }
    }
}

#[tokio::test]
async fn added_modern_outlet_uncertainty_never_replays_or_crosses_protocols() {
    for model in [
        "yutai.plug.jdls1",
        "opple.switch.0010d5",
        "opple.switch.0010d6",
    ] {
        let mapping = ModelMapping::pinned(model, None).unwrap().unwrap();
        for field in mapping
            .properties
            .iter()
            .filter(|p| p.write_domain.is_some())
        {
            for mode in [2, 4, 5, 6] {
                let t = setup(model, true, true, true).await;
                let c = make_command(
                    &t,
                    field,
                    values(field)[0].clone(),
                    "modern-extended-unknown",
                );
                let core = admitted(&t, &c);
                let elevated =
                    mapping.control_option(field).unwrap().definition.risk == Risk::Elevated;
                let preview = if elevated {
                    Some(core.prepare_user_approval(c.clone()).unwrap())
                } else {
                    None
                };
                t.w.mode.store(mode, Ordering::SeqCst);
                let op = operation();
                let work = async {
                    if let Some(preview) = preview {
                        core.execute_user_approved(preview, &op).await
                    } else {
                        core.execute(c.clone(), op.cancellation.clone()).await
                    }
                };
                tokio::pin!(work);
                if mode == 4 {
                    tokio::select! {_=t.w.started.notified()=>{},_=&mut work=>panic!("Expected pending outlet write")}
                    op.cancellation.cancel();
                }
                assert_eq!(work.await.status, Status::Unknown);
                let fresh = admitted(&t, &c);
                assert!(fresh.prepare_user_approval(c.clone()).is_err());
                fresh.execute(c.clone(), CancellationToken::new()).await;
                assert_eq!(
                    t.f.store
                        .execution(&c.request_id)
                        .unwrap()
                        .unwrap()
                        .1
                        .status,
                    Status::Unknown
                );
                assert_eq!(t.w.writes.lock().unwrap().len(), 1);
                assert!(t.f.cloud.calls.lock().unwrap().is_empty());
                assert!(t.p.reads.lock().unwrap().is_empty());
                assert!(t.f.vault.lease(t.account.id, &operation()).await.is_ok());
            }
        }
    }
}
