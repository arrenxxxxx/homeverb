use homeverb_host::{MijiaOutletGrant, mijia_outlet_policy};
use serde_json::{Value, json};

fn entries() -> Vec<MijiaOutletGrant> {
    let example: Value = serde_json::from_str(include_str!(
        "../../../examples/native-open.mijia-outlet.json"
    ))
    .unwrap();
    serde_json::from_value(example["mijia_outlet_grants"].clone()).unwrap()
}

#[test]
fn exact_outlet_grants_preserve_all_nine_targets_and_cloud_scope() {
    let grants = entries();
    let policy = mijia_outlet_policy(&grants).unwrap();
    assert_eq!(policy.grants.len(), 9);
    for (input, output) in grants.iter().zip(policy.grants) {
        assert_eq!(output.device_id, input.device_id);
        assert_eq!(output.component_id, input.component_id);
        assert_eq!(
            output.argument_schema,
            json!({"const":{input.property.clone():input.value}})
        );
        assert_eq!(output.transports, vec![homeverb_domain::Transport::Cloud]);
    }
}

#[test]
fn malformed_duplicates_wrong_family_and_unreviewed_targets_are_denied() {
    for entry in entries() {
        for value in [
            Value::Null,
            json!("true"),
            json!({}),
            json!(-1),
            json!(3),
            json!(1.5),
        ] {
            let mut bad = entry.clone();
            bad.value = value;
            assert!(mijia_outlet_policy(&[bad]).is_err());
        }
        for id in [
            "not-an-id",
            "40000000-0000-1000-8000-000000000001",
            "40000000-0000-4000-0000-000000000001",
        ] {
            let mut bad = entry.clone();
            bad.device_id = id.into();
            assert!(mijia_outlet_policy(&[bad]).is_err());
        }
        let mut bad = entry.clone();
        bad.component_id = "light-2".into();
        assert!(mijia_outlet_policy(&[bad]).is_err());
        assert!(mijia_outlet_policy(&[entry.clone(), entry]).is_err());
    }
    let mut one = entries()[2].clone();
    one.value = json!(1);
    let mut floating = one.clone();
    floating.value = json!(1.0);
    assert!(mijia_outlet_policy(&[one.clone(), floating]).is_err());
    assert!(mijia_outlet_policy(&vec![one; 65]).is_err());
    assert!(serde_json::from_value::<MijiaOutletGrant>(json!({"device_id":"40000000-0000-4000-8000-000000000001","component_id":"switch-2","property":"state","value":true,"confirmed":true})).is_err());
}

#[test]
fn invalid_targets_do_not_create_a_profile() {
    let directory =
        std::env::temp_dir().join(format!("homeverb-invalid-outlet-{}", uuid::Uuid::new_v4()));
    let mut grant = entries()[0].clone();
    grant.property = "unlock".into();
    assert!(
        homeverb_host::open_mijia_session_with_outlets(&directory, &[], &[], &[grant.clone()])
            .is_err()
    );
    assert!(!directory.exists());
    assert!(
        homeverb_host::open_platform_session_with_outlets(
            &directory,
            Default::default(),
            &[],
            &[],
            &[grant]
        )
        .is_err()
    );
    assert!(!directory.exists());
}

#[test]
fn additional_switch_outlet_targets_are_exact_and_independent() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../examples/native-open.mijia-switch-outlet.json"
    ))
    .unwrap();
    let inputs: Vec<MijiaOutletGrant> =
        serde_json::from_value(fixture["mijia_outlet_grants"].clone()).unwrap();
    let policy = mijia_outlet_policy(&inputs).unwrap();
    assert_eq!(policy.grants.len(), 15);
    for (input, output) in inputs.iter().zip(&policy.grants) {
        assert_eq!(output.device_id, input.device_id);
        assert_eq!(output.component_id, input.component_id);
        assert_eq!(
            output.argument_schema,
            json!({"const":{input.property.clone():input.value}})
        );
        assert_eq!(output.transports, vec![homeverb_domain::Transport::Cloud]);
        for field in ["state", "on_1", "child_lock_7", "default_power_on_state_2"] {
            if field == input.property {
                continue;
            }
            let mut bad = input.clone();
            bad.property = field.into();
            assert!(mijia_outlet_policy(&[bad]).is_err());
        }
        let mut bad = input.clone();
        bad.value = json!("true");
        assert!(mijia_outlet_policy(&[bad]).is_err());
        assert!(mijia_outlet_policy(&[input.clone(), input.clone()]).is_err());
    }
    assert!(policy.grants.iter().any(|g| g.capability == "vendor.xiaomi.mijia.custom" && g.action == "set_child_lock_7"));
}
