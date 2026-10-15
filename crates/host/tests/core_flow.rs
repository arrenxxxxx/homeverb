use homeverb_core::*;
use homeverb_domain::*;
use homeverb_mock::{DEVICE_ID, MockAdapter, Scenario};
use homeverb_storage::SqliteStore;
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
};
use tokio_util::sync::CancellationToken;

struct Harness {
    core: Core,
    adapter: Arc<MockAdapter>,
    store: Arc<SqliteStore>,
    path: PathBuf,
}
impl Harness {
    async fn new(scenario: Scenario, allow: bool) -> Self {
        let path = std::env::temp_dir().join(format!("homeverb-test-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let clock = Arc::new(SystemClock);
        let adapter = Arc::new(
            MockAdapter::open(&path.join("appliance.sqlite3"), clock.clone(), scenario).unwrap(),
        );
        let grants = if allow {
            ["power", "climate"]
                .into_iter()
                .map(|capability| Grant {
                    device_id: DEVICE_ID.into(),
                    component_id: "main".into(),
                    capability: capability.into(),
                    action: "set".into(),
                    transports: vec![Transport::Lan, Transport::Cloud],
                    argument_schema: json!({"type": "object"}),
                })
                .collect()
        } else {
            vec![]
        };
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            clock,
            Policy { grants },
        );
        core.discover(&operation()).await.unwrap();
        Self {
            core,
            adapter,
            store,
            path,
        }
    }
    async fn execute(&self, command: Command) -> Execution {
        self.core.execute(command, CancellationToken::new()).await
    }
    fn sends(&self) -> usize {
        self.adapter.sends.load(Ordering::SeqCst)
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
fn operation() -> Operation {
    Operation::new(10_000, CancellationToken::new()).unwrap()
}
fn power(state: bool) -> Command {
    Command {
        schema_version: SCHEMA_VERSION.into(),
        request_id: uuid::Uuid::new_v4().to_string(),
        device_id: DEVICE_ID.into(),
        component_id: "main".into(),
        capability: "power".into(),
        action: "set".into(),
        arguments: json!({"state": state}),
        connection: Route::Auto,
        timeout_ms: 10_000,
        wait_for: WaitFor::Verified,
    }
}
fn climate(arguments: Value) -> Command {
    Command {
        capability: "climate".into(),
        arguments,
        ..power(true)
    }
}

#[tokio::test]
async fn multi_argument_actions_stay_intact_without_explicit_field_independence() {
    for idempotency in [
        Idempotency::SetValue,
        Idempotency::NonIdempotent,
        Idempotency::Unknown,
        Idempotency::ReadOnly,
    ] {
        let h = Harness::new(Scenario::Normal, true).await;
        let mut descriptor = serde_json::to_value(MockAdapter::descriptor()).unwrap();
        let action = &mut descriptor["capabilities"][1]["actions"]["set"];
        action.as_object_mut().unwrap().remove("independent_fields");
        action["idempotency"] = json!(idempotency);
        action["verification"] = json!("unavailable");
        h.store
            .save_device(&serde_json::from_value(descriptor).unwrap())
            .unwrap();
        let request = climate(json!({"mode": "heat", "target_temperature": 24}));
        let result = h.execute(request.clone()).await;
        assert_eq!(h.sends(), 1, "{idempotency:?}: {result:?}");
        assert_eq!(result.status, Status::Accepted);
        assert_eq!(result.error.as_ref().unwrap().code, "VERIFICATION_PENDING");
        assert!(result.items.is_none());
        assert!(h.core.plan(&request).unwrap().get("items").is_none());
        let state = h
            .core
            .read(DEVICE_ID, "main", Route::Auto, false, &operation())
            .await
            .unwrap();
        assert_eq!(state.observations["climate.mode"].value, "heat");
        assert_eq!(state.observations["climate.target_temperature"].value, 24);
        assert_eq!(h.execute(request).await.status, Status::Accepted);
        assert_eq!(h.sends(), 1);
    }
}

#[tokio::test]
async fn explicit_false_preserves_full_argument_state_verification() {
    let h = Harness::new(Scenario::Normal, true).await;
    let mut descriptor = MockAdapter::descriptor();
    descriptor.capabilities[1]
        .actions
        .get_mut("set")
        .unwrap()
        .independent_fields = false;
    h.store.save_device(&descriptor).unwrap();
    let request = climate(json!({"mode": "heat", "target_temperature": 24}));
    assert!(h.core.plan(&request).unwrap().get("items").is_none());
    let result = h.execute(request.clone()).await;
    assert_eq!(result.status, Status::Verified, "{result:?}");
    assert_eq!(result.data, Some(request.arguments));
    assert!(result.items.is_none());
    assert!(result.verification.is_some());
    assert_eq!(h.sends(), 1);
}

#[tokio::test]
async fn field_independence_cannot_bypass_joint_argument_requirements() {
    let h = Harness::new(Scenario::Normal, true).await;
    let mut descriptor = MockAdapter::descriptor();
    descriptor.capabilities[1]
        .actions
        .get_mut("set")
        .unwrap()
        .parameters["required"] = json!(["mode", "target_temperature"]);
    h.store.save_device(&descriptor).unwrap();
    let request = climate(json!({"mode": "heat", "target_temperature": 24}));
    assert_eq!(h.core.plan(&request).unwrap_err().code, "INVALID_ARGUMENT");
    let result = h.execute(request).await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error.unwrap().code, "INVALID_ARGUMENT");
    assert!(result.attempts.is_empty());
    assert_eq!(h.sends(), 0);
    assert!(h.store.recent_executions().unwrap().is_empty());
}

#[tokio::test]
async fn uncertain_unsplit_action_sends_all_arguments_once_without_route_replay() {
    let h = Harness::new(Scenario::LostAck, true).await;
    let mut descriptor = serde_json::to_value(MockAdapter::descriptor()).unwrap();
    let action = &mut descriptor["capabilities"][1]["actions"]["set"];
    action.as_object_mut().unwrap().remove("independent_fields");
    action["idempotency"] = json!("non_idempotent");
    h.store
        .save_device(&serde_json::from_value(descriptor).unwrap())
        .unwrap();
    let request = climate(json!({"mode": "heat", "target_temperature": 24}));
    let result = h.execute(request.clone()).await;
    assert_eq!(result.status, Status::Unknown, "{result:?}");
    assert!(result.items.is_none());
    assert_eq!(result.attempts.len(), 1);
    assert_eq!(result.attempts[0].transport, Transport::Lan);
    assert_eq!(result.attempts[0].phase, Phase::PossiblySent);
    let state = h
        .core
        .read(DEVICE_ID, "main", Route::Auto, false, &operation())
        .await
        .unwrap();
    assert_eq!(state.observations["climate.mode"].value, "heat");
    assert_eq!(state.observations["climate.target_temperature"].value, 24);
    assert_eq!(h.execute(request).await.status, Status::Unknown);
    assert_eq!(h.sends(), 1);
}

#[tokio::test]
async fn exact_receipt_lookup_is_local_despite_metadata_policy_and_lock_changes() {
    let h = Harness::new(Scenario::LostAck, true).await;
    let request = power(true);
    let result = h.execute(request.clone()).await;
    assert_eq!(result.status, Status::Unknown);
    let mut descriptor = MockAdapter::descriptor();
    descriptor.device.schema_version = "unsupported".into();
    h.store.save_device(&descriptor).unwrap();
    let readonly = Core::new(
        h.store.clone(),
        vec![],
        Arc::new(SystemClock),
        Policy::default(),
    );
    let lock = h.store.lock_device(DEVICE_ID).unwrap();
    let history = serde_json::to_value(h.store.recent_executions().unwrap()).unwrap();
    let receipt = readonly
        .execution(&request.request_id, &operation())
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(receipt).unwrap(),
        serde_json::to_value(result).unwrap()
    );
    assert_eq!(
        serde_json::to_value(h.store.recent_executions().unwrap()).unwrap(),
        history
    );
    assert_eq!(h.sends(), 1);
    assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
    assert!(h.store.snapshot(DEVICE_ID, "main").unwrap().is_none());
    drop(lock);
}

#[tokio::test]
async fn receipt_lookup_preserves_active_records_outside_the_recent_history_limit() {
    let h = Harness::new(Scenario::Normal, false).await;
    let lock = h.store.lock_device(DEVICE_ID).unwrap();
    let mut active = power(true);
    active.request_id = "  active request  ".into();
    let mut pending = Execution::new(&active, chrono::Utc::now());
    pending.status = Status::Unknown;
    pending.error = Some(Error::new(
        "WRITE_OUTCOME_UNKNOWN",
        "Pending fictional write.",
    ));
    h.store.save_execution(&active, &pending).unwrap();
    for index in 0..=homeverb_storage::RECEIPT_LIMIT {
        let mut command = power(false);
        command.request_id = format!("finished-{index}");
        let mut receipt = Execution::new(&command, chrono::Utc::now());
        receipt.error = Some(Error::new("DEVICE_REJECTED", "Fictional rejection."));
        h.store.finish_execution(&command, &receipt).unwrap();
        if index >= homeverb_storage::RECEIPT_LIMIT - 1 {
            assert!(
                !h.core
                    .recent_executions()
                    .unwrap()
                    .iter()
                    .any(|r| r.request_id == active.request_id)
            );
            let found = h
                .core
                .execution(&active.request_id, &operation())
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_value(found).unwrap(),
                serde_json::to_value(&pending).unwrap()
            );
        }
    }
    assert_eq!(
        h.core
            .execution("active request", &operation())
            .await
            .unwrap_err()
            .code,
        "REQUEST_NOT_FOUND"
    );
    assert_eq!(
        h.core
            .execution("finished-0", &operation())
            .await
            .unwrap_err()
            .code,
        "REQUEST_RETIRED"
    );
    assert_eq!(h.sends(), 0);
    assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
    drop(lock);
}

