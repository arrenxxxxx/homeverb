use homeverb_core::Policy;
use homeverb_domain::{Action, Command, Transport};
use homeverb_host::{MeijuPowerGrant, meiju_power_policy};
use serde_json::json;
use uuid::Uuid;

fn grant(device: &str, state: bool) -> MeijuPowerGrant {
    MeijuPowerGrant {
        device_id: device.into(),
        state,
    }
}

fn command(device: &str, state: bool) -> Command {
    serde_json::from_value(json!({"schema_version":"1.0","request_id":"power-test",
        "device_id":device,"component_id":"main","capability":"power","action":"set",
        "arguments":{"state":state},"connection":"auto","timeout_ms":1000,"wait_for":"accepted"}))
    .unwrap()
}

#[test]
fn startup_grants_authorize_only_exact_device_state_component_action_and_cloud() {
    let id = Uuid::new_v4().to_string();
    let policy = meiju_power_policy(&[grant(&id, false)]).unwrap();
    let action: Action = serde_json::from_value(json!({"parameters":{"type":"object"},
        "transports":["lan","cloud"],"idempotency":"set_value","risk":"low",
        "atomic":false,"verification":"unavailable"}))
    .unwrap();
    let allowed = command(&id, false);
    assert_eq!(
        policy.authorize(&allowed, &action).unwrap(),
        vec![Transport::Cloud]
    );
    let denied = |policy: &Policy, c: &Command, a: &Action| {
        assert_eq!(policy.authorize(c, a).unwrap_err().code, "POLICY_DENIED");
    };
    denied(&Policy::default(), &allowed, &action);
    denied(&policy, &command(&id, true), &action);
    denied(
        &policy,
        &command(&Uuid::new_v4().to_string(), false),
        &action,
    );
    for field in ["component", "capability", "action", "route", "arguments"] {
        let mut c = allowed.clone();
        match field {
            "component" => c.component_id = "freezer".into(),
            "capability" => c.capability = "climate".into(),
            "action" => c.action = "toggle".into(),
            "route" => c.connection = homeverb_domain::Route::Lan,
            _ => c.arguments = json!({"state":false,"confirmed":true}),
        }
        denied(&policy, &c, &action);
    }
    let mut elevated = action.clone();
    elevated.risk = homeverb_domain::Risk::Elevated;
    denied(&policy, &allowed, &elevated);
    let both = meiju_power_policy(&[grant(&id, false), grant(&id, true)]).unwrap();
    assert_eq!(
        both.authorize(&command(&id, true), &action).unwrap(),
        vec![Transport::Cloud]
    );
}

#[test]
fn malformed_duplicate_wildcard_and_oversized_lists_confer_no_authority() {
    let id = "abcdef00-1234-4567-89ab-123456abcdef".to_owned();
    assert!(meiju_power_policy(&[]).unwrap().grants.is_empty());
    for invalid in [
        "*".into(),
        "dev-mock-ac".into(),
        Uuid::nil().to_string(),
        id.to_uppercase(),
        id.replace('-', ""),
    ] {
        assert!(meiju_power_policy(&[grant(&invalid, false)]).is_err());
    }
    assert!(meiju_power_policy(&[grant(&id, false), grant(&id, false)]).is_err());
    let mut grants: Vec<_> = (0..64)
        .map(|_| grant(&Uuid::new_v4().to_string(), false))
        .collect();
    assert_eq!(meiju_power_policy(&grants).unwrap().grants.len(), 64);
    grants.push(grant(&Uuid::new_v4().to_string(), false));
    assert!(meiju_power_policy(&grants).is_err());
    for value in [
        json!({"device_id":id}),
        json!({"device_id":id,"state":0}),
        json!({"device_id":id,"state":false,"confirmed":true}),
    ] {
        assert!(serde_json::from_value::<MeijuPowerGrant>(value).is_err());
    }
}
