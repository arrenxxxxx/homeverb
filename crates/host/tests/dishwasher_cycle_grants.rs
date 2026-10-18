use homeverb_domain::{Capability, Command, Risk, Route, Transport};
use homeverb_host::{MeijuControlGrants, MeijuDishwasherCycleGrant, meiju_dishwasher_cycle_policy};
use homeverb_midea::{DISHWASHER_CYCLE_ACTIONS, DishwasherCycleAction};
use serde_json::json;
use uuid::Uuid;

fn grant(id: &str, action: DishwasherCycleAction) -> MeijuDishwasherCycleGrant {
    MeijuDishwasherCycleGrant {
        device_id: id.into(),
        action,
        arguments: json!({}),
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
        for other in DISHWASHER_CYCLE_ACTIONS {
            if other != action {
                let mut peer = c.clone();
                peer.action = other.action_name().into();
                assert!(policy.authorize(&peer, &ordinary).is_err());
            }
        }
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
        5
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

#[test]
fn additional_cycle_grants_match_exact_arguments_without_expanding_legacy_authority() {
    let id = "abcdef00-1234-4567-89ab-123456abcdef";
    let legacy: MeijuDishwasherCycleGrant =
        serde_json::from_value(json!({"device_id":id,"action":"pause_cycle"})).unwrap();
    assert_eq!(legacy.arguments, json!({}));
    let mut c: Command = serde_json::from_str(include_str!(
        "../../../examples/command.meiju-dishwasher-cycle.json"
    ))
    .unwrap();
    c.device_id = id.into();
    let cap: Capability = serde_json::from_str(include_str!(
        "../../../examples/capability.meiju-dishwasher-cycle-additional.json"
    ))
    .unwrap();
    for action in DISHWASHER_CYCLE_ACTIONS {
        c.action = action.action_name().into();
        let mut ordinary = cap.actions[action.action_name()].clone();
        ordinary.risk = Risk::Low;
        let empty_policy = meiju_dishwasher_cycle_policy(&[grant(id, action)]).unwrap();
        let mut grants = vec![];
        for code in [0, 9, 10, 13, 14] {
            let mut exact = grant(id, action);
            exact.arguments = json!({"additional":code});
            let policy = meiju_dishwasher_cycle_policy(&[exact.clone()]).unwrap();
            c.arguments = exact.arguments.clone();
            assert!(policy.authorize(&c, &ordinary).is_ok());
            assert!(empty_policy.authorize(&c, &ordinary).is_err());
            for other in [
                json!({}),
                json!({"additional":if code == 9 { 10 } else { 9 }}),
            ] {
                c.arguments = other;
                assert!(policy.authorize(&c, &ordinary).is_err());
            }
            grants.push(exact);
        }
        assert_eq!(
            meiju_dishwasher_cycle_policy(&grants).unwrap().grants.len(),
            5
        );
        let mut duplicate = grants[1].clone();
        duplicate.arguments = json!({"additional":9.0});
        grants.push(duplicate);
        assert!(meiju_dishwasher_cycle_policy(&grants).is_err());
        for arguments in [
            json!(null),
            json!([]),
            json!({"additional":null}),
            json!({"additional":true}),
            json!({"additional":1}),
            json!({"additional":9.5}),
            json!({"additional":"9"}),
            json!({"additional":9,"program":"eco_wash"}),
        ] {
            let mut invalid = grant(id, action);
            invalid.arguments = arguments;
            assert!(meiju_dishwasher_cycle_policy(&[invalid]).is_err());
        }
    }
}