#[tokio::test]
async fn receipt_lookup_distinguishes_missing_invalid_cancelled_and_expired_requests() {
    let h = Harness::new(Scenario::Normal, false).await;
    assert_eq!(
        h.core
            .execution("missing", &operation())
            .await
            .unwrap_err()
            .code,
        "REQUEST_NOT_FOUND"
    );
    for id in ["", " \t\n"] {
        assert_eq!(
            h.core.execution(id, &operation()).await.unwrap_err().code,
            "INVALID_ARGUMENT"
        );
    }
    let cancelled = operation();
    cancelled.cancellation.cancel();
    assert_eq!(
        h.core
            .execution("missing", &cancelled)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    let mut expired = operation();
    expired.deadline = tokio::time::Instant::now() - std::time::Duration::from_secs(1);
    assert_eq!(
        h.core
            .execution("missing", &expired)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert!(h.core.recent_executions().unwrap().is_empty());
    assert_eq!(h.sends(), 0);
    assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn maximum_length_request_ids_support_composites_and_replay_protection() {
    for request_id in ["r".repeat(256), "\u{00e9}".repeat(128)] {
        let h = Harness::new(Scenario::Normal, true).await;
        let mut request = climate(json!({"mode": "heat", "target_temperature": 24}));
        request.request_id = request_id;
        request.validate().unwrap();
        let plan = h.core.plan(&request);
        assert!(
            plan.is_ok(),
            "A valid request ID must support composite planning: {plan:?}"
        );
        let result = h.execute(request.clone()).await;
        assert_eq!(result.status, Status::Verified, "{result:?}");
        let items = result.items.unwrap();
        assert_eq!(items.len(), 2);
        for (index, item) in items.iter().enumerate() {
            assert_eq!(item.request_id, format!("{}:{index}", request.request_id));
            assert_eq!(
                serde_json::to_value(
                    h.core
                        .execution(&item.request_id, &operation())
                        .await
                        .unwrap()
                )
                .unwrap(),
                serde_json::to_value(item).unwrap()
            );
            assert_eq!(
                h.store
                    .execution(&item.request_id)
                    .unwrap()
                    .unwrap()
                    .1
                    .status,
                Status::Verified
            );
        }
        assert_eq!(h.execute(request.clone()).await.status, Status::Verified);
        assert_eq!(h.sends(), 2);
        request.request_id.push('x');
        assert_eq!(
            h.execute(request).await.error.unwrap().code,
            "INVALID_ARGUMENT"
        );
        assert_eq!(h.sends(), 2);
    }
}

#[tokio::test]
async fn maximum_length_device_ids_have_cross_instance_write_locks() {
    for device_id in ["d".repeat(256), "\u{00e9}".repeat(128)] {
        let h = Harness::new(Scenario::Normal, true).await;
        let mut descriptor = MockAdapter::descriptor();
        descriptor.device.id = device_id.clone();
        h.store.save_device(&descriptor).unwrap();
        let core = Core::new(
            h.store.clone(),
            vec![h.adapter.clone()],
            Arc::new(SystemClock),
            Policy {
                grants: vec![Grant {
                    device_id: device_id.clone(),
                    component_id: "main".into(),
                    capability: "power".into(),
                    action: "set".into(),
                    transports: vec![Transport::Lan],
                    argument_schema: json!({"type": "object"}),
                }],
            },
        );
        let request = Command {
            device_id: device_id.clone(),
            ..power(true)
        };
        request.validate().unwrap();
        let result = core
            .execute(request.clone(), CancellationToken::new())
            .await;
        assert_eq!(result.status, Status::Verified, "{result:?}");
        let other = SqliteStore::open(&h.path).unwrap();
        let lock = h.store.lock_device(&device_id).unwrap();
        assert!(
            matches!(other.lock_device(&device_id), Err(error) if error.code == "RESOURCE_BUSY")
        );
        let mut different_id = device_id.clone();
        different_id.pop();
        different_id.push('x');
        drop(other.lock_device(&different_id).unwrap());
        drop(lock);
        drop(other.lock_device(&device_id).unwrap());
        assert_eq!(
            core.execute(request, CancellationToken::new()).await.status,
            Status::Verified
        );
        assert_eq!(h.sends(), 1);
    }
}

#[tokio::test]
async fn live_read_and_verified_write_match_the_public_contract() {
    let h = Harness::new(Scenario::Normal, true).await;
    let before = h
        .core
        .read("mock-ac", "main", Route::Auto, false, &operation())
        .await
        .unwrap();
    assert_eq!(before.observations["power.state"].value, false);
    let result = h.execute(power(true)).await;
    assert_eq!(result.status, Status::Verified);
    assert!(result.verification.unwrap().received_at >= result.started_at);
    assert_eq!(h.sends(), 1);
    let cached = h
        .core
        .read(DEVICE_ID, "main", Route::Auto, true, &operation())
        .await
        .unwrap();
    assert!(cached.from_cache);
    assert_eq!(cached.observations["power.state"].value, true);
}

#[tokio::test]
async fn reject_invalid_arguments_before_transmission() {
    let h = Harness::new(Scenario::Normal, true).await;
    for arguments in [
        json!({"target_temperature": 60}),
        json!({"target_temperature": 25.1}),
        json!({"mode": "invented"}),
        json!({"force": true}),
        json!({}),
        json!({"target_temperature": "25"}),
    ] {
        let result = h.execute(climate(arguments)).await;
        assert_eq!(result.status, Status::Failed);
        assert_eq!(result.error.unwrap().code, "INVALID_ARGUMENT");
    }
    assert_eq!(h.sends(), 0);
}

#[tokio::test]
async fn schema_version_timeout_and_identity_fail_before_transmission() {
    let h = Harness::new(Scenario::Normal, true).await;
    for command in [
        Command {
            schema_version: "2.0".into(),
            ..power(true)
        },
        Command {
            timeout_ms: 0,
            ..power(true)
        },
        Command {
            device_id: "mock-ac".into(),
            ..power(true)
        },
        Command {
            component_id: "freezer".into(),
            ..power(true)
        },
    ] {
        assert_eq!(h.execute(command).await.status, Status::Failed);
    }
    assert_eq!(h.sends(), 0);
}

#[tokio::test]
async fn no_write_grant_and_high_risk_are_denied() {
    let h = Harness::new(Scenario::Normal, false).await;
    assert_eq!(
        h.execute(power(true)).await.error.unwrap().code,
        "POLICY_DENIED"
    );
    let h = Harness::new(Scenario::Normal, true).await;
    let mut descriptor = MockAdapter::descriptor();
    descriptor.capabilities[0]
        .actions
        .get_mut("set")
        .unwrap()
        .risk = Risk::High;
    h.store.save_device(&descriptor).unwrap();
    assert_eq!(
        h.execute(power(true)).await.error.unwrap().code,
        "POLICY_DENIED"
    );
    assert_eq!(h.sends(), 0);
}

#[tokio::test]
async fn lost_ack_never_replays_even_when_state_actually_changed() {
    let h = Harness::new(Scenario::LostAck, true).await;
    let command = power(true);
    let result = h.execute(command.clone()).await;
    assert_eq!(result.status, Status::Unknown);
    assert!(!result.error.unwrap().retryable);
    assert_eq!(result.attempts.len(), 1);
    assert_eq!(h.execute(command).await.status, Status::Unknown);
    assert_eq!(h.sends(), 1);
    let state = h
        .core
        .read(DEVICE_ID, "main", Route::Auto, false, &operation())
        .await
        .unwrap();
    assert_eq!(state.observations["power.state"].value, true);
}

#[tokio::test]
async fn strict_lan_blocks_cloud_and_auto_reroutes_only_proven_unsent() {
    let h = Harness::new(Scenario::LanOffline, true).await;
    let result = h
        .execute(Command {
            connection: Route::Lan,
            ..power(true)
        })
        .await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(h.sends(), 0);
    let result = h.execute(power(true)).await;
    assert_eq!(result.status, Status::Verified);
    assert_eq!(result.attempts.len(), 2);
    assert_eq!(result.attempts[0].phase, Phase::NotSent);
    assert_eq!(result.attempts[1].transport, Transport::Cloud);
    assert_eq!(h.sends(), 1);
}

#[tokio::test]
async fn acceptance_and_stale_state_do_not_prove_execution() {
    for scenario in [Scenario::AcceptOnly, Scenario::StaleRead] {
        let h = Harness::new(scenario, true).await;
        let result = h.execute(power(true)).await;
        assert_eq!(result.status, Status::Accepted);
        assert!(result.verification.is_none());
        assert_eq!(h.sends(), 1);
    }
}

#[tokio::test]
async fn explicit_rejection_never_switches_routes() {
    let h = Harness::new(Scenario::Reject, true).await;
    let result = h.execute(power(true)).await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error.unwrap().code, "DEVICE_REJECTED");
    assert_eq!(result.attempts.len(), 1);
}

#[tokio::test(start_paused = true)]
async fn deadline_distinguishes_unsent_from_possible_transmission() {
    for (scenario, expected, sends) in [
        (Scenario::SlowBeforeSend, Status::Failed, 0),
        (Scenario::SlowAfterSend, Status::Unknown, 1),
    ] {
        let h = Harness::new(scenario, true).await;
        let result = h
            .execute(Command {
                timeout_ms: 10,
                ..power(true)
            })
            .await;
        assert_eq!(result.status, expected);
        assert_eq!(h.sends(), sends);
    }
}

#[tokio::test(start_paused = true)]
async fn cancellation_after_send_stops_waiting_without_rollback() {
    let h = Harness::new(Scenario::SlowAfterSend, true).await;
    let cancellation = CancellationToken::new();
    let other = cancellation.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        other.cancel();
    });
    let result = h.core.execute(power(true), cancellation).await;
    assert_eq!(result.status, Status::Unknown);
    assert_eq!(h.sends(), 1);
}

