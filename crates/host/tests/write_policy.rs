use homeverb_core::{Core, Grant, Operation, Policy, Store, SystemClock};
use homeverb_domain::*;
use homeverb_mock::{DEVICE_ID, MockAdapter, Scenario};
use homeverb_storage::SqliteStore;
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
};
use tokio_util::sync::CancellationToken;

struct Fixture {
    core: Core,
    adapter: Arc<MockAdapter>,
    store: Arc<SqliteStore>,
    directory: PathBuf,
}
impl Fixture {
    async fn new(grants: Vec<Grant>, scenario: Scenario) -> Self {
        let directory =
            std::env::temp_dir().join(format!("homeverb-policy-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&directory).unwrap());
        let clock = Arc::new(SystemClock);
        let adapter = Arc::new(
            MockAdapter::open(
                &directory.join("appliance.sqlite3"),
                clock.clone(),
                scenario,
            )
            .unwrap(),
        );
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            clock,
            Policy { grants },
        );
        core.discover(&Operation::new(10_000, CancellationToken::new()).unwrap())
            .await
            .unwrap();
        Self {
            core,
            adapter,
            store,
            directory,
        }
    }
    async fn execute(&self, command: Command) -> Execution {
        self.core.execute(command, CancellationToken::new()).await
    }
    fn sends(&self) -> usize {
        self.adapter.sends.load(Ordering::SeqCst)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

fn grant(transports: Vec<Transport>, argument_schema: Value) -> Grant {
    Grant {
        device_id: DEVICE_ID.into(),
        component_id: "main".into(),
        capability: "climate".into(),
        action: "set".into(),
        transports,
        argument_schema,
    }
}
fn command(arguments: Value) -> Command {
    Command {
        schema_version: SCHEMA_VERSION.into(),
        request_id: uuid::Uuid::new_v4().to_string(),
        device_id: DEVICE_ID.into(),
        component_id: "main".into(),
        capability: "climate".into(),
        action: "set".into(),
        arguments,
        connection: Route::Auto,
        timeout_ms: 10_000,
        wait_for: WaitFor::Verified,
    }
}

#[tokio::test]
async fn argument_bounds_and_component_identity_are_enforced_before_sending() {
    let h = Fixture::new(vec![grant(vec![Transport::Lan], json!({
        "type": "object", "properties": {"target_temperature": {"type": "number", "minimum": 20, "maximum": 24}},
        "required": ["target_temperature"], "additionalProperties": false
    }))], Scenario::Normal).await;
    let mut descriptor = MockAdapter::descriptor();
    descriptor.device.components.push(Component {
        id: "other".into(),
        capabilities: vec!["climate".into()],
    });
    h.store.save_device(&descriptor).unwrap();
    for denied in [
        command(json!({"target_temperature": 25})),
        command(json!({"target_temperature": 19})),
        command(json!({"mode": "heat"})),
        Command {
            component_id: "other".into(),
            ..command(json!({"target_temperature": 22}))
        },
    ] {
        assert_eq!(h.core.plan(&denied).unwrap_err().code, "POLICY_DENIED");
        let result = h.execute(denied).await;
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
        assert!(result.attempts.is_empty());
    }
    assert_eq!(h.sends(), 0);
    assert!(h.core.recent_executions().unwrap().is_empty());
    assert_eq!(
        h.execute(command(json!({"target_temperature": 23})))
            .await
            .status,
        Status::Verified
    );
    assert_eq!(h.sends(), 1);
}

#[tokio::test]
async fn route_authority_limits_auto_selection_and_explicit_routes() {
    let h = Fixture::new(
        vec![grant(vec![Transport::Cloud], json!(true))],
        Scenario::Normal,
    )
    .await;
    let result = h
        .execute(Command {
            connection: Route::Lan,
            ..command(json!({"target_temperature": 23}))
        })
        .await;
    assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
    assert!(result.attempts.is_empty());
    let plan = h
        .core
        .plan(&command(json!({"target_temperature": 23})))
        .unwrap();
    assert_eq!(plan["transports"], json!(["cloud"]));
    let result = h.execute(command(json!({"target_temperature": 23}))).await;
    assert_eq!(result.status, Status::Verified);
    assert_eq!(result.attempts.len(), 1);
    assert_eq!(result.attempts[0].transport, Transport::Cloud);
}

#[tokio::test]
async fn fallback_never_uses_a_route_whose_grant_does_not_match_the_whole_command() {
    let h = Fixture::new(
        vec![
            grant(
                vec![Transport::Lan],
                json!({"const": {"target_temperature": 23}}),
            ),
            grant(
                vec![Transport::Cloud],
                json!({"const": {"target_temperature": 24}}),
            ),
        ],
        Scenario::LanOffline,
    )
    .await;
    let result = h.execute(command(json!({"target_temperature": 23}))).await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error.unwrap().code, "LAN_UNREACHABLE");
    assert_eq!(result.attempts.len(), 1);
    assert_eq!(result.attempts[0].transport, Transport::Lan);
    assert_eq!(h.sends(), 0);
    let result = h.execute(command(json!({"target_temperature": 24}))).await;
    assert_eq!(result.status, Status::Verified);
    assert_eq!(result.attempts[0].transport, Transport::Cloud);
    assert_eq!(h.sends(), 1);
}

#[tokio::test]
async fn grants_cannot_be_spliced_to_authorize_a_composite_command() {
    let h = Fixture::new(
        vec![
            grant(vec![Transport::Lan], json!({"const": {"mode": "heat"}})),
            grant(
                vec![Transport::Lan],
                json!({"const": {"target_temperature": 24}}),
            ),
        ],
        Scenario::Normal,
    )
    .await;
    let result = h
        .execute(command(json!({"mode": "heat", "target_temperature": 24})))
        .await;
    assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
    assert_eq!(h.sends(), 0);
    assert!(h.core.recent_executions().unwrap().is_empty());
}

#[tokio::test]
async fn malformed_or_external_grant_schemas_and_empty_routes_confer_no_authority() {
    for (transports, schema) in [
        (vec![Transport::Lan], json!({"type": "unsupported"})),
        (
            vec![Transport::Lan],
            json!({"$ref": "https://example.invalid/authorization.json"}),
        ),
        (vec![Transport::Lan], json!(false)),
        (vec![], json!(true)),
    ] {
        let h = Fixture::new(vec![grant(transports, schema)], Scenario::Normal).await;
        let result = h.execute(command(json!({"target_temperature": 23}))).await;
        assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
        assert_eq!(h.sends(), 0);
    }
}

#[tokio::test]
async fn action_routes_and_risk_still_restrict_an_explicit_grant() {
    for risk in [Risk::Low, Risk::Elevated, Risk::High] {
        let h = Fixture::new(
            vec![grant(vec![Transport::Cloud], json!(true))],
            Scenario::Normal,
        )
        .await;
        let mut descriptor = MockAdapter::descriptor();
        let action = descriptor
            .capabilities
            .iter_mut()
            .find(|capability| capability.id == "climate")
            .unwrap()
            .actions
            .get_mut("set")
            .unwrap();
        action.risk = risk;
        if risk == Risk::Low {
            action.transports = vec![Transport::Lan];
        }
        h.store.save_device(&descriptor).unwrap();
        let result = h.execute(command(json!({"target_temperature": 23}))).await;
        assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
        assert_eq!(h.sends(), 0);
    }
}

#[tokio::test]
async fn composite_parts_cannot_expand_the_parent_grants_or_ignore_their_own_routes() {
    for parent_transports in [
        vec![Transport::Cloud],
        vec![Transport::Lan, Transport::Cloud],
    ] {
        let h = Fixture::new(
            vec![
                grant(
                    parent_transports.clone(),
                    json!({"type": "object", "minProperties": 2}),
                ),
                grant(
                    vec![Transport::Lan],
                    json!({"type": "object", "maxProperties": 1}),
                ),
            ],
            Scenario::LanOffline,
        )
        .await;
        let command = command(json!({"mode": "heat", "target_temperature": 23}));
        let result = h.execute(command.clone()).await;
        if parent_transports.len() == 1 {
            assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
            assert!(result.attempts.is_empty());
            assert_eq!(h.core.plan(&command).unwrap_err().code, "POLICY_DENIED");
        } else {
            assert_eq!(result.status, Status::Failed);
            let items = result.items.unwrap();
            assert_eq!(items[0].attempts.len(), 1);
            assert_eq!(items[0].attempts[0].transport, Transport::Lan);
            assert_eq!(items[1].error.as_ref().unwrap().code, "NOT_EXECUTED");
            let plan = h.core.plan(&command).unwrap();
            for item in plan["items"].as_array().unwrap() {
                assert_eq!(item["transports"], json!(["lan"]));
            }
        }
        assert_eq!(h.sends(), 0);
    }
}

#[tokio::test]
async fn matching_grants_allow_bounded_fallback_and_valid_composites() {
    let h = Fixture::new(
        vec![
            grant(vec![Transport::Lan], json!({"type": "object"})),
            grant(vec![Transport::Cloud], json!({"type": "object"})),
        ],
        Scenario::LanOffline,
    )
    .await;
    let result = h
        .execute(command(json!({"mode": "heat", "target_temperature": 23})))
        .await;
    assert_eq!(result.status, Status::Verified);
    for item in result.items.unwrap() {
        assert_eq!(item.attempts.len(), 2);
        assert_eq!(item.attempts[0].phase, Phase::NotSent);
        assert_eq!(item.attempts[1].transport, Transport::Cloud);
    }
    assert_eq!(h.sends(), 2);
}

#[tokio::test]
async fn session_grant_inspection_keeps_exact_scope_and_whole_argument_rules() {
    let lan = grant(
        vec![Transport::Lan],
        json!({"const":{"mode":"cool","target_temperature":22}}),
    );
    let cloud = grant(
        vec![Transport::Cloud],
        json!({"const":{"mode":"heat","target_temperature":24}}),
    );
    let mut other_device = cloud.clone();
    other_device.device_id = "other-device".into();
    let mut other_component = cloud.clone();
    other_component.component_id = "other-component".into();
    let mut other_action = cloud.clone();
    other_action.action = "other-action".into();
    let invalid = grant(
        vec![Transport::Cloud],
        json!({"type":"invalid-json-schema-type"}),
    );
    let h = Fixture::new(
        vec![
            lan,
            cloud.clone(),
            other_device,
            other_component,
            other_action,
            invalid,
        ],
        Scenario::Normal,
    )
    .await;
    let before = json!(h.store.devices().unwrap());
    let op = Operation::new(1000, CancellationToken::new()).unwrap();
    let local = h
        .core
        .inspect_session_grants(DEVICE_ID, "conn-example-lan", &op)
        .await
        .unwrap();
    assert_eq!(local["transport"], "lan");
    assert_eq!(
        local["grants"],
        json!([{"component_id":"main","capability":"climate","action":"set","argument_schema":{"const":{"mode":"cool","target_temperature":22}}}])
    );
    let remote = h
        .core
        .inspect_session_grants(DEVICE_ID, "conn-example-cloud", &op)
        .await
        .unwrap();
    assert_eq!(remote["source"], "session_policy");
    assert_eq!(remote["provider_authorization"], "not_checked");
    assert_eq!(remote["grants"].as_array().unwrap().len(), 3);
    assert_eq!(
        remote["grants"][0]["argument_schema"],
        cloud.argument_schema
    );
    assert_eq!(remote["grants"][1]["component_id"], "other-component");
    assert_eq!(remote["grants"][2]["action"], "other-action");
    let denied = command(json!({"mode":"heat","target_temperature":22}));
    assert_eq!(h.core.plan(&denied).unwrap_err().code, "POLICY_DENIED");
    let mut edited = remote.clone();
    edited["grants"][0]["argument_schema"] = json!(true);
    assert_eq!(
        h.core
            .inspect_session_grants(DEVICE_ID, "conn-example-cloud", &op)
            .await
            .unwrap(),
        remote
    );
    assert_eq!(json!(h.store.devices().unwrap()), before);
    assert!(h.core.recent_executions().unwrap().is_empty());
    assert_eq!(h.sends(), 0);
}

#[tokio::test]
async fn inspected_rules_do_not_supply_risk_approval_or_action_support() {
    for risk in [Risk::Elevated, Risk::High] {
        let h = Fixture::new(
            vec![grant(vec![Transport::Cloud], json!(true))],
            Scenario::Normal,
        )
        .await;
        let mut descriptor = MockAdapter::descriptor();
        let action = descriptor
            .capabilities
            .iter_mut()
            .find(|c| c.id == "climate")
            .unwrap()
            .actions
            .get_mut("set")
            .unwrap();
        action.risk = risk;
        action.independent_fields = false;
        h.store.save_device(&descriptor).unwrap();
        let op = Operation::new(1000, CancellationToken::new()).unwrap();
        let before = h
            .core
            .inspect_session_grants(DEVICE_ID, "conn-example-cloud", &op)
            .await
            .unwrap();
        assert_eq!(before["grants"][0]["argument_schema"], json!(true));
        assert_eq!(
            h.core
                .plan(&command(json!({"target_temperature":24})))
                .unwrap_err()
                .code,
            "POLICY_DENIED"
        );
        descriptor
            .capabilities
            .iter_mut()
            .find(|c| c.id == "climate")
            .unwrap()
            .actions
            .clear();
        h.store.save_device(&descriptor).unwrap();
        assert_eq!(
            h.core
                .inspect_session_grants(DEVICE_ID, "conn-example-cloud", &op)
                .await
                .unwrap(),
            before
        );
        assert_eq!(
            h.core
                .plan(&command(json!({"target_temperature":24})))
                .unwrap_err()
                .code,
            "CAPABILITY_UNSUPPORTED"
        );
        assert_eq!(h.sends(), 0);
    }
}

#[tokio::test(start_paused = true)]
async fn grant_inspection_refuses_aliases_unknown_connections_and_expired_operations() {
    let h = Fixture::new(vec![], Scenario::Normal).await;
    let op = Operation::new(1000, CancellationToken::new()).unwrap();
    for (device, connection, code) in [
        ("mock-ac", "conn-example-cloud", "INVALID_ARGUMENT"),
        (DEVICE_ID, "missing", "INVALID_ARGUMENT"),
        ("missing", "conn-example-cloud", "DEVICE_NOT_FOUND"),
    ] {
        assert_eq!(
            h.core
                .inspect_session_grants(device, connection, &op)
                .await
                .unwrap_err()
                .code,
            code
        );
    }
    let empty = h
        .core
        .inspect_session_grants(DEVICE_ID, "conn-example-cloud", &op)
        .await
        .unwrap();
    assert_eq!(empty["grants"], json!([]));
    op.cancellation.cancel();
    assert_eq!(
        h.core
            .inspect_session_grants(DEVICE_ID, "conn-example-cloud", &op)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let expired = Operation::new(1, CancellationToken::new()).unwrap();
    tokio::time::advance(std::time::Duration::from_millis(1)).await;
    assert_eq!(
        h.core
            .inspect_session_grants(DEVICE_ID, "conn-example-cloud", &expired)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(h.sends(), 0);
}
