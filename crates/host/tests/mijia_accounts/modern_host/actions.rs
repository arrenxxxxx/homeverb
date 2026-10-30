use super::*;
use homeverb_host::{MijiaActionGrant, mijia_action_policy};
use homeverb_mijia::{ModernControlRequest, ModernControlResponse, ModernControlTransport};

#[derive(Default)]
struct Writer {
    sends: AtomicUsize,
    mode: AtomicUsize,
    started: tokio::sync::Notify,
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
        assert!(r.cookie().contains("PRIVATE-MODERN"));
        let fields: HashMap<_, _> = url::form_urlencoded::parse(r.body()).into_owned().collect();
        let key = Sha256::digest(
            [
                STANDARD.decode(&fields["ssecurity"]).unwrap(),
                STANDARD.decode(&fields["_nonce"]).unwrap(),
            ]
            .concat(),
        );
        let data: Value =
            serde_json::from_slice(&stream(&key, &STANDARD.decode(&fields["data"]).unwrap()))
                .unwrap();
        assert_eq!(data["params"]["did"], "PRIVATE-MODERN-DID");
        assert_eq!(data["params"]["in"], json!([]));
        assert_eq!(data["params"].as_object().unwrap().len(), 4);
        p.mark_sent();
        self.sends.fetch_add(1, Ordering::SeqCst);
        self.started.notify_one();
        match self.mode.load(Ordering::SeqCst) {
            1 => {
                return Err(homeverb_domain::Error::new(
                    "MIJIA_REQUEST_FAILED",
                    "Fictional lost reply.",
                ));
            }
            2 => std::future::pending::<()>().await,
            3 => op.cancellation.cancel(),
            _ => {}
        }
        let result = json!({"code":0,"result":{"did":data["params"]["did"],"siid":data["params"]["siid"],"aiid":data["params"]["aiid"],"code":0,"out":[]}});
        Ok(ModernControlResponse {
            body: secret(
                STANDARD
                    .encode(stream(&key, &serde_json::to_vec(&result).unwrap()))
                    .as_bytes(),
            ),
            encoding: ModernContentEncoding::Plain,
        })
    }
}
async fn connected(model: &str) -> (ModernFixture, Value, Value) {
    let f = ModernFixture::new();
    *f.peer.directory.lock().unwrap() = Some(
        json!({"list":[{"did":"PRIVATE-MODERN-DID","name":"Fictional action appliance","model":model,"pid":0}],"has_more":false}),
    );
    let qr = f.begin().await;
    let report = serde_json::to_value(
        f.session
            .continue_modern_device_login(answer_metadata(qr.challenge_id))
            .await
            .unwrap(),
    )
    .unwrap();
    let cards = f.call(json!({"method":"list_devices"})).await;
    let card = cards["data"][0].clone();
    f.session.close();
    (f, report["account"]["id"].clone(), card)
}
fn reopen(
    f: &ModernFixture,
    w: Arc<Writer>,
    grants: &[MijiaActionGrant],
    enabled: bool,
) -> Session {
    let policy = mijia_action_policy(grants).unwrap();
    let a = MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
        .with_modern_protocol(f.peer.clone(), f.peer.clone())
        .with_modern_control_transport(w);
    let a = Arc::new(if enabled {
        a.with_action_controls(policy.clone())
    } else {
        a
    });
    let core = Core::new(
        Arc::new(SqliteStore::open(&f.directory).unwrap()),
        vec![a.clone()],
        Arc::new(SystemClock),
        policy,
    );
    Session::with_platform_accounts(core, None, Some(a))
}
async fn call(s: &Session, r: Value) -> Value {
    let method = r["method"].as_str().unwrap().to_string();
    let result=s.call(serde_json::from_value(json!({"schema_version":"1.0","operation_id":"action-test","timeout_ms":5000,"request":r})).unwrap()).await;
    if method != "execute" {
        validate(&result, &method);
    }
    result
}
fn inspection(card: &Value, include: Option<bool>) -> Value {
    let mut r = json!({"method":"inspect_control_options","device_id":card["id"],"connection_id":card["connections"][0]["id"]});
    if let Some(include) = include {
        r["include_no_argument_actions"] = json!(include);
    }
    r
}
fn grant(card: &Value, o: &Value) -> MijiaActionGrant {
    serde_json::from_value(json!({"device_id":card["id"],"component_id":o["component_id"],"capability":o["capability"],"action":o["action"]})).unwrap()
}
fn command(g: &MijiaActionGrant, id: &str) -> Value {
    json!({"schema_version":"1.0","request_id":id,"device_id":g.device_id,"component_id":g.component_id,"capability":g.capability,"action":g.action,"arguments":{},"connection":"cloud","timeout_ms":5000,"wait_for":"accepted"})
}
#[tokio::test]
async fn native_action_inspection_is_opt_in_local_and_requires_independent_startup_grants() {
    for (model, count) in [
        ("abhome.light.wy0a01", 4),
        ("jdjz.fan.2072fc", 1),
        ("xiaomi.hood.cyp10", 1),
    ] {
        let (f, account, card) = connected(model).await;
        let w = Arc::new(Writer::default());
        let read_only = reopen(&f, w.clone(), &[], false);
        let before = f.peer.reads.load(Ordering::SeqCst);
        let auth = f.peer.auth_count();
        let old = call(&read_only, inspection(&card, None)).await;
        assert_eq!(old, call(&read_only, inspection(&card, Some(false))).await);
        let expanded = call(&read_only, inspection(&card, Some(true))).await;
        let options: Vec<_> = expanded["data"]["options"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|o| o["definition"]["idempotency"] == "non_idempotent")
            .collect();
        assert_eq!(options.len(), count);
        assert_eq!(
            expanded["data"]["options"].as_array().unwrap().len(),
            old["data"]["options"].as_array().unwrap().len() + count
        );
        assert_eq!(f.peer.reads.load(Ordering::SeqCst), before);
        assert_eq!(f.peer.auth_count(), auth);
        assert_eq!(w.sends.load(Ordering::SeqCst), 0);
        let grants: Vec<_> = options.into_iter().map(|o| grant(&card, o)).collect();
        read_only.close();
        let s = reopen(&f, w.clone(), &grants, true);
        assert!(
            call(&s, json!({"method":"refresh_account","account_id":account})).await["error"]
                .is_null()
        );
        for (i, g) in grants.iter().enumerate() {
            let c = command(g, &format!("native-action-{i}"));
            assert!(call(&s, json!({"method":"plan","command":c})).await["error"].is_null());
            assert_eq!(
                call(&s, json!({"method":"execute","command":c})).await["status"],
                "accepted"
            );
            assert_eq!(
                call(&s, json!({"method":"execute","command":c})).await["status"],
                "accepted"
            );
            assert_eq!(w.sends.load(Ordering::SeqCst), i + 1);
        }
        s.close();
        let refused = reopen(&f, w.clone(), &grants, false);
        let c = command(&grants[0], "disabled-action-family");
        assert_eq!(
            call(&refused, json!({"method":"execute","command":c})).await["status"],
            "failed"
        );
        assert_eq!(w.sends.load(Ordering::SeqCst), count);
    }
}
#[tokio::test]
async fn native_action_unknown_reopens_without_replay_and_background_cancels_pending_send() {
    for mode in [1, 2, 3] {
        let (f, account, card) = connected("jdjz.fan.2072fc").await;
        let w = Arc::new(Writer::default());
        let ro = reopen(&f, w.clone(), &[], false);
        let inspection = call(&ro, inspection(&card, Some(true))).await;
        let o = inspection["data"]["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["action"] == "toggle")
            .unwrap();
        let grants = vec![grant(&card, o)];
        ro.close();
        let s = reopen(&f, w.clone(), &grants, true);
        assert!(
            call(&s, json!({"method":"refresh_account","account_id":account})).await["error"]
                .is_null()
        );
        w.mode.store(mode, Ordering::SeqCst);
        let c = command(&grants[0], "native-action-unknown");
        let result = if mode == 2 {
            let mut work = Box::pin(call(&s, json!({"method":"execute","command":c})));
            tokio::select! {_=w.started.notified()=>{},_=&mut work=>panic!("Expected pending action")}
            s.set_foreground(false);
            let result = work.await;
            s.set_foreground(true);
            result
        } else {
            call(&s, json!({"method":"execute","command":c})).await
        };
        assert_eq!(result["status"], "unknown");
        s.close();
        let fresh = reopen(&f, w.clone(), &grants, true);
        assert_eq!(
            call(&fresh, json!({"method":"execute","command":c})).await["status"],
            "unknown"
        );
        assert_eq!(w.sends.load(Ordering::SeqCst), 1);
        fresh.set_foreground(false);
        assert_eq!(
            call(&fresh, inspection_request(&card)).await["error"]["code"],
            "HOST_BACKGROUND"
        );
    }
}
fn inspection_request(card: &Value) -> Value {
    inspection(card, Some(true))
}