#[tokio::test]
async fn restart_preserves_pending_write_without_replay() {
    let h = Harness::new(Scenario::Normal, true).await;
    let command = power(true);
    let mut pending = Execution::new(&command, chrono::Utc::now());
    pending.status = Status::Unknown;
    pending.error = Some(Error::new(
        "WRITE_OUTCOME_UNKNOWN",
        "A previous process ended before confirmation.",
    ));
    h.store.save_execution(&command, &pending).unwrap();
    let reopened = homeverb_host::open_mock(&h.path, Scenario::Normal, true).unwrap();
    assert_eq!(
        reopened
            .execute(command, CancellationToken::new())
            .await
            .status,
        Status::Unknown
    );
    let state = reopened
        .read(DEVICE_ID, "main", Route::Auto, false, &operation())
        .await
        .unwrap();
    assert_eq!(state.observations["power.state"].value, false);
}

#[tokio::test]
async fn request_id_collision_is_rejected_and_device_lock_is_cross_instance() {
    let h = Harness::new(Scenario::Normal, true).await;
    let mut command = power(true);
    h.execute(command.clone()).await;
    command.arguments = json!({"state": false});
    assert_eq!(
        h.execute(command).await.error.unwrap().code,
        "REQUEST_CONFLICT"
    );
    let other = SqliteStore::open(&h.path).unwrap();
    let lock = h.store.lock_device(DEVICE_ID).unwrap();
    assert!(other.lock_device(DEVICE_ID).is_err());
    drop(lock);
    assert!(other.lock_device(DEVICE_ID).is_ok());
    assert_eq!(h.sends(), 1);
}

