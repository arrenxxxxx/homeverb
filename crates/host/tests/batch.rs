use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::*;
use homeverb_mock::MockAdapter;
use homeverb_storage::SqliteStore;
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy)]
enum Behavior {
    Complete,
    Accept,
    Unknown,
    Reject,
    Slow,
    Cancel,
}

struct BatchAdapter {
    behaviors: Vec<Behavior>,
    sends: AtomicUsize,
    active: AtomicUsize,
    maximum: AtomicUsize,
}

struct Active<'a>(&'a AtomicUsize);
impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[async_trait]
impl Adapter for BatchAdapter {
    fn id(&self) -> &str {
        "homeverb.mock"
    }

    async fn discover(&self, _operation: &Operation) -> Result<Vec<Descriptor>> {
        Ok(self
            .behaviors
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let mut descriptor = MockAdapter::descriptor();
                descriptor.device.id = format!("dev-batch-{index}");
                descriptor.device.alias = format!("batch-{index}");
                for capability in &mut descriptor.capabilities {
                    for action in capability.actions.values_mut() {
                        action.verification = VerificationMethod::CorrelatedCompletion;
                    }
                }
                descriptor
            })
            .collect())
    }

    async fn read(
        &self,
        _device: &Device,
        _component: &str,
        _connection: &Connection,
        _operation: &Operation,
    ) -> Result<Snapshot> {
        panic!("Completion batches must not read state");
    }

    async fn execute(
        &self,
        validated: &ValidatedCommand,
        connection: &Connection,
        progress: WriteProgress,
        operation: &Operation,
    ) -> Result<AdapterOutcome> {
        let command = validated.command();
        let index: usize = command
            .device_id
            .strip_prefix("dev-batch-")
            .unwrap()
            .parse()
            .unwrap();
        self.sends.fetch_add(1, Ordering::SeqCst);
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.maximum.fetch_max(active, Ordering::SeqCst);
        let _active = Active(&self.active);
        progress.mark_sent();
        if matches!(self.behaviors[index], Behavior::Cancel) {
            operation.cancellation.cancel();
        }
        tokio::time::sleep(Duration::from_millis(
            if matches!(self.behaviors[index], Behavior::Slow | Behavior::Cancel) {
                60_000
            } else {
                4 - index as u64 % 4
            },
        ))
        .await;
        let phase = match self.behaviors[index] {
            Behavior::Unknown => Phase::PossiblySent,
            Behavior::Reject => Phase::Rejected,
            _ => Phase::Accepted,
        };
        if phase == Phase::Accepted {
            progress.mark_accepted();
        }
        Ok(AdapterOutcome {
            phase,
            error: None,
            completion: matches!(self.behaviors[index], Behavior::Complete).then(|| {
                CorrelatedCompletion {
                    request_id: command.request_id.clone(),
                    device_id: command.device_id.clone(),
                    component_id: command.component_id.clone(),
                    connection_id: connection.id.clone(),
                    completed_at: None,
                    received_at: chrono::Utc::now(),
                }
            }),
        })
    }
}

struct Harness {
    core: Core,
    adapter: Arc<BatchAdapter>,
    store: Arc<SqliteStore>,
    path: PathBuf,
}
impl Harness {
    async fn new(behaviors: Vec<Behavior>) -> Self {
        let path = std::env::temp_dir().join(format!("homeverb-batch-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let adapter = Arc::new(BatchAdapter {
            behaviors,
            sends: AtomicUsize::new(0),
            active: AtomicUsize::new(0),
            maximum: AtomicUsize::new(0),
        });
        let grants = (0..adapter.behaviors.len())
            .flat_map(|index| {
                ["power", "climate"].map(|capability| Grant {
                    device_id: format!("dev-batch-{index}"),
                    component_id: "main".into(),
                    capability: capability.into(),
                    action: "set".into(),
                    transports: vec![Transport::Lan, Transport::Cloud],
                    argument_schema: json!({"type":"object"}),
                })
            })
            .collect();
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            Arc::new(SystemClock),
            Policy { grants },
        );
        core.discover(&operation(10_000)).await.unwrap();
        Self {
            core,
            adapter,
            store,
            path,
        }
    }

    fn batch(&self) -> BatchCommand {
        BatchCommand {
            schema_version: SCHEMA_VERSION.into(),
            commands: (0..self.adapter.behaviors.len())
                .map(|index| Command {
                    schema_version: SCHEMA_VERSION.into(),
                    request_id: format!("request-{index}"),
                    device_id: format!("dev-batch-{index}"),
                    component_id: "main".into(),
                    capability: "power".into(),
                    action: "set".into(),
                    arguments: json!({"state":true}),
                    connection: Route::Auto,
                    timeout_ms: 10_000,
                    wait_for: WaitFor::Verified,
                })
                .collect(),
        }
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
fn operation(timeout: u64) -> Operation {
    Operation::new(timeout, CancellationToken::new()).unwrap()
}

#[tokio::test(start_paused = true)]
async fn batch_bounds_concurrency_preserves_input_order_and_replays_each_saved_command() {
    let h = Harness::new(vec![Behavior::Complete; 9]).await;
    let batch = h.batch();
    let plan = h.core.plan_batch(&batch, &operation(10_000)).await.unwrap();
    assert_eq!(plan["items"].as_array().unwrap().len(), 9);
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 0);
    assert!(h.store.recent_executions().unwrap().is_empty());
    let result = h
        .core
        .execute_batch(batch.clone(), &operation(10_000))
        .await
        .unwrap();
    assert_eq!(result.status, Status::Verified);
    assert_eq!(h.adapter.maximum.load(Ordering::SeqCst), 4);
    assert_eq!(h.adapter.active.load(Ordering::SeqCst), 0);
    for (command, item) in batch.commands.iter().zip(&result.items) {
        assert_eq!(item.request_id, command.request_id);
        assert_eq!(item.device_id, command.device_id);
        assert_eq!(item.status, Status::Verified);
        assert!(item.verification.is_some());
    }
    let replay = h
        .core
        .execute_batch(batch, &operation(10_000))
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(replay.items).unwrap(),
        serde_json::to_value(result.items).unwrap()
    );
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 9);
}

