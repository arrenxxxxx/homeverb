use homeverb_domain::{Command, Idempotency, Transport};
use homeverb_host::{MijiaActionGrant, mijia_action_policy};
use serde_json::{Value, json};

fn examples() -> Vec<MijiaActionGrant> {
    let v: Value = serde_json::from_str(include_str!(
        "../../../examples/native-open.mijia-actions.json"
    ))
    .unwrap();
    serde_json::from_value(v["mijia_action_grants"].clone()).unwrap()
}
#[test]
fn six_action_grants_are_exact_empty_argument_cloud_authority() {
    let grants = examples();
    assert_eq!(grants.len(), 6);
    let policy = mijia_action_policy(&grants).unwrap();
    assert_eq!(policy.grants.len(), 6);
    for grant in grants {
        let option = homeverb_mijia::no_argument_action_option(
            &grant.component_id,
            &grant.capability,
            &grant.action,
        )
        .unwrap();
        assert_eq!(option.definition.idempotency, Idempotency::NonIdempotent);
        let mut c:Command=serde_json::from_value(json!({"schema_version":"1.0","request_id":"fictional-action","device_id":grant.device_id,"component_id":grant.component_id,"capability":grant.capability,"action":grant.action,"arguments":{},"connection":"cloud","timeout_ms":5000,"wait_for":"accepted"})).unwrap();
        assert_eq!(
            policy.authorize(&c, &option.definition).unwrap(),
            vec![Transport::Cloud]
        );
        for arguments in [
            json!({"value":true}),
            json!({"in":[]}),
            json!({"confirmed":true}),
            Value::Null,
        ] {
            c.arguments = arguments;
            assert!(policy.authorize(&c, &option.definition).is_err());
        }
        c.arguments = json!({});
        c.action = "set".into();
        assert!(policy.authorize(&c, &option.definition).is_err());
    }
}
#[test]
fn native_action_grants_refuse_duplicates_unreviewed_scopes_and_noncanonical_ids() {
    let valid = examples().remove(0);
    assert!(mijia_action_policy(&[valid.clone(), valid.clone()]).is_err());
    assert!(mijia_action_policy(&vec![valid.clone(); 65]).is_err());
    for mode in 0..10 {
        let mut g = valid.clone();
        match mode {
            0 => g.device_id = "*".into(),
            1 => g.device_id = "40000000-0000-5000-8000-000000000001".into(),
            2 => g.device_id = "ABCDEFAB-0000-4000-8000-000000000001".into(),
            3 => g.component_id = "light-3".into(),
            4 => g.component_id = "hood-2".into(),
            5 => g.capability = "power".into(),
            6 => g.action = "set".into(),
            7 => g.action = "reset".into(),
            8 => g.component_id = "switch-2".into(),
            _ => g.device_id = "40000000-0000-4000-7000-000000000001".into(),
        }
        assert!(mijia_action_policy(&[g]).is_err());
    }
    for key in ["arguments", "value", "siid", "aiid", "confirmed"] {
        let mut v = json!({"device_id":valid.device_id,"component_id":valid.component_id,"capability":valid.capability,"action":valid.action});
        v[key] = json!({});
        assert!(serde_json::from_value::<MijiaActionGrant>(v).is_err());
    }
}
#[test]
fn invalid_action_startup_does_not_create_a_profile() {
    let path =
        std::env::temp_dir().join(format!("homeverb-action-invalid-{}", uuid::Uuid::new_v4()));
    let mut g = examples().remove(0);
    g.action = "unreviewed".into();
    assert!(
        homeverb_host::open_mijia_session_with_actions(&path, &[], &[], &[], &[g.clone()]).is_err()
    );
    assert!(!path.exists());
    assert!(
        homeverb_host::open_platform_session_with_actions(
            &path,
            homeverb_host::MeijuControlGrants::default(),
            &[],
            &[],
            &[],
            &[g]
        )
        .is_err()
    );
    assert!(!path.exists());
}