#[tokio::test]
async fn offline_cache_does_not_call_adapter_and_fresh_read_never_uses_cache() {
    let h = Harness::new(Scenario::Normal, true).await;
    h.core
        .read(DEVICE_ID, "main", Route::Auto, false, &operation())
        .await
        .unwrap();
    let reads = h.adapter.reads.load(Ordering::SeqCst);
    h.core
        .read(DEVICE_ID, "main", Route::Auto, true, &operation())
        .await
        .unwrap();
    assert_eq!(h.adapter.reads.load(Ordering::SeqCst), reads);
    let offline = homeverb_host::open_mock(&h.path, Scenario::Offline, true).unwrap();
    assert!(
        offline
            .read(DEVICE_ID, "main", Route::Auto, true, &operation())
            .await
            .unwrap()
            .from_cache
    );
    assert!(
        offline
            .read(DEVICE_ID, "main", Route::Auto, false, &operation())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn non_atomic_write_reports_each_item_without_compensation() {
    let h = Harness::new(Scenario::Partial, true).await;
    let result = h
        .execute(climate(json!({"target_temperature": 25.0, "mode": "heat"})))
        .await;
    assert_eq!(result.status, Status::Partial);
    let items = result.items.unwrap();
    assert_eq!(items[0].status, Status::Verified);
    assert_eq!(items[1].status, Status::Failed);
    assert_eq!(h.sends(), 2);
    let state = h
        .core
        .read(DEVICE_ID, "main", Route::Auto, false, &operation())
        .await
        .unwrap();
    assert_eq!(state.observations["climate.mode"].value, "heat");
    assert_eq!(state.observations["climate.target_temperature"].value, 26.0);
}

#[tokio::test]
async fn dry_run_has_no_observation_or_transmission_side_effects() {
    let h = Harness::new(Scenario::Normal, true).await;
    assert!(h.core.plan(&power(true)).is_ok());
    assert!(
        h.core
            .plan(&climate(json!({"target_temperature": 60})))
            .is_err()
    );
    assert_eq!(h.sends(), 0);
    assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
    assert!(h.core.recent_executions().unwrap().is_empty());
}

#[tokio::test]
async fn credentials_and_model_supplied_identity_fail_closed() {
    assert_eq!(
        UnavailableSecretStore
            .get(&CredentialRef::new(), &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_UNAVAILABLE"
    );
    let mut command = json!(power(true));
    command["principal"] = json!("admin");
    assert!(serde_json::from_value::<Command>(command).is_err());
}

#[tokio::test]
async fn retired_receipt_cannot_replay_an_old_write() {
    let h = Harness::new(Scenario::Normal, true).await;
    let original = power(true);
    h.execute(original.clone()).await;
    for index in 0..homeverb_storage::RECEIPT_LIMIT {
        assert_eq!(
            h.execute(power(index % 2 == 0)).await.status,
            Status::Verified
        );
    }
    let sends = h.sends();
    let result = h.execute(original).await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error.unwrap().code, "REQUEST_RETIRED");
    assert!(result.attempts.is_empty());
    assert_eq!(h.sends(), sends);
    assert_eq!(
        h.core.recent_executions().unwrap().len(),
        homeverb_storage::RECEIPT_LIMIT
    );
}

struct FailingJournal {
    inner: Arc<SqliteStore>,
    calls: std::sync::atomic::AtomicUsize,
    fail_at: usize,
    lookup_delay: std::time::Duration,
    fail_snapshot: bool,
    fail_diagnostics: bool,
}
impl FailingJournal {
    fn new(inner: Arc<SqliteStore>, fail_at: usize) -> Self {
        Self {
            inner,
            calls: 0.into(),
            fail_at,
            lookup_delay: std::time::Duration::ZERO,
            fail_snapshot: false,
            fail_diagnostics: false,
        }
    }
}
impl Store for FailingJournal {
    fn devices(&self) -> Result<Vec<Descriptor>> {
        std::thread::sleep(self.lookup_delay);
        self.inner.devices()
    }
    fn save_device(&self, value: &Descriptor) -> Result<()> {
        self.inner.save_device(value)
    }
    fn snapshot(&self, device: &str, component: &str) -> Result<Option<Snapshot>> {
        self.inner.snapshot(device, component)
    }
    fn save_snapshot(&self, value: &Snapshot) -> Result<()> {
        if self.fail_snapshot {
            return Err(Error::new(
                "STORAGE_ERROR",
                "The test cache rejected this update.",
            ));
        }
        self.inner.save_snapshot(value)
    }
    fn lock_device(&self, device: &str) -> Result<Box<dyn DeviceLock>> {
        self.inner.lock_device(device)
    }
    fn execution(&self, request: &str) -> Result<Option<(Command, Execution)>> {
        self.inner.execution(request)
    }
    fn save_execution(&self, command: &Command, result: &Execution) -> Result<()> {
        if self.calls.fetch_add(1, Ordering::SeqCst) + 1 == self.fail_at {
            return Err(Error::new(
                "STORAGE_ERROR",
                "The test journal rejected this update.",
            ));
        }
        self.inner.save_execution(command, result)
    }
    fn recent_executions(&self) -> Result<Vec<Execution>> {
        self.inner.recent_executions()
    }
    fn device_executions(&self, device_id: &str) -> Result<Vec<Execution>> {
        self.inner.device_executions(device_id)
    }
    fn finish_execution(&self, command: &Command, result: &Execution) -> Result<()> {
        if self.calls.fetch_add(1, Ordering::SeqCst) + 1 == self.fail_at {
            return Err(Error::new(
                "STORAGE_ERROR",
                "The test journal rejected this update.",
            ));
        }
        self.inner.finish_execution(command, result)
    }
    fn rooms(&self) -> Result<Vec<Room>> {
        if self.fail_diagnostics {
            return Err(Error::new("PRIVATE_ERROR_CODE", "private-backend-message"));
        }
        self.inner.rooms()
    }
    fn create_room(&self, id: &str, name: &str) -> Result<Room> {
        self.inner.create_room(id, name)
    }
    fn rename_room(&self, id: &str, name: &str) -> Result<Room> {
        self.inner.rename_room(id, name)
    }
    fn delete_room(&self, id: &str) -> Result<()> {
        self.inner.delete_room(id)
    }
    fn reorder_rooms(&self, ids: &[String]) -> Result<Vec<Room>> {
        self.inner.reorder_rooms(ids)
    }
    fn preferences(&self) -> Result<Vec<DevicePreferences>> {
        self.inner.preferences()
    }
    fn device_cards(&self) -> Result<Vec<DeviceCard>> {
        self.inner.device_cards()
    }
    fn set_preferences(&self, device: &str, input: &PreferencesInput) -> Result<DevicePreferences> {
        self.inner.set_preferences(device, input)
    }
    fn reorder_devices(&self, ids: &[String]) -> Result<Vec<DevicePreferences>> {
        self.inner.reorder_devices(ids)
    }
    fn assign_devices_to_room(&self, input: &RoomAssignment) -> Result<Vec<DevicePreferences>> {
        self.inner.assign_devices_to_room(input)
    }
}

#[tokio::test]
async fn diagnostics_projects_only_safe_fields_without_adapter_calls() {
    let h = Harness::new(Scenario::Offline, false).await;
    let private = "diagnostic-private-marker";
    let mut descriptor = MockAdapter::descriptor();
    descriptor.device.id = private.into();
    descriptor.device.alias = private.into();
    descriptor.device.brand = private.into();
    descriptor.device.model = private.into();
    descriptor.device.support_status = private.into();
    for connection in &mut descriptor.device.connections {
        connection.id = private.into();
        connection.adapter = private.into();
        connection.account_id = Some(private.into());
        connection.identity_ref = private.into();
    }
    h.store.save_device(&descriptor).unwrap();
    h.core.create_room(private).unwrap();
    let mut command = power(true);
    command.request_id = private.into();
    command.device_id = private.into();
    command.arguments = json!({"private": private});
    let mut execution = Execution::new(&command, chrono::Utc::now());
    execution.status = Status::Partial;
    execution.data = Some(json!({"private": private}));
    execution.error = Some(Error::new(private, private));
    execution.warnings.push(private.into());
    execution.verification = Some(Verification {
        kind: VerificationMethod::StateMatch,
        source: Transport::Lan,
        observed_at: None,
        received_at: chrono::Utc::now(),
        matches_target: true,
        note: private.into(),
    });
    execution.attempts.push(Attempt {
        transport: Transport::Lan,
        phase: Phase::PossiblySent,
        duration_ms: 17,
    });
    let mut child = execution.clone();
    child.status = Status::Unknown;
    execution.items = Some(vec![child]);
    h.store.finish_execution(&command, &execution).unwrap();

    let report = h.core.diagnostics(&operation()).await.unwrap();
    let json = serde_json::to_value(&report).unwrap();
    assert!(!json.to_string().contains(private));
    assert!(!json.to_string().contains(DEVICE_ID));
    assert_eq!(json["device_count"], 2);
    assert_eq!(json["room_count"], 1);
    assert_eq!(json["connections"], json!({"lan": 2, "cloud": 2}));
    assert_eq!(json["recent_receipts"][0]["status"], "partial");
    assert_eq!(json["recent_receipts"][0]["error_code"], "OTHER");
    assert_eq!(json["recent_receipts"][0]["warning_count"], 1);
    assert_eq!(json["recent_receipts"][0]["item_outcomes"]["unknown"], 1);
    assert_eq!(json["recent_receipts"][0]["attempts"][0]["duration_ms"], 17);
    assert_eq!(h.sends(), 0);
    assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
    assert_eq!(h.core.recent_executions().unwrap().len(), 1);
}

#[tokio::test]
async fn diagnostics_bounds_receipts_and_attempts_without_changing_the_journal() {
    let h = Harness::new(Scenario::Normal, false).await;
    for index in 0..105 {
        let mut command = power(true);
        command.request_id = format!("diagnostic-{index}");
        let mut execution = Execution::new(&command, chrono::Utc::now());
        execution.status = Status::Unknown;
        execution.error = Some(Error::new("WRITE_OUTCOME_UNKNOWN", "Private message"));
        execution.attempts = vec![
            Attempt {
                transport: Transport::Cloud,
                phase: Phase::PossiblySent,
                duration_ms: index
            };
            35
        ];
        h.store.save_execution(&command, &execution).unwrap();
    }
    let report = serde_json::to_value(h.core.diagnostics(&operation()).await.unwrap()).unwrap();
    let receipts = report["recent_receipts"].as_array().unwrap();
    assert_eq!(receipts.len(), 100);
    assert_eq!(receipts[0]["error_code"], "WRITE_OUTCOME_UNKNOWN");
    assert_eq!(receipts[0]["attempts"][0]["duration_ms"], 104);
    assert_eq!(receipts[0]["attempts"].as_array().unwrap().len(), 32);
    assert_eq!(receipts[0]["attempts_omitted"], 3);
    assert_eq!(
        h.store
            .execution("diagnostic-0")
            .unwrap()
            .unwrap()
            .1
            .attempts
            .len(),
        35
    );
    assert_eq!(h.sends(), 0);
}

#[tokio::test]
async fn diagnostics_sanitizes_storage_failures_and_obeys_the_shared_budget() {
    let h = Harness::new(Scenario::Normal, false).await;
    let mut store = FailingJournal::new(h.store.clone(), usize::MAX);
    store.fail_diagnostics = true;
    let core = Core::new(
        Arc::new(store),
        vec![h.adapter.clone()],
        Arc::new(SystemClock),
        Policy::default(),
    );
    let error = core.diagnostics(&operation()).await.unwrap_err();
    assert_eq!(error.code, "STORAGE_ERROR");
    assert!(!serde_json::to_string(&error).unwrap().contains("private"));
    let cancelled = operation();
    cancelled.cancellation.cancel();
    assert_eq!(
        core.diagnostics(&cancelled).await.unwrap_err().code,
        "CANCELLED"
    );
    let mut store = FailingJournal::new(h.store.clone(), usize::MAX);
    store.lookup_delay = std::time::Duration::from_millis(10);
    store.fail_diagnostics = true;
    let core = Core::new(
        Arc::new(store),
        vec![],
        Arc::new(SystemClock),
        Policy::default(),
    );
    let short = Operation::new(1, CancellationToken::new()).unwrap();
    assert_eq!(
        core.diagnostics(&short).await.unwrap_err().code,
        "DEADLINE_EXCEEDED"
    );
}

#[tokio::test]
async fn journal_failures_preserve_the_strongest_known_physical_outcome() {
    for (scenario, fail_at, expected, sends) in [
        (Scenario::Normal, 1, Status::Failed, 0),
        (Scenario::Normal, 2, Status::Accepted, 1),
        (Scenario::Normal, 3, Status::Verified, 1),
        (Scenario::LostAck, 2, Status::Unknown, 1),
        (Scenario::CorrelatedCompletion, 1, Status::Failed, 0),
        (Scenario::CorrelatedCompletion, 2, Status::Verified, 1),
        (Scenario::CorrelatedCompletion, 3, Status::Verified, 1),
    ] {
        let h = Harness::new(scenario, true).await;
        let store = Arc::new(FailingJournal::new(h.store.clone(), fail_at));
        let core = Core::new(
            store,
            vec![h.adapter.clone()],
            Arc::new(SystemClock),
            Policy {
                grants: vec![Grant {
                    device_id: DEVICE_ID.into(),
                    component_id: "main".into(),
                    capability: "power".into(),
                    action: "set".into(),
                    transports: vec![Transport::Lan, Transport::Cloud],
                    argument_schema: json!({"type": "object"}),
                }],
            },
        );
        let result = core.execute(power(true), CancellationToken::new()).await;
        assert_eq!(result.status, expected);
        assert_eq!(h.sends(), sends);
        if expected == Status::Verified {
            assert!(result.error.is_none());
            assert_eq!(result.warnings.len(), 1);
        }
        if expected == Status::Unknown {
            assert_eq!(result.error.unwrap().code, "WRITE_OUTCOME_UNKNOWN");
            assert_eq!(result.warnings.len(), 1);
        }
    }
}

fn fault_core(h: &Harness, store: FailingJournal, capability: &str) -> Core {
    Core::new(
        Arc::new(store),
        vec![h.adapter.clone()],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: DEVICE_ID.into(),
                component_id: "main".into(),
                capability: capability.into(),
                action: "set".into(),
                transports: vec![Transport::Lan, Transport::Cloud],
                argument_schema: json!({"type": "object"}),
            }],
        },
    )
}

