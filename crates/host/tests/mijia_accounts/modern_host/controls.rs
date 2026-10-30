use super::*;
use homeverb_mijia::{ModernControlRequest, ModernControlResponse, ModernControlTransport};

#[derive(Default)]
struct Writer {
    sends: AtomicUsize,
    lost: AtomicBool,
}
#[async_trait]
impl ModernControlTransport for Writer {
    async fn send_modern_control(
        &self,
        request: &ModernControlRequest,
        progress: &WriteProgress,
        _: &Operation,
    ) -> Result<ModernControlResponse> {
        assert_eq!(
            request.url(),
            "https://core.api.mijia.tech/app/miotspec/prop/set"
        );
        assert!(request.cookie().contains("PRIVATE-MODERN"));
        assert!(!request.cookie().contains("passToken"));
        let fields: HashMap<_, _> = url::form_urlencoded::parse(request.body())
            .into_owned()
            .collect();
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
        assert_eq!(
            data["params"][0],
            json!({"did":"PRIVATE-MODERN-DID","siid":2,"piid":1,"value":false})
        );
        progress.mark_sent();
        self.sends.fetch_add(1, Ordering::SeqCst);
        if self.lost.load(Ordering::SeqCst) {
            return Err(homeverb_domain::Error::new(
                "MIJIA_REQUEST_FAILED",
                "Fictional lost acknowledgement.",
            ));
        }
        let reply =
            json!({"code":0,"result":[{"did":"PRIVATE-MODERN-DID","siid":2,"piid":1,"code":0}]});
        Ok(ModernControlResponse {
            body: secret(
                STANDARD
                    .encode(stream(&key, &serde_json::to_vec(&reply).unwrap()))
                    .as_bytes(),
            ),
            encoding: ModernContentEncoding::Plain,
        })
    }
}
async fn connected(model: &str) -> (ModernFixture, Value, Value) {
    let f = ModernFixture::new();
    *f.peer.directory.lock().unwrap() = Some(
        json!({"list":[{"did":"PRIVATE-MODERN-DID","name":"Modern appliance","model":model,"pid":0}],"has_more":false}),
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
    policy: Policy,
    enabled: bool,
    outlet: bool,
) -> Session {
    let a = MijiaAdapter::new(f.vault.clone(), f.cloud.clone(), Arc::new(SystemClock))
        .with_modern_protocol(f.peer.clone(), f.peer.clone())
        .with_modern_control_transport(w);
    let a = if enabled {
        if outlet {
            a.with_outlet_controls()
                .with_control_policies(Policy::default(), policy.clone())
        } else {
            a.with_fan_controls()
        }
    } else {
        a
    };
    let a = Arc::new(a);
    let core = Core::new(
        Arc::new(SqliteStore::open(&f.directory).unwrap()),
        vec![a.clone()],
        Arc::new(SystemClock),
        policy,
    );
    Session::with_platform_accounts(core, None, Some(a))
}
async fn call(session: &Session, request: Value) -> Value {
    session
        .call(
            serde_json::from_value(json!({
                "schema_version": "1.0",
                "operation_id": "modern-control",
                "timeout_ms": 5000,
                "request": request,
            }))
            .unwrap(),
        )
        .await
}
fn command(card: &Value, outlet: bool) -> Value {
    json!({"schema_version":"1.0","request_id":"modern-native-command","device_id":card["id"],"component_id":if outlet {"switch-2"} else {"fan-2"},"capability":"power","action":"set","arguments":{"state":false},"connection":"cloud","timeout_ms":5000,"wait_for":"accepted"})
}
fn policy(card: &Value, outlet: bool) -> Policy {
    if outlet {
        homeverb_host::mijia_outlet_policy(&[homeverb_host::MijiaOutletGrant {
            device_id: card["id"].as_str().unwrap().into(),
            component_id: "switch-2".into(),
            property: "state".into(),
            value: json!(false),
        }])
        .unwrap()
    } else {
        homeverb_host::mijia_fan_policy(&[homeverb_host::MijiaFanGrant {
            device_id: card["id"].as_str().unwrap().into(),
            component_id: "fan-2".into(),
            property: "state".into(),
            value: json!(false),
        }])
        .unwrap()
    }
}
#[tokio::test]
async fn native_modern_options_grant_refresh_plan_and_execute_use_only_the_selected_protocol() {
    let (f, account, card) = connected("jdjz.fan.2072fc").await;
    let w = Arc::new(Writer::default());
    let read_only = reopen(&f, w.clone(), Policy::default(), false, false);
    let before = f.peer.reads.load(Ordering::SeqCst);
    let options=call(&read_only,json!({"method":"inspect_control_options","device_id":card["id"],"connection_id":card["connections"][0]["id"]})).await;
    assert!(options["error"].is_null(), "{options}");
    assert!(!options["data"]["options"].as_array().unwrap().is_empty());
    assert_eq!(options["data"]["authorization"], "not_checked");
    assert_eq!(f.peer.reads.load(Ordering::SeqCst), before);
    let c = command(&card, false);
    assert!(call(&read_only, json!({"method":"plan","command":c})).await["error"].is_object());
    assert_eq!(w.sends.load(Ordering::SeqCst), 0);
    read_only.close();
    let session = reopen(&f, w.clone(), policy(&card, false), true, false);
    assert!(
        call(
            &session,
            json!({"method":"refresh_account","account_id":account})
        )
        .await["error"]
            .is_null()
    );
    assert!(call(&session, json!({"method":"plan","command":c})).await["error"].is_null());
    let result = call(&session, json!({"method":"execute","command":c})).await;
    assert_eq!(result["status"], "accepted", "{result}");
    call(&session, json!({"method":"execute","command":c})).await;
    assert_eq!(w.sends.load(Ordering::SeqCst), 1);
    assert_eq!(f.peer.auth_count(), 4);
    assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    session.close();
}
#[tokio::test]
async fn native_modern_outlet_consumes_actual_preview_and_preserves_unknown_receipts() {
    for lost in [false, true] {
        let (f, account, card) = connected("cuco.plug.v3").await;
        let w = Arc::new(Writer::default());
        let session = reopen(&f, w.clone(), policy(&card, true), true, true);
        assert!(
            call(
                &session,
                json!({"method":"refresh_account","account_id":account})
            )
            .await["error"]
                .is_null()
        );
        let c = command(&card, true);
        assert!(call(&session, json!({"method":"plan","command":c})).await["error"].is_object());
        let prepare = || {
            serde_json::from_value(
                json!({"schema_version":"1.0","operation_id":"preview","command":c}),
            )
            .unwrap()
        };
        let execute = |id: Value| {
            serde_json::from_value(
                json!({"schema_version":"1.0","operation_id":"consent","approval_id":id}),
            )
            .unwrap()
        };
        let preview = session.prepare_native_approval(prepare()).await;
        assert_eq!(preview["data"]["action"]["risk"], "elevated");
        assert_eq!(w.sends.load(Ordering::SeqCst), 0);
        session.set_foreground(false);
        session.set_foreground(true);
        assert_eq!(
            session
                .execute_native_approved(execute(preview["data"]["approval_id"].clone()))
                .await["error"]["code"],
            "APPROVAL_INVALID"
        );
        let preview = session.prepare_native_approval(prepare()).await;
        w.lost.store(lost, Ordering::SeqCst);
        let result = session
            .execute_native_approved(execute(preview["data"]["approval_id"].clone()))
            .await;
        assert_eq!(
            result["status"],
            if lost { "unknown" } else { "accepted" },
            "{result}"
        );
        assert_eq!(
            session
                .execute_native_approved(execute(preview["data"]["approval_id"].clone()))
                .await["error"]["code"],
            "APPROVAL_INVALID"
        );
        call(&session, json!({"method":"execute","command":c})).await;
        assert_eq!(w.sends.load(Ordering::SeqCst), 1);
        assert_eq!(f.peer.auth_count(), 4);
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
        session.close();
    }
}
