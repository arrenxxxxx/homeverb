use homeverb_domain::{Capability, Command, Risk, Route, Transport};
use homeverb_host::{MeijuControlGrants, MeijuDishwasherCycleGrant, meiju_dishwasher_cycle_policy};
use homeverb_midea::{DISHWASHER_CYCLE_ACTIONS, DishwasherCycleAction};
use serde_json::json;
use uuid::Uuid;

fn grant(id: &str, action: DishwasherCycleAction) -> MeijuDishwasherCycleGrant {
    MeijuDishwasherCycleGrant {
        device_id: id.into(),
        action,
    }
}

#[test]
fn exact_cycle_permissions_remain_independent_and_require_approval() {
    let id = Uuid::new_v4().to_string();
    let cap: Capability = serde_json::from_str(include_str!(
        "../../../examples/capability.meiju-dishwasher-cycle.json"
    ))
    .unwrap();
    for action in DISHWASHER_CYCLE_ACTIONS {
        let policy = meiju_dishwasher_cycle_policy(&[grant(&id, action)]).unwrap();
        let mut c: Command = serde_json::from_str(include_str!(
            "../../../examples/command.meiju-dishwasher-cycle.json"
        ))
        .unwrap();
        c.device_id = id.clone();
        c.action = action.action_name().into();
        let contract = &cap.actions[action.action_name()];
        assert_eq!(
            policy.authorize(&c, contract).unwrap_err().code,
            "POLICY_DENIED"
        );
        // This copy isolates grant matching; production metadata stays elevated.
        let mut ordinary = contract.clone();
        ordinary.risk = Risk::Low;
        assert_eq!(
            policy.authorize(&c, &ordinary).unwrap(),
            vec![Transport::Cloud]
        );
        for field in [
            "device",
            "component",
            "capability",
            "action",
            "start",
            "arguments",
            "route",
        ] {
            let mut bad = c.clone();
            match field {
                "device" => bad.device_id = Uuid::new_v4().to_string(),
                "component" => bad.component_id = "freezer".into(),
                "capability" => bad.capability = "power".into(),
                "action" => {
                    bad.action = DISHWASHER_CYCLE_ACTIONS
                        .into_iter()
                        .find(|value| *value != action)
                        .unwrap()
                        .action_name()
                        .into()
                }
                "start" => {
                    bad.action = "start_cycle".into();
                    bad.arguments = json!({"program":"eco_wash"});
                }
                "arguments" => bad.arguments = json!({"confirmed":true}),
                _ => bad.connection = Route::Lan,
            }
            assert!(policy.authorize(&bad, &ordinary).is_err(), "{field}");
        }
    }
    let grants: Vec<_> = DISHWASHER_CYCLE_ACTIONS
        .into_iter()
        .map(|action| grant(&id, action))
        .collect();
    assert_eq!(
        meiju_dishwasher_cycle_policy(&grants).unwrap().grants.len(),
        3
    );
}

#[test]
fn cycle_grants_reject_defaults_duplicates_and_invalid_scope_before_storage() {
    let id = "abcdef00-1234-4567-89ab-123456abcdef";
    let action = DishwasherCycleAction::PauseCycle;
    assert!(
        meiju_dishwasher_cycle_policy(&[])
            .unwrap()
            .grants
            .is_empty()
    );
    for body in [
        json!({"device_id":id}),
        json!({"device_id":id,"action":"start_cycle"}),
        json!({"device_id":id,"action":"pause"}),
        json!({"device_id":id,"action":null}),
        json!({"device_id":id,"action":"pause_cycle","arguments":{}}),
        json!({"device_id":id,"action":"pause_cycle","confirmed":true}),
    ] {
        assert!(serde_json::from_value::<MeijuDishwasherCycleGrant>(body).is_err());
    }
    for bad in [
        id.to_uppercase(),
        id.replace('-', ""),
        Uuid::nil().to_string(),
        "*".into(),
    ] {
        let dir = std::env::temp_dir().join(format!("homeverb-cycle-grant-{}", Uuid::new_v4()));
        let grants = [grant(&bad, action)];
        assert!(
            homeverb_host::open_meiju_with_grants(
                &dir,
                MeijuControlGrants {
                    dishwasher_cycle: &grants,
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert!(!dir.exists());
    }
    assert!(meiju_dishwasher_cycle_policy(&[grant(id, action), grant(id, action)]).is_err());
    let mut grants: Vec<_> = (0..64)
        .map(|_| grant(&Uuid::new_v4().to_string(), action))
        .collect();
    assert_eq!(
        meiju_dishwasher_cycle_policy(&grants).unwrap().grants.len(),
        64
    );
    grants.push(grant(id, action));
    assert!(meiju_dishwasher_cycle_policy(&grants).is_err());
}