#[tokio::test]
async fn validation_time_counts_toward_the_total_write_budget() {
    let h = Harness::new(Scenario::Normal, true).await;
    let mut store = FailingJournal::new(h.store.clone(), 0);
    store.lookup_delay = std::time::Duration::from_millis(50);
    let core = fault_core(&h, store, "power");
    let result = core
        .execute(
            Command {
                timeout_ms: 5,
                ..power(true)
            },
            CancellationToken::new(),
        )
        .await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error.unwrap().code, "DEADLINE_EXCEEDED");
    assert_eq!(h.sends(), 0);
}

#[tokio::test(start_paused = true)]
async fn host_deadline_and_cancellation_are_not_reset_by_core() {
    let h = Harness::new(Scenario::Normal, true).await;
    let operation = Operation::new(10, CancellationToken::new()).unwrap();
    tokio::time::advance(std::time::Duration::from_millis(11)).await;
    let result = h.core.execute_with_operation(power(true), &operation).await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error.unwrap().code, "DEADLINE_EXCEEDED");
    assert_eq!(h.sends(), 0);

    let operation = Operation::new(10_000, CancellationToken::new()).unwrap();
    operation.cancellation.cancel();
    let result = h.core.execute_with_operation(power(true), &operation).await;
    assert_eq!(result.error.unwrap().code, "CANCELLED");
    assert_eq!(h.sends(), 0);

    let h = Harness::new(Scenario::SlowBeforeSend, true).await;
    let operation = Operation::new(10_000, CancellationToken::new()).unwrap();
    let start = tokio::time::Instant::now();
    let result = h
        .core
        .execute_with_operation(
            Command {
                timeout_ms: 10,
                ..power(true)
            },
            &operation,
        )
        .await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error.unwrap().code, "DEADLINE_EXCEEDED");
    assert!(start.elapsed() < std::time::Duration::from_millis(100));
    assert_eq!(h.sends(), 0);
}

