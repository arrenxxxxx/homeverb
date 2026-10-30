//! Exact action Core journeys with real SQLite and fictional protected accounts.
use super::*;
use crate::{ControlOption, ModernControlRequest, ModernControlResponse, ModernControlTransport};
use homeverb_core::{Grant, WriteProgress};
use homeverb_domain::{Command, Idempotency, Phase, Status, VerificationMethod, WaitFor};

const DID: &str = "PRIVATE-ACTION-ROOT.s7";
struct Writer {
    cloud: Arc<Cloud>,
    writes: Mutex<Vec<(bool, Value)>>,
    mode: AtomicU8,
    started: tokio::sync::Notify,
}
impl Writer {
    fn new(cloud: Arc<Cloud>) -> Arc<Self> {
        Arc::new(Self {
            cloud,
            writes: Mutex::new(vec![]),
            mode: AtomicU8::new(0),
            started: tokio::sync::Notify::new(),
        })
    }
    fn count(&self) -> usize {
        self.writes.lock().unwrap().len()
    }
    async fn reply(
        &self,
        body: &[u8],
        modern: bool,
        p: &WriteProgress,
        op: &Operation,
    ) -> Result<SecretBytes> {
        let fields: HashMap<_, _> = reqwest::Url::parse(&format!(
            "https://example.invalid/?{}",
            std::str::from_utf8(body).unwrap()
        ))
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
        let key = codec::signed_nonce(&fields["ssecurity"], &fields["_nonce"])?;
        let data: Value =
            serde_json::from_slice(codec::decrypt(&key, fields["data"].as_bytes())?.expose())
                .unwrap();
        let params = data["params"].as_object().unwrap();
        assert_eq!(params.len(), 4);
        assert_eq!(params["did"], DID);
        assert_eq!(params["in"], json!([]));
        let mode = self.mode.load(Ordering::SeqCst);
        if mode == 1 {
            return Err(crate::failed());
        }
        p.mark_sent();
        self.writes
            .lock()
            .unwrap()
            .push((modern, data["params"].clone()));
        self.started.notify_one();
        if mode == 2 {
            return Err(crate::failed());
        }
        if mode == 3 {
            std::future::pending::<()>().await;
        }
        let mut ack = json!({"did":DID,"siid":params["siid"],"aiid":params["aiid"],"code":match mode {4=>-4006,5=>1,6=>-704220015,_=>0},"out":[]});
        if mode == 7 {
            ack["did"] = json!("foreign");
        }
        if mode == 8 {
            op.cancellation.cancel();
        }
        Ok(SecretBytes::new(
            codec::encrypt(
                &key,
                &serde_json::to_vec(&json!({"code":0,"result":ack})).unwrap(),
            )
            .as_str()
            .as_bytes()
            .to_vec(),
        ))
    }
}
#[async_trait]
impl CloudTransport for Writer {
    async fn send(&self, r: &CloudRequest, op: &Operation) -> Result<CloudResponse> {
        self.cloud.send(r, op).await
    }
    async fn send_control(
        &self,
        r: &CloudRequest,
        p: &WriteProgress,
        op: &Operation,
    ) -> Result<CloudResponse> {
        assert_eq!(r.endpoint(), CloudEndpoint::Action);
        assert!(r.cookie().contains("PRIVATE-SERVICE-TOKEN"));
        assert!(!r.cookie().contains("PRIVATE-MODERN-TOKEN"));
        Ok(CloudResponse {
            body: self.reply(r.body(), false, p, op).await?,
            service_token: None,
        })
    }
}
#[async_trait]
impl ModernControlTransport for Writer {
    async fn send_modern_control(
        &self,
        r: &ModernControlRequest,
        p: &WriteProgress,
        op: &Operation,
    ) -> Result<ModernControlResponse> {
        assert_eq!(r.url(), "https://core.api.mijia.tech/app/miotspec/action");
        assert!(r.cookie().contains("PRIVATE-MODERN-TOKEN"));
        assert!(!r.cookie().contains("PRIVATE-SERVICE-TOKEN"));
        Ok(ModernControlResponse {
            body: self.reply(r.body(), true, p, op).await?,
            encoding: ModernContentEncoding::Plain,
        })
    }
}
struct Setup {
    f: Fixture,
    p: Arc<Peer>,
    w: Arc<Writer>,
    account: AccountRecord,
    device: Device,
    modern: bool,
    options: Vec<ControlOption>,
}
fn base(t: &Setup) -> MijiaAdapter {
    MijiaAdapter::new(t.f.vault.clone(), t.w.clone(), Arc::new(SystemClock))
        .with_modern_protocol(t.p.clone(), t.p.clone())
        .with_modern_control_transport(t.w.clone())
}
async fn setup(model: &str, modern: bool) -> Setup {
    let f = Fixture::new();
    let p = Peer::new();
    let w = Writer::new(f.cloud.clone());
    let list = vec![json!({"did":DID,"name":"Fictional action appliance","model":model,"pid":0})];
    *f.cloud.devices.lock().unwrap() = list.clone();
    *p.devices.lock().unwrap() = list;
    let a = MijiaAdapter::new(f.vault.clone(), w.clone(), Arc::new(SystemClock))
        .with_modern_protocol(p.clone(), p.clone())
        .with_modern_control_transport(w.clone());
    let report = a
        .connect_devices(credentials(), None, "Home", &operation())
        .await
        .unwrap();
    f.publish(&report).await;
    let account = if modern {
        let r = connect(&a).await;
        let devices = a
            .refresh_modern_devices(r.account.id, &operation())
            .await
            .unwrap();
        f.publish(&devices).await;
        r.account
    } else {
        report.account
    };
    let device = f.core.list().unwrap().remove(0);
    let before = bytes(&f, &account);
    let reads = p.reads.lock().unwrap().len();
    let calls = f.cloud.calls.lock().unwrap().len();
    let writes = puts(&f);
    let options = a
        .inspect_action_options(&device, &device.connections[0], &operation())
        .await
        .unwrap();
    assert_eq!(before, bytes(&f, &account));
    assert_eq!(writes, puts(&f));
    assert_eq!(reads, p.reads.lock().unwrap().len());
    assert_eq!(calls, f.cloud.calls.lock().unwrap().len());
    assert_eq!(w.count(), 0);
    assert!(
        f.core
            .describe(&device.id)
            .unwrap()
            .capabilities
            .iter()
            .all(|c| c.actions.is_empty())
    );
    Setup {
        f,
        p,
        w,
        account,
        device,
        modern,
        options,
    }
}
fn command(t: &Setup, option: &ControlOption, id: &str) -> Command {
    Command {
        schema_version: "1.0".into(),
        request_id: id.into(),
        device_id: t.device.id.clone(),
        component_id: option.component_id.clone(),
        capability: option.capability.clone(),
        action: option.action.clone(),
        arguments: json!({}),
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
fn policy(commands: &[Command]) -> Policy {
    Policy {
        grants: commands.iter().map(grant).collect(),
    }
}
async fn publish(t: &Setup, a: &MijiaAdapter) {
    let report = if t.modern {
        a.refresh_modern_devices(t.account.id, &operation())
            .await
            .unwrap()
    } else {
        a.refresh_devices(t.account.id, &operation()).await.unwrap()
    };
    t.f.publish(&report).await;
}
async fn enabled(t: &Setup, commands: &[Command]) -> Arc<MijiaAdapter> {
    let a = Arc::new(base(t).with_action_controls(policy(commands)));
    publish(t, &a).await;
    a
}
fn core(t: &Setup, a: Arc<MijiaAdapter>, commands: &[Command]) -> Core {
    Core::new(
        t.f.store.clone(),
        vec![a],
        Arc::new(SystemClock),
        policy(commands),
    )
}

#[tokio::test]
async fn six_exact_actions_publish_and_execute_on_each_protected_protocol() {
    for modern in [false, true] {
        let mut count = 0;
        for (model, expected) in [
            ("abhome.light.wy0a01", 4),
            ("jdjz.fan.2072fc", 1),
            ("xiaomi.hood.cyp10", 1),
        ] {
            let t = setup(model, modern).await;
            assert_eq!(t.options.len(), expected);
            count += expected;
            let commands: Vec<_> = t
                .options
                .iter()
                .enumerate()
                .map(|(i, o)| command(&t, o, &format!("action-{i}")))
                .collect();
            if model == "abhome.light.wy0a01" {
                let example: Value = serde_json::from_str(include_str!(
                    "../../../../../../examples/capability.mijia-action.json"
                ))
                .unwrap();
                assert_eq!(
                    serde_json::to_value(&t.options[0].definition).unwrap(),
                    example["actions"]["toggle"]
                );
            }
            let a = enabled(&t, &commands).await;
            let core = core(&t, a.clone(), &commands);
            let description = core.describe(&t.device.id).unwrap();
            assert_eq!(
                description.device.components.len(),
                t.device.components.len()
            );
            assert_eq!(
                description
                    .capabilities
                    .iter()
                    .map(|c| c.actions.len())
                    .sum::<usize>(),
                expected
            );
            assert!(
                description
                    .capabilities
                    .iter()
                    .flat_map(|c| c.properties.values())
                    .all(|p| p["writable"] == false)
            );
            let before = bytes(&t.f, &t.account);
            let reads = t.p.reads.lock().unwrap().len();
            let auth = t.p.auth.lock().unwrap().len();
            let calls = t.f.cloud.calls.lock().unwrap().len();
            for (i, c) in commands.iter().enumerate() {
                let o = &t.options[i];
                assert_eq!(o.definition.idempotency, Idempotency::NonIdempotent);
                assert_eq!(o.definition.verification, VerificationMethod::Unavailable);
                assert!(
                    o.definition
                        .validate_arguments(&json!({"value":true}))
                        .is_err()
                );
                let r = core.execute(c.clone(), CancellationToken::new()).await;
                assert_eq!(r.status, Status::Accepted, "{:?}", r.error);
                assert_eq!(r.attempts.len(), 1);
                assert_eq!(r.attempts[0].phase, Phase::Accepted);
                assert!(r.verification.is_none());
                assert_eq!(
                    core.execute(c.clone(), CancellationToken::new())
                        .await
                        .status,
                    Status::Accepted
                );
                assert_eq!(t.w.count(), i + 1);
                let writes = t.w.writes.lock().unwrap();
                assert_eq!(writes[i].0, modern);
                assert_eq!(
                    writes[i].1["siid"],
                    if model == "xiaomi.hood.cyp10" { 6 } else { 2 }
                );
                assert_eq!(
                    writes[i].1["aiid"],
                    if model == "abhome.light.wy0a01" {
                        i + 1
                    } else {
                        1
                    }
                );
            }
            let mut verified = commands[0].clone();
            verified.request_id = "requested-verification".into();
            verified.wait_for = WaitFor::Verified;
            let r = core.execute(verified, CancellationToken::new()).await;
            assert_eq!(r.status, Status::Accepted);
            assert!(r.verification.is_none());
            assert_eq!(
                a.inspect_action_options(&t.device, &t.device.connections[0], &operation())
                    .await
                    .unwrap()
                    .len(),
                expected
            );
            // Adding actions does not invalidate ordinary state reads or history option inspection.
            core.inspect_history_options(&t.device.id, &t.device.connections[0].id, &operation())
                .await
                .unwrap();
            core.read(
                &t.device.id,
                &commands[0].component_id,
                Route::Cloud,
                false,
                &operation(),
            )
            .await
            .unwrap();
            assert_eq!(bytes(&t.f, &t.account), before);
            assert_eq!(t.p.auth.lock().unwrap().len(), auth);
            assert_eq!(t.w.count(), expected + 1);
            assert!(t.p.reads.lock().unwrap().len() <= reads + 1);
            assert!(t.f.cloud.calls.lock().unwrap().len() <= calls + 1);
        }
        assert_eq!(count, 6);
    }
}
#[tokio::test]
async fn both_core_and_independent_action_policy_are_required_before_send() {
    for modern in [false, true] {
        let t = setup("abhome.light.wy0a01", modern).await;
        let original = command(&t, &t.options[0], "seed");
        let enabled = enabled(&t, std::slice::from_ref(&original)).await;
        for mode in 0..9 {
            let mut c = original.clone();
            c.request_id = format!("refused-{mode}");
            let mut adapter = base(&t)
                .with_fan_controls()
                .with_reference_controls()
                .with_outlet_controls();
            let mut adapter_policy = policy(std::slice::from_ref(&c));
            match mode {
                0 => {}
                1 => adapter = adapter.with_action_controls(Policy::default()),
                2 => {
                    adapter = adapter
                        .with_control_policies(adapter_policy.clone(), adapter_policy.clone());
                }
                3 => {
                    adapter_policy.grants[0].device_id = Uuid::new_v4().to_string();
                    adapter = adapter.with_action_controls(adapter_policy);
                }
                4 => {
                    adapter_policy.grants[0].action = "brightness_up".into();
                    adapter = adapter.with_action_controls(adapter_policy);
                }
                5 => {
                    adapter_policy.grants[0].component_id = "light-6".into();
                    adapter = adapter.with_action_controls(adapter_policy);
                }
                6 => {
                    adapter_policy.grants[0].transports = vec![Transport::Lan];
                    adapter = adapter.with_action_controls(adapter_policy);
                }
                7 => {
                    adapter = adapter.with_action_controls(adapter_policy);
                }
                _ => {
                    adapter = adapter.with_action_controls(adapter_policy);
                    c.arguments = json!({"value":true});
                }
            }
            let core = core(
                &t,
                Arc::new(adapter),
                if mode == 7 {
                    &[]
                } else {
                    std::slice::from_ref(&c)
                },
            );
            let result = core.execute(c, CancellationToken::new()).await;
            assert_eq!(result.status, Status::Failed);
            assert!(result.attempts.iter().all(|a| a.phase == Phase::NotSent));
        }
        assert_eq!(t.w.count(), 0);
        let result = core(&t, enabled, std::slice::from_ref(&original))
            .execute(original, CancellationToken::new())
            .await;
        assert_eq!(result.status, Status::Accepted);
    }
}
#[tokio::test]
async fn action_outcomes_survive_reopen_revocation_conflicts_and_history_retirement() {
    for modern in [false, true] {
        for mode in [0, 1, 2, 4, 5, 6, 7, 8] {
            let t = setup("jdjz.fan.2072fc", modern).await;
            let c = command(&t, &t.options[0], "durable-action");
            let a = enabled(&t, std::slice::from_ref(&c)).await;
            t.w.mode.store(mode, Ordering::SeqCst);
            let core = core(&t, a, std::slice::from_ref(&c));
            let result = core.execute(c.clone(), CancellationToken::new()).await;
            let (status, phase) = match mode {
                0 => (Status::Accepted, Phase::Accepted),
                1 => (Status::Failed, Phase::NotSent),
                4 => (Status::Failed, Phase::Rejected),
                _ => (Status::Unknown, Phase::PossiblySent),
            };
            assert_eq!(result.status, status);
            assert_eq!(result.attempts[0].phase, phase);
            assert!(result.verification.is_none());
            let count = t.w.count();
            let reopened = Arc::new(SqliteStore::open(&t.f.directory.0).unwrap());
            let vault = Arc::new(AccountVault::new(reopened.clone(), t.f.secrets.clone()));
            let adapter = Arc::new(MijiaAdapter::new(vault, t.w.clone(), Arc::new(SystemClock)));
            let fresh = Core::new(
                reopened.clone(),
                vec![adapter.clone()],
                Arc::new(SystemClock),
                policy(std::slice::from_ref(&c)),
            );
            let revoked = Core::new(
                reopened.clone(),
                vec![adapter],
                Arc::new(SystemClock),
                Policy::default(),
            );
            let denied = revoked.execute(c.clone(), CancellationToken::new()).await;
            assert_eq!(denied.status, Status::Failed);
            assert_eq!(denied.error.unwrap().code, "POLICY_DENIED");
            assert_eq!(
                reopened.execution(&c.request_id).unwrap().unwrap().1.status,
                status
            );
            assert_eq!(
                fresh
                    .execute(c.clone(), CancellationToken::new())
                    .await
                    .status,
                status
            );
            assert_eq!(t.w.count(), count);
            let mut changed = c.clone();
            changed.wait_for = WaitFor::Verified;
            assert_eq!(
                fresh
                    .execute(changed, CancellationToken::new())
                    .await
                    .error
                    .unwrap()
                    .code,
                "REQUEST_CONFLICT"
            );
            assert_eq!(reopened.clear_execution_history().unwrap(), 1);
            let retired = fresh.execute(c, CancellationToken::new()).await;
            assert_eq!(retired.status, Status::Failed);
            assert_eq!(retired.error.unwrap().code, "REQUEST_RETIRED");
            assert_eq!(t.w.count(), count);
        }
    }
}
#[tokio::test]
async fn pending_action_cancel_deadline_drop_and_concurrent_duplicates_never_replay() {
    for modern in [false, true] {
        for mode in 0..3 {
            let t = setup("jdjz.fan.2072fc", modern).await;
            let mut c = command(&t, &t.options[0], "pending-action");
            if mode == 1 {
                c.timeout_ms = 100;
            }
            let a = enabled(&t, std::slice::from_ref(&c)).await;
            let core = core(&t, a, std::slice::from_ref(&c));
            t.w.mode.store(3, Ordering::SeqCst);
            let cancellation = CancellationToken::new();
            let mut work = Box::pin(core.execute(c.clone(), cancellation.clone()));
            tokio::select! {_=t.w.started.notified()=>{},_=&mut work=>panic!("Expected pending action")}
            assert!(t.f.vault.lease(t.account.id, &operation()).await.is_err());
            assert!(t.f.store.lock_device(&t.device.id).is_err());
            assert!(t.f.store.clear_execution_history().is_err());
            let concurrent = core.execute(c.clone(), CancellationToken::new()).await;
            assert!(matches!(
                concurrent.status,
                Status::Failed | Status::Unknown
            ));
            assert_eq!(t.w.count(), 1);
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
            assert_eq!(t.w.count(), 1);
        }
    }
}
#[tokio::test]
async fn unknown_legacy_action_is_not_replayed_after_modern_adoption() {
    let t = setup("jdjz.fan.2072fc", false).await;
    let c = command(&t, &t.options[0], "adopted-action");
    let a = enabled(&t, std::slice::from_ref(&c)).await;
    let core = core(&t, a.clone(), std::slice::from_ref(&c));
    t.w.mode.store(2, Ordering::SeqCst);
    assert_eq!(
        core.execute(c.clone(), CancellationToken::new())
            .await
            .status,
        Status::Unknown
    );
    let modern = connect(&a).await;
    assert_eq!(modern.account.id, t.account.id);
    let report = a
        .refresh_modern_devices(t.account.id, &operation())
        .await
        .unwrap();
    t.f.publish(&report).await;
    assert_eq!(
        core.execute(c, CancellationToken::new()).await.status,
        Status::Unknown
    );
    assert_eq!(t.w.count(), 1);
    assert!(!t.w.writes.lock().unwrap()[0].0);
}

#[tokio::test]
async fn action_inspection_refuses_foreign_identity_and_preserves_read_only_scope() {
    for modern in [false, true] {
        let t = setup("abhome.light.wy0a01", modern).await;
        let a = base(&t);
        let before = bytes(&t.f, &t.account);
        let calls = t.f.cloud.calls.lock().unwrap().len();
        let reads = t.p.reads.lock().unwrap().len();
        for mode in 0..5 {
            let mut device = t.device.clone();
            let mut connection = device.connections[0].clone();
            let op = operation();
            match mode {
                0 => device.model = "foreign.light".into(),
                1 => connection.identity_ref = Uuid::new_v4().to_string(),
                2 => connection.account_id = Some(Uuid::new_v4().to_string()),
                3 => device.components.clear(),
                _ => op.cancellation.cancel(),
            }
            assert!(
                a.inspect_action_options(&device, &connection, &op)
                    .await
                    .is_err()
            );
        }
        assert_eq!(bytes(&t.f, &t.account), before);
        assert_eq!(t.f.cloud.calls.lock().unwrap().len(), calls);
        assert_eq!(t.p.reads.lock().unwrap().len(), reads);
        assert_eq!(t.w.count(), 0);
    }
    for model in ["zhimi.airpurifier.v6"] {
        let t = setup(model, false).await;
        assert!(t.options.is_empty());
    }
}
#[tokio::test]
async fn public_action_metadata_cannot_expand_protected_arguments_identity_or_semantics() {
    for modern in [false, true] {
        for mode in 0..4 {
            let t = setup("abhome.light.wy0a01", modern).await;
            let mut c = command(&t, &t.options[0], "forged-action");
            if mode == 0 {
                c.arguments = json!({"in":[1],"confirmed":true});
            }
            if mode == 1 {
                c.action = "reset".into();
            }
            let a = enabled(&t, std::slice::from_ref(&c)).await;
            let mut d = t.f.core.describe(&t.device.id).unwrap();
            let cap = d
                .capabilities
                .iter_mut()
                .find(|cap| {
                    cap.id == c.capability && cap.component_id.as_deref() == Some(&c.component_id)
                })
                .unwrap();
            let mut action = cap.actions["toggle"].clone();
            action.parameters = json!({"type":"object"});
            action.idempotency = Idempotency::SetValue;
            cap.actions.insert(c.action.clone(), action);
            if mode == 2 {
                d.device.connections[0].identity_ref = Uuid::new_v4().to_string();
            }
            if mode == 3 {
                c.arguments = json!(null);
            }
            if mode == 2 {
                assert_eq!(
                    t.f.store.save_device(&d).unwrap_err().code,
                    "DEVICE_IDENTITY_CONFLICT"
                );
                assert_eq!(t.w.count(), 0);
                continue;
            }
            t.f.store.save_device(&d).unwrap();
            let result = core(&t, a, std::slice::from_ref(&c))
                .execute(c, CancellationToken::new())
                .await;
            assert_eq!(result.status, Status::Failed);
            assert!(result.attempts.iter().all(|a| a.phase == Phase::NotSent));
            assert_eq!(t.w.count(), 0);
        }
    }
}
#[tokio::test]
async fn protected_action_binding_tampering_and_missing_modern_transport_refuse_before_send() {
    for modern in [false, true] {
        for mode in 0..7 {
            let t = setup("jdjz.fan.2072fc", modern).await;
            let c = command(&t, &t.options[0], "protected-action");
            let mut a = enabled(&t, std::slice::from_ref(&c)).await;
            let mut saved = payload(&t.f, &t.account);
            match mode {
                0 => saved["bindings"][0]["listed"] = json!(false),
                1 => saved["bindings"][0]["model"] = json!("foreign.fan"),
                2 => saved["bindings"][0]["mapping"]["spec_type"] = json!("foreign-specification"),
                3 => {
                    saved["bindings"][0]["mapping"]["source"]["services"][0]["type"] =
                        json!("urn:miot-spec-v2:service:fan:00007808:foreign-fan:1")
                }
                4 => {
                    if modern {
                        saved["modern"] = Value::Null;
                    } else {
                        saved["session"] = Value::Null;
                    }
                }
                5 => {
                    if modern {
                        saved["bindings"][0]["modern"]["parent"] =
                            json!("PRIVATE-UNADMITTED-PARENT");
                    } else {
                        saved["bindings"][0]["reported_spec"] = json!("foreign-specification");
                    }
                }
                _ => {
                    if modern {
                        a = Arc::new(
                            MijiaAdapter::new(
                                t.f.vault.clone(),
                                t.w.clone(),
                                Arc::new(SystemClock),
                            )
                            .with_modern_protocol(t.p.clone(), t.p.clone())
                            .with_action_controls(policy(std::slice::from_ref(&c))),
                        );
                    } else {
                        saved["bindings"][0]["connection_id"] = json!(Uuid::new_v4().to_string());
                    }
                }
            }
            t.f.secrets.values.lock().unwrap().insert(
                t.account.credential_ref.to_string(),
                serde_json::to_vec(&saved).unwrap(),
            );
            let inspected = a
                .inspect_action_options(&t.device, &t.device.connections[0], &operation())
                .await;
            assert!(inspected.is_err() || inspected.unwrap().is_empty());
            let result = core(&t, a, std::slice::from_ref(&c))
                .execute(c, CancellationToken::new())
                .await;
            assert_eq!(
                result.status,
                Status::Failed,
                "modern={modern}, mode={mode}"
            );
            assert!(result.attempts.iter().all(|a| a.phase == Phase::NotSent));
            assert_eq!(t.w.count(), 0);
        }
    }
}

#[path = "actions/power.rs"]
mod power;