#[tokio::test(start_paused = true)]
async fn batch_keeps_mixed_outcomes_and_continues_independent_devices_after_failure() {
    let h = Harness::new(vec![
        Behavior::Reject,
        Behavior::Accept,
        Behavior::Unknown,
        Behavior::Complete,
        Behavior::Complete,
    ])
    .await;
    let result = h
        .core
        .execute_batch(h.batch(), &operation(10_000))
        .await
        .unwrap();
    assert_eq!(result.status, Status::Partial);
    assert_eq!(
        result
            .items
            .iter()
            .map(|item| item.status)
            .collect::<Vec<_>>(),
        vec![
            Status::Failed,
            Status::Accepted,
            Status::Unknown,
            Status::Verified,
            Status::Verified
        ]
    );
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 5);
    assert!(result.items.iter().all(|item| item.attempts.len() == 1));
    for (behavior, expected) in [
        (Behavior::Accept, Status::Accepted),
        (Behavior::Unknown, Status::Unknown),
        (Behavior::Reject, Status::Failed),
    ] {
        let h = Harness::new(vec![behavior; 2]).await;
        assert_eq!(
            h.core
                .execute_batch(h.batch(), &operation(10_000))
                .await
                .unwrap()
                .status,
            expected
        );
    }
}

#[tokio::test(start_paused = true)]
async fn batch_preflights_every_item_and_composite_id_before_any_send() {
    let h = Harness::new(vec![Behavior::Complete; 2]).await;
    let mut cases = Vec::new();
    let mut batch = h.batch();
    batch.commands[1].arguments = json!({"state":"on"});
    cases.push(batch);
    let mut batch = h.batch();
    batch.commands[1].component_id = "other".into();
    cases.push(batch);
    let mut batch = h.batch();
    batch.commands[1].device_id = "missing".into();
    cases.push(batch);
    let mut batch = h.batch();
    batch.commands[1].device_id = batch.commands[0].device_id.clone();
    cases.push(batch);
    let mut batch = h.batch();
    batch.commands[1].request_id = batch.commands[0].request_id.clone();
    cases.push(batch);
    let mut batch = h.batch();
    batch.commands.clear();
    cases.push(batch);
    let mut batch = h.batch();
    batch.schema_version = "9.0".into();
    cases.push(batch);
    let mut batch = h.batch();
    batch.commands = vec![batch.commands[0].clone(); 33];
    cases.push(batch);
    let mut batch = h.batch();
    batch.commands[0].capability = "climate".into();
    batch.commands[0].arguments = json!({"mode":"heat","target_temperature":24});
    batch.commands[1].request_id = format!("{}:0", batch.commands[0].request_id);
    cases.push(batch);
    for batch in cases {
        assert!(
            h.core
                .execute_batch(batch, &operation(10_000))
                .await
                .is_err()
        );
        assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 0);
        assert!(h.store.recent_executions().unwrap().is_empty());
    }
    let denied = Core::new(
        h.store.clone(),
        vec![h.adapter.clone()],
        Arc::new(SystemClock),
        Policy::default(),
    );
    assert_eq!(
        denied
            .execute_batch(h.batch(), &operation(10_000))
            .await
            .unwrap_err()
            .code,
        "POLICY_DENIED"
    );
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 0);
}

#[tokio::test(start_paused = true)]
async fn a_grant_for_one_device_never_authorizes_the_remaining_selection() {
    let h = Harness::new(vec![Behavior::Complete; 2]).await;
    let core = Core::new(
        h.store.clone(),
        vec![h.adapter.clone()],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: "dev-batch-0".into(),
                component_id: "main".into(),
                capability: "power".into(),
                action: "set".into(),
                transports: vec![Transport::Lan, Transport::Cloud],
                argument_schema: json!({"type":"object"}),
            }],
        },
    );
    assert_eq!(
        core.execute_batch(h.batch(), &operation(10_000))
            .await
            .unwrap_err()
            .code,
        "POLICY_DENIED"
    );
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 0);
    assert!(h.store.recent_executions().unwrap().is_empty());
}