#[tokio::test]
async fn composite_journal_failures_keep_items_and_stop_further_writes() {
    for (scenario, fail_at, first, second, aggregate, sends) in [
        (
            Scenario::CorrelatedCompletion,
            3,
            Status::Verified,
            Status::Failed,
            Status::Partial,
            1,
        ),
        (
            Scenario::Normal,
            2,
            Status::Failed,
            Status::Failed,
            Status::Failed,
            0,
        ),
        (
            Scenario::Normal,
            3,
            Status::Accepted,
            Status::Failed,
            Status::Partial,
            1,
        ),
        (
            Scenario::Normal,
            4,
            Status::Verified,
            Status::Failed,
            Status::Partial,
            1,
        ),
        (
            Scenario::LostAck,
            3,
            Status::Unknown,
            Status::Failed,
            Status::Partial,
            1,
        ),
        (
            Scenario::Normal,
            8,
            Status::Verified,
            Status::Verified,
            Status::Verified,
            2,
        ),
        (
            Scenario::Partial,
            7,
            Status::Verified,
            Status::Failed,
            Status::Partial,
            2,
        ),
    ] {
        let h = Harness::new(scenario, true).await;
        let core = fault_core(&h, FailingJournal::new(h.store.clone(), fail_at), "climate");
        let command = climate(json!({"mode": "heat", "target_temperature": 25.0}));
        let result = core
            .execute(command.clone(), CancellationToken::new())
            .await;
        assert_eq!(result.status, aggregate, "failure at {fail_at}");
        let items = result
            .items
            .as_ref()
            .expect("Every composite item must be retained");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].status, first);
        assert_eq!(items[1].status, second);
        if sends < 2 {
            assert_eq!(items[1].error.as_ref().unwrap().code, "NOT_EXECUTED");
            assert!(items[1].attempts.is_empty());
        }
        for item in items {
            if item.status == Status::Verified {
                assert!(item.verification.is_some());
                assert!(item.error.is_none());
            }
        }
        if first == Status::Unknown {
            assert_eq!(
                items[0].error.as_ref().unwrap().code,
                "WRITE_OUTCOME_UNKNOWN"
            );
        }
        if fail_at >= 7 {
            assert_eq!(result.warnings.len(), 1);
        } else if matches!(first, Status::Verified | Status::Unknown) {
            assert_eq!(items[0].warnings.len(), 1);
        }
        if aggregate == Status::Partial {
            assert_eq!(result.error.as_ref().unwrap().code, "PARTIAL_EXECUTION");
        }
        core.execute(command, CancellationToken::new()).await;
        assert_eq!(h.sends(), sends, "Reusing a request must never send again");
    }
}

