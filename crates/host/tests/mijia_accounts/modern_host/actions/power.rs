//! API 1.41 opt-in and trusted power-toggle journeys with fictional peers.
use super::*;
use homeverb_host::{ExecuteApprovalRequest, PrepareApprovalRequest};

fn inspect(card: &Value, low: bool, power: bool) -> Value {
    let mut r = inspection(card, Some(low));
    r["include_power_toggle_actions"] = json!(power);
    r
}
fn prepare(c: &Value) -> PrepareApprovalRequest {
    serde_json::from_value(
        json!({"schema_version":"1.0","operation_id":"power-preview","command":c}),
    )
    .unwrap()
}
fn confirm(preview: &Value) -> ExecuteApprovalRequest {
    serde_json::from_value(json!({"schema_version":"1.0","operation_id":"power-consent","approval_id":preview["data"]["approval_id"]})).unwrap()
}
async fn preview_for(s: &Session, c: &Value) -> Value {
    s.prepare_native_approval(prepare(c)).await
}
async fn execute(s: &Session, p: &Value) -> Value {
    s.execute_native_approved(confirm(p)).await
}
fn toggles(result: &Value) -> Vec<Value> {
    result["data"]["options"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["action"] == "toggle" && o["capability"] == "vendor.xiaomi.mijia.switch")
        .cloned()
        .collect()
}

#[tokio::test]
async fn all_power_options_are_separate_local_and_require_independent_trusted_consent() {
    for (model, count) in [
        ("cuco.plug.v3", 1),
        ("yutai.plug.jdls1", 1),
        ("opple.switch.0010d5", 1),
        ("opple.switch.0010d6", 2),
    ] {
        let (f, account, card) = connected(model).await;
        let w = Arc::new(Writer::default());
        let ro = reopen(&f, w.clone(), &[], false);
        let reads = f.peer.reads.load(Ordering::SeqCst);
        let auth = f.peer.auth_count();
        let old = call(&ro, inspection(&card, None)).await;
        assert_eq!(old, call(&ro, inspect(&card, false, false)).await);
        assert_eq!(old, call(&ro, inspect(&card, true, false)).await);
        let expanded = call(&ro, inspect(&card, false, true)).await;
        assert_eq!(expanded, call(&ro, inspect(&card, true, true)).await);
        let options = toggles(&expanded);
        assert_eq!(options.len(), count);
        assert_eq!(
            expanded["data"]["options"].as_array().unwrap().len(),
            old["data"]["options"].as_array().unwrap().len() + count
        );
        assert_eq!(f.peer.reads.load(Ordering::SeqCst), reads);
        assert_eq!(f.peer.auth_count(), auth);
        let grants: Vec<_> = options
            .iter()
            .map(|o| {
                assert_eq!(o["definition"]["risk"], "elevated");
                assert_eq!(o["definition"]["idempotency"], "non_idempotent");
                assert_eq!(o["definition"]["verification"], "unavailable");
                grant(&card, o)
            })
            .collect();
        assert!(preview_for(&ro, &command(&grants[0], "read-only")).await["error"].is_object());
        ro.close();
        let s = reopen(&f, w.clone(), &grants, true);
        assert!(
            call(&s, json!({"method":"refresh_account","account_id":account})).await["error"]
                .is_null()
        );
        for (i, g) in grants.iter().enumerate() {
            let c = command(g, &format!("native-power-{i}"));
            assert!(call(&s, json!({"method":"plan","command":c})).await["error"].is_object());
            assert_eq!(
                call(&s, json!({"method":"execute","command":c})).await["status"],
                "failed"
            );
            let preview = preview_for(&s, &c).await;
            assert_eq!(preview["data"]["action"]["risk"], "elevated", "{preview}");
            assert_eq!(w.sends.load(Ordering::SeqCst), i);
            let result = execute(&s, &preview).await;
            assert_eq!(result["status"], "accepted", "{result}");
            assert!(result["verification"].is_null());
            assert_eq!(
                execute(&s, &preview).await["error"]["code"],
                "APPROVAL_INVALID"
            );
            assert!(preview_for(&s, &c).await["error"].is_object());
            assert_eq!(w.sends.load(Ordering::SeqCst), i + 1);
        }
        s.close();
        let disabled = reopen(&f, w.clone(), &grants, false);
        let c = command(&grants[0], "disabled-family");
        let preview = preview_for(&disabled, &c).await;
        assert!(preview["error"].is_null(), "{preview}");
        assert_eq!(execute(&disabled, &preview).await["status"], "failed");
        assert_eq!(w.sends.load(Ordering::SeqCst), count);
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn power_opt_in_does_not_include_low_risk_actions() {
    let (f, _, card) = connected("abhome.light.wy0a01").await;
    let w = Arc::new(Writer::default());
    let s = reopen(&f, w.clone(), &[], false);
    let old = call(&s, inspection(&card, None)).await;
    assert_eq!(old, call(&s, inspect(&card, false, true)).await);
    let low = call(&s, inspect(&card, true, false)).await;
    assert_eq!(low, call(&s, inspect(&card, true, true)).await);
    assert_eq!(w.sends.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn power_approval_background_and_unknown_reopen_never_replay() {
    for mode in [1, 2, 3] {
        let (f, account, card) = connected("opple.switch.0010d6").await;
        let w = Arc::new(Writer::default());
        let ro = reopen(&f, w.clone(), &[], false);
        let options = toggles(&call(&ro, inspect(&card, false, true)).await);
        let grants = vec![grant(&card, &options[1])];
        ro.close();
        let s = reopen(&f, w.clone(), &grants, true);
        assert!(
            call(&s, json!({"method":"refresh_account","account_id":account})).await["error"]
                .is_null()
        );
        let c = command(&grants[0], "native-power-unknown");
        let preview = preview_for(&s, &c).await;
        assert!(preview["error"].is_null());
        s.set_foreground(false);
        assert_eq!(
            call(&s, inspect(&card, false, true)).await["error"]["code"],
            "HOST_BACKGROUND"
        );
        s.set_foreground(true);
        assert_eq!(
            execute(&s, &preview).await["error"]["code"],
            "APPROVAL_INVALID"
        );
        assert_eq!(w.sends.load(Ordering::SeqCst), 0);
        let dismissed = preview_for(&s, &c).await;
        assert!(s.discard_native_approval(dismissed["data"]["approval_id"].as_u64().unwrap()));
        assert_eq!(
            execute(&s, &dismissed).await["error"]["code"],
            "APPROVAL_INVALID"
        );
        let preview = preview_for(&s, &c).await;
        w.mode.store(mode, Ordering::SeqCst);
        let result = if mode == 2 {
            let mut work = Box::pin(execute(&s, &preview));
            tokio::select! {_=w.started.notified()=>{},_=&mut work=>panic!("Expected pending power toggle")}
            s.set_foreground(false);
            work.await
        } else {
            execute(&s, &preview).await
        };
        assert_eq!(result["status"], "unknown", "{result}");
        s.close();
        let fresh = reopen(&f, w.clone(), &grants, true);
        assert!(preview_for(&fresh, &c).await["error"].is_object());
        assert_eq!(
            call(&fresh, json!({"method":"execute","command":c})).await["status"],
            "failed"
        );
        let receipt = call(
            &fresh,
            json!({"method":"get_execution","request_id":c["request_id"]}),
        )
        .await;
        assert!(receipt["error"].is_null(), "{receipt}");
        assert_eq!(receipt["data"]["status"], "unknown", "{receipt}");
        assert_eq!(w.sends.load(Ordering::SeqCst), 1);
    }
}
