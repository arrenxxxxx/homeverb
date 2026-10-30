//! API 1.42 inspection and trusted maintenance journeys with fictional peers.
use super::*;
use homeverb_host::{ExecuteApprovalRequest, PrepareApprovalRequest};

fn inspect(card: &Value, low: bool, power: bool, maintenance: bool) -> Value {
    let mut r = inspection(card, Some(low));
    r["include_power_toggle_actions"] = json!(power);
    r["include_maintenance_actions"] = json!(maintenance);
    r
}
fn prepare(c: &Value) -> PrepareApprovalRequest {
    serde_json::from_value(
        json!({"schema_version":"1.0","operation_id":"maintenance-preview","command":c}),
    )
    .unwrap()
}
fn confirm(preview: &Value) -> ExecuteApprovalRequest {
    serde_json::from_value(json!({"schema_version":"1.0","operation_id":"maintenance-consent","approval_id":preview["data"]["approval_id"]})).unwrap()
}
async fn preview_for(s: &Session, c: &Value) -> Value {
    s.prepare_native_approval(prepare(c)).await
}
async fn execute(s: &Session, p: &Value) -> Value {
    s.execute_native_approved(confirm(p)).await
}
fn options(result: &Value) -> Vec<Value> {
    result["data"]["options"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| {
            matches!(
                o["action"].as_str(),
                Some("ptz_calibrate" | "restart_device" | "pop_up" | "stop_dry_cleaning")
            )
        })
        .cloned()
        .collect()
}

#[tokio::test]
async fn maintenance_inspection_is_independent_and_every_exact_action_requires_consent() {
    for (model, count) in [("xiaomi.camera.083ac1", 3), ("xiaomi.hood.cyp10", 1)] {
        let (f, account, card) = connected(model).await;
        let w = Arc::new(Writer::default());
        let ro = reopen(&f, w.clone(), &[], false);
        let reads = f.peer.reads.load(Ordering::SeqCst);
        let auth = f.peer.auth_count();
        assert_eq!(
            call(&ro, inspection(&card, None)).await,
            call(&ro, inspect(&card, false, false, false)).await
        );
        for low in [false, true] {
            let old = call(&ro, inspect(&card, low, false, false)).await;
            assert_eq!(old, call(&ro, inspect(&card, low, true, false)).await);
            let new = call(&ro, inspect(&card, low, false, true)).await;
            assert_eq!(new, call(&ro, inspect(&card, low, true, true)).await);
            assert_eq!(options(&new).len(), count);
            assert_eq!(
                new["data"]["options"].as_array().unwrap().len(),
                old["data"]["options"].as_array().unwrap().len() + count
            );
        }
        let expanded = call(&ro, inspect(&card, false, false, true)).await;
        let grants: Vec<_> = options(&expanded)
            .iter()
            .map(|o| {
                assert_eq!(o["definition"]["risk"], "elevated");
                assert_eq!(o["definition"]["idempotency"], "non_idempotent");
                assert_eq!(o["definition"]["verification"], "unavailable");
                grant(&card, o)
            })
            .collect();
        assert!(
            preview_for(&ro, &command(&grants[0], "read-only-maintenance")).await["error"]
                .is_object()
        );
        assert_eq!(f.peer.reads.load(Ordering::SeqCst), reads);
        assert_eq!(f.peer.auth_count(), auth);
        assert_eq!(w.sends.load(Ordering::SeqCst), 0);
        ro.close();
        let s = reopen(&f, w.clone(), &grants, true);
        assert!(
            call(&s, json!({"method":"refresh_account","account_id":account})).await["error"]
                .is_null()
        );
        for (i, g) in grants.iter().enumerate() {
            let c = command(g, &format!("native-maintenance-{i}"));
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
        let c = command(&grants[0], "disabled-maintenance-family");
        let preview = preview_for(&disabled, &c).await;
        assert!(preview["error"].is_null());
        assert_eq!(execute(&disabled, &preview).await["status"], "failed");
        assert_eq!(w.sends.load(Ordering::SeqCst), count);
        assert_eq!(f.cloud.calls.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn maintenance_opt_in_does_not_select_low_risk_or_power_toggle_options() {
    for model in ["abhome.light.wy0a01", "cuco.plug.v3"] {
        let (f, _, card) = connected(model).await;
        let w = Arc::new(Writer::default());
        let s = reopen(&f, w.clone(), &[], false);
        for low in [false, true] {
            for power in [false, true] {
                let old = call(&s, inspect(&card, low, power, false)).await;
                assert_eq!(old, call(&s, inspect(&card, low, power, true)).await);
            }
        }
        assert_eq!(w.sends.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn maintenance_dismiss_background_and_unknown_reopen_never_replay() {
    for (model, action) in [
        ("xiaomi.camera.083ac1", "ptz_calibrate"),
        ("xiaomi.camera.083ac1", "restart_device"),
        ("xiaomi.camera.083ac1", "pop_up"),
        ("xiaomi.hood.cyp10", "stop_dry_cleaning"),
    ] {
        for mode in [1, 2, 3] {
            let (f, account, card) = connected(model).await;
            let w = Arc::new(Writer::default());
            let ro = reopen(&f, w.clone(), &[], false);
            let inspected = options(&call(&ro, inspect(&card, false, false, true)).await);
            let grants = vec![grant(
                &card,
                inspected.iter().find(|o| o["action"] == action).unwrap(),
            )];
            ro.close();
            let s = reopen(&f, w.clone(), &grants, true);
            assert!(
                call(&s, json!({"method":"refresh_account","account_id":account})).await["error"]
                    .is_null()
            );
            let c = command(&grants[0], "native-maintenance-unknown");
            let preview = preview_for(&s, &c).await;
            assert!(preview["error"].is_null());
            s.set_foreground(false);
            assert_eq!(
                call(&s, inspect(&card, false, false, true)).await["error"]["code"],
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
                tokio::select! {_=w.started.notified()=>{},_=&mut work=>panic!("Expected pending maintenance")}
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
            assert_eq!(receipt["data"]["status"], "unknown");
            assert_eq!(w.sends.load(Ordering::SeqCst), 1);
        }
    }
}