#[tokio::test]
async fn failed_observation_storage_does_not_erase_verification() {
    for command in [
        power(true),
        climate(json!({"mode": "heat", "target_temperature": 25.0})),
    ] {
        let h = Harness::new(Scenario::Normal, true).await;
        let mut store = FailingJournal::new(h.store.clone(), 0);
        store.fail_snapshot = true;
        let core = fault_core(&h, store, &command.capability);
        let result = core.execute(command, CancellationToken::new()).await;
        let verified = if let Some(items) = &result.items {
            assert_eq!(result.status, Status::Partial);
            assert_eq!(items[1].error.as_ref().unwrap().code, "NOT_EXECUTED");
            &items[0]
        } else {
            &result
        };
        assert_eq!(verified.status, Status::Verified);
        assert!(verified.error.is_none());
        assert!(verified.verification.is_some());
        assert_eq!(verified.warnings.len(), 1);
        assert_eq!(h.sends(), 1);
    }
}

#[tokio::test]
async fn final_journal_failure_preserves_rejection_and_skipped_item_reasons() {
    let h = Harness::new(Scenario::Reject, true).await;
    let core = fault_core(&h, FailingJournal::new(h.store.clone(), 2), "power");
    let result = core.execute(power(true), CancellationToken::new()).await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error.unwrap().code, "DEVICE_REJECTED");
    assert_eq!(result.warnings.len(), 1);

    let h = Harness::new(Scenario::LostAck, true).await;
    let core = fault_core(&h, FailingJournal::new(h.store.clone(), 4), "climate");
    let result = core
        .execute(
            climate(json!({"mode": "heat", "target_temperature": 25.0})),
            CancellationToken::new(),
        )
        .await;
    assert_eq!(result.status, Status::Partial);
    let items = result.items.unwrap();
    assert_eq!(items[0].status, Status::Unknown);
    assert_eq!(items[1].error.as_ref().unwrap().code, "NOT_EXECUTED");
    assert!(items[1].attempts.is_empty());
    assert_eq!(items[1].warnings.len(), 1);
    assert_eq!(h.sends(), 1);
}

