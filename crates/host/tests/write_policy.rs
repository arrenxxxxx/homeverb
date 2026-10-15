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