#[tokio::test(start_paused = true)]
async fn maximum_batch_keeps_nested_composite_evidence_and_individual_wait_conditions() {
    let h = Harness::new(vec![Behavior::Complete; 32]).await;
    let mut batch = h.batch();
    batch.commands[0].capability = "climate".into();
    batch.commands[0].arguments = json!({"mode":"heat", "target_temperature":24});
    batch.commands[1].wait_for = WaitFor::Accepted;
    let result = h
        .core
        .execute_batch(batch, &operation(10_000))
        .await
        .unwrap();
    assert_eq!(result.items.len(), 32);
    assert_eq!(result.status, Status::Partial);
    let children = result.items[0].items.as_ref().unwrap();
    assert_eq!(children.len(), 2);
    assert!(
        children
            .iter()
            .all(|child| child.status == Status::Verified && child.verification.is_some())
    );
    assert_eq!(result.items[1].status, Status::Accepted);
    assert!(result.items[1].error.is_none());
    assert_eq!(h.adapter.maximum.load(Ordering::SeqCst), 4);
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 33);
}

#[tokio::test(start_paused = true)]
async fn dropping_a_batch_future_leaves_no_detached_calls_or_queued_work() {
    let h = Harness::new(vec![Behavior::Slow; 5]).await;
    let scope = operation(10_000);
    assert!(
        tokio::time::timeout(
            Duration::from_millis(1),
            h.core.execute_batch(h.batch(), &scope)
        )
        .await
        .is_err()
    );
    assert_eq!(h.adapter.active.load(Ordering::SeqCst), 0);
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 4);
    let reopened = SqliteStore::open(&h.path).unwrap();
    let receipts = reopened.recent_executions().unwrap();
    assert_eq!(receipts.len(), 4);
    assert!(receipts.iter().all(|item| item.status == Status::Unknown));
    assert!(reopened.execution("request-4").unwrap().is_none());
}

#[tokio::test(start_paused = true)]
async fn existing_request_conflicts_and_child_receipts_prevent_all_new_batch_writes() {
    let h = Harness::new(vec![Behavior::Complete; 2]).await;
    let mut prior = h.batch().commands.remove(0);
    prior.request_id = "occupied".into();
    h.core.execute(prior, CancellationToken::new()).await;
    for composite in [false, true] {
        let mut batch = h.batch();
        if composite {
            batch.commands[1].capability = "climate".into();
            batch.commands[1].arguments = json!({"mode":"heat","target_temperature":24});
            let mut child = batch.commands[0].clone();
            child.request_id = format!("{}:0", batch.commands[1].request_id);
            h.core.execute(child, CancellationToken::new()).await;
        } else {
            batch.commands[1].request_id = "occupied".into();
        }
        let sends = h.adapter.sends.load(Ordering::SeqCst);
        assert_eq!(
            h.core
                .execute_batch(batch, &operation(10_000))
                .await
                .unwrap_err()
                .code,
            "REQUEST_CONFLICT"
        );
        assert_eq!(h.adapter.sends.load(Ordering::SeqCst), sends);
    }
}

#[tokio::test(start_paused = true)]
async fn batch_deadline_and_cancellation_prevent_queued_sends_and_preserve_completed_items() {
    for behavior in [Behavior::Slow, Behavior::Cancel] {
        let h = Harness::new(vec![
            Behavior::Complete,
            Behavior::Complete,
            Behavior::Complete,
            Behavior::Complete,
            behavior,
            behavior,
            behavior,
            behavior,
            Behavior::Complete,
        ])
        .await;
        let batch = h.batch();
        let result = h.core.execute_batch(batch, &operation(10)).await.unwrap();
        assert_eq!(result.status, Status::Partial);
        assert!(
            result.items[..4]
                .iter()
                .all(|item| item.status == Status::Verified)
        );
        assert_eq!(result.items[4].status, Status::Unknown);
        assert_eq!(result.items[8].status, Status::Failed);
        assert_eq!(result.items[8].attempts[0].phase, Phase::NotSent);
        assert!(h.adapter.sends.load(Ordering::SeqCst) <= 8);
        assert_eq!(h.adapter.active.load(Ordering::SeqCst), 0);
        assert_eq!(h.store.recent_executions().unwrap().len(), 9);
    }
    let h = Harness::new(vec![Behavior::Complete]).await;
    let expired = operation(1);
    tokio::time::advance(Duration::from_millis(2)).await;
    assert_eq!(
        h.core
            .execute_batch(h.batch(), &expired)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    let cancelled = operation(10_000);
    cancelled.cancellation.cancel();
    assert_eq!(
        h.core
            .execute_batch(h.batch(), &cancelled)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 0);
}