#[tokio::test]
async fn successful_non_atomic_write_keeps_all_receipts_and_does_not_turn_power_on() {
    let h = Harness::new(Scenario::Normal, true).await;
    let command = climate(json!({"mode": "heat", "target_temperature": 25.0}));
    let result = h.execute(command.clone()).await;
    assert_eq!(result.status, Status::Verified);
    let items = result.items.unwrap();
    assert!(
        items
            .iter()
            .all(|item| item.status == Status::Verified && item.verification.is_some())
    );
    assert_eq!(items.len(), 2);
    assert_eq!(h.execute(command).await.status, Status::Verified);
    assert_eq!(h.sends(), 2);
    let state = h
        .core
        .read(DEVICE_ID, "main", Route::Auto, false, &operation())
        .await
        .unwrap();
    assert_eq!(state.observations["power.state"].value, false);
}

#[tokio::test]
async fn non_idempotent_uncertain_write_and_pre_cancelled_write_are_never_replayed() {
    let h = Harness::new(Scenario::LostAck, true).await;
    let mut descriptor = MockAdapter::descriptor();
    descriptor.capabilities[0]
        .actions
        .get_mut("set")
        .unwrap()
        .idempotency = Idempotency::NonIdempotent;
    h.store.save_device(&descriptor).unwrap();
    let result = h.execute(power(true)).await;
    assert_eq!(result.status, Status::Unknown);
    assert_eq!(result.attempts.len(), 1);
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let result = h.core.execute(power(false), cancellation).await;
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error.unwrap().code, "CANCELLED");
    assert_eq!(h.sends(), 1);
}
