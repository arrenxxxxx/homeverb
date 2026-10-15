use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use homeverb_core::*;
use homeverb_domain::*;
use homeverb_mock::{DEVICE_ID, MockAdapter};
use homeverb_storage::SqliteStore;
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug)]
enum Fault {
    None,
    NoSourceTime,
    Missing,
    Request,
    Device,
    Component,
    Connection,
    OldReceipt,
    FutureReceipt,
    OldCompletion,
    FutureCompletion,
    Uncertain,
    Rejected,
    NotSent,
    SlowAccepted,
    CancelAccepted,
}

struct FixedClock(DateTime<Utc>);
impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

struct CompletionAdapter {
    fault: Fault,
    clock: Arc<FixedClock>,
    sends: AtomicUsize,
    reads: AtomicUsize,
}

#[async_trait]
impl Adapter for CompletionAdapter {
    fn id(&self) -> &str {
        "homeverb.mock"
    }

    async fn discover(&self, _operation: &Operation) -> Result<Vec<Descriptor>> {
        let mut descriptor = MockAdapter::descriptor();
        descriptor.device.components[0].capabilities = vec!["vendor.example.operation".into()];
        descriptor.capabilities = vec![
            serde_json::from_value(json!({
                "schema_version": "1.0", "id": "vendor.example.operation", "revision": 1,
                "properties": {}, "events": {},
                "actions": {"run": {
                    "parameters": {"type": "object", "additionalProperties": false},
                    "transports": ["lan", "cloud"], "idempotency": "non_idempotent",
                    "risk": "low", "atomic": false, "verification": "correlated_completion"
                }}
            }))
            .unwrap(),
        ];
        Ok(vec![descriptor])
    }

    async fn read(
        &self,
        device: &Device,
        component: &str,
        _connection: &Connection,
        _operation: &Operation,
    ) -> Result<Snapshot> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(Snapshot {
            schema_version: SCHEMA_VERSION.into(),
            device_id: device.id.clone(),
            component_id: component.into(),
            from_cache: false,
            observations: Default::default(),
            warnings: vec![],
        })
    }

    async fn execute(
        &self,
        validated: &ValidatedCommand,
        connection: &Connection,
        progress: WriteProgress,
        operation: &Operation,
    ) -> Result<AdapterOutcome> {
        self.sends.fetch_add(1, Ordering::SeqCst);
        let command = validated.command();
        let now = self.clock.now();
        let mut completion = CorrelatedCompletion {
            request_id: command.request_id.clone(),
            device_id: command.device_id.clone(),
            component_id: command.component_id.clone(),
            connection_id: connection.id.clone(),
            completed_at: Some(now),
            received_at: now,
        };
        let phase = match self.fault {
            Fault::NotSent => Phase::NotSent,
            Fault::Uncertain => Phase::PossiblySent,
            Fault::Rejected => Phase::Rejected,
            _ => Phase::Accepted,
        };
        if phase != Phase::NotSent {
            progress.mark_sent();
        }
        if phase == Phase::Accepted {
            progress.mark_accepted();
        }
        match self.fault {
            Fault::NoSourceTime => completion.completed_at = None,
            Fault::Request => completion.request_id.push_str("-other"),
            Fault::Device => completion.device_id.push_str("-other"),
            Fault::Component => completion.component_id.push_str("-other"),
            Fault::Connection => completion.connection_id.push_str("-other"),
            Fault::OldReceipt => {
                completion.received_at -= Duration::milliseconds(1);
                completion.completed_at = None;
            }
            Fault::FutureReceipt => completion.received_at += Duration::milliseconds(1),
            Fault::OldCompletion => completion.completed_at = Some(now - Duration::milliseconds(1)),
            Fault::FutureCompletion => {
                completion.completed_at = Some(now + Duration::milliseconds(1))
            }
            Fault::SlowAccepted | Fault::CancelAccepted => {
                if matches!(self.fault, Fault::CancelAccepted) {
                    operation.cancellation.cancel();
                }
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
            _ => (),
        }
        Ok(AdapterOutcome {
            phase,
            error: (phase == Phase::NotSent)
                .then(|| Error::new("INVALID_ARGUMENT", "The fictional request was not sent.")),
            completion: (!matches!(self.fault, Fault::Missing)).then_some(completion),
        })
    }
}

struct Harness {
    core: Core,
    adapter: Arc<CompletionAdapter>,
    store: Arc<SqliteStore>,
    path: PathBuf,
}

impl Harness {
    async fn new(fault: Fault) -> Self {
        let path =
            std::env::temp_dir().join(format!("homeverb-completion-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let clock = Arc::new(FixedClock(Utc::now()));
        let adapter = Arc::new(CompletionAdapter {
            fault,
            clock: clock.clone(),
            sends: AtomicUsize::new(0),
            reads: AtomicUsize::new(0),
        });
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            clock,
            Policy {
                grants: vec![Grant {
                    device_id: DEVICE_ID.into(),
                    component_id: "main".into(),
                    capability: "vendor.example.operation".into(),
                    action: "run".into(),
                    transports: vec![Transport::Lan, Transport::Cloud],
                    argument_schema: json!({"type": "object"}),
                }],
            },
        );
        core.discover(&Operation::new(10_000, CancellationToken::new()).unwrap())
            .await
            .unwrap();
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
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn command() -> Command {
    Command {
        schema_version: SCHEMA_VERSION.into(),
        request_id: uuid::Uuid::new_v4().to_string(),
        device_id: DEVICE_ID.into(),
        component_id: "main".into(),
        capability: "vendor.example.operation".into(),
        action: "run".into(),
        arguments: json!({}),
        connection: Route::Auto,
        timeout_ms: 10_000,
        wait_for: WaitFor::Verified,
    }
}

#[tokio::test]
async fn completion_verifies_actions_without_observable_properties_and_survives_reopening() {
    for (fault, route, source) in [
        (Fault::None, Route::Lan, Transport::Lan),
        (Fault::NoSourceTime, Route::Cloud, Transport::Cloud),
    ] {
        let h = Harness::new(fault).await;
        let command = Command {
            connection: route,
            ..command()
        };
        let result = h.execute(command.clone()).await;
        assert_eq!(result.status, Status::Verified, "{result:?}");
        assert!(result.error.is_none());
        let evidence = result.verification.as_ref().unwrap();
        assert_eq!(evidence.kind, VerificationMethod::CorrelatedCompletion);
        assert_eq!(evidence.source, source);
        assert_eq!(
            evidence.observed_at.is_none(),
            matches!(fault, Fault::NoSourceTime)
        );
        assert_eq!(evidence.received_at, result.started_at);
        assert!(h.store.snapshot(DEVICE_ID, "main").unwrap().is_none());
        let reopened = SqliteStore::open(&h.path).unwrap();
        let saved = reopened.execution(&command.request_id).unwrap().unwrap().1;
        assert_eq!(
            serde_json::to_value(saved).unwrap(),
            serde_json::to_value(&result).unwrap()
        );
        let replay = h.execute(command).await;
        assert_eq!(
            serde_json::to_value(replay).unwrap(),
            serde_json::to_value(result).unwrap()
        );
        assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 1);
        assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn missing_mismatched_and_invalid_completion_preserve_acceptance_without_replay() {
    for fault in [
        Fault::Missing,
        Fault::Request,
        Fault::Device,
        Fault::Component,
        Fault::Connection,
        Fault::OldReceipt,
        Fault::FutureReceipt,
        Fault::OldCompletion,
        Fault::FutureCompletion,
    ] {
        let h = Harness::new(fault).await;
        let command = command();
        let result = h.execute(command.clone()).await;
        assert_eq!(result.status, Status::Accepted, "{fault:?}: {result:?}");
        assert_eq!(result.error.as_ref().unwrap().code, "VERIFICATION_PENDING");
        assert!(result.verification.is_none());
        assert_eq!(result.attempts.len(), 1);
        assert_eq!(h.execute(command).await.status, Status::Accepted);
        assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 1);
        assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
        assert!(h.store.snapshot(DEVICE_ID, "main").unwrap().is_none());
    }
}

#[tokio::test]
async fn completion_cannot_promote_uncertain_rejected_or_unsent_results() {
    for (fault, status) in [
        (Fault::Uncertain, Status::Unknown),
        (Fault::Rejected, Status::Failed),
        (Fault::NotSent, Status::Failed),
    ] {
        let h = Harness::new(fault).await;
        let result = h.execute(command()).await;
        assert_eq!(result.status, status, "{fault:?}: {result:?}");
        assert!(result.verification.is_none());
        assert_eq!(result.attempts.len(), 1);
        assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn accepted_only_and_other_verification_methods_do_not_use_completion() {
    let h = Harness::new(Fault::None).await;
    let result = h
        .execute(Command {
            wait_for: WaitFor::Accepted,
            ..command()
        })
        .await;
    assert_eq!(result.status, Status::Accepted);
    assert!(result.error.is_none());
    assert!(result.verification.is_none());
    for method in [
        VerificationMethod::Unavailable,
        VerificationMethod::StateMatch,
    ] {
        let mut descriptor = h.store.devices().unwrap().remove(0);
        descriptor.capabilities[0]
            .actions
            .get_mut("run")
            .unwrap()
            .verification = method;
        h.store.save_device(&descriptor).unwrap();
        let result = h.execute(command()).await;
        assert_eq!(result.status, Status::Accepted);
        assert_eq!(result.error.unwrap().code, "VERIFICATION_PENDING");
        assert!(result.verification.is_none());
    }
    assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 1);
}

#[tokio::test(start_paused = true)]
async fn cancellation_and_deadline_during_completion_wait_preserve_known_acceptance() {
    for fault in [Fault::SlowAccepted, Fault::CancelAccepted] {
        let h = Harness::new(fault).await;
        let command = Command {
            timeout_ms: 10,
            ..command()
        };
        let start = tokio::time::Instant::now();
        let result = h.execute(command.clone()).await;
        assert!(start.elapsed() < std::time::Duration::from_millis(100));
        assert_eq!(result.status, Status::Accepted, "{fault:?}: {result:?}");
        assert_eq!(result.error.unwrap().code, "VERIFICATION_PENDING");
        assert!(result.verification.is_none());
        assert_eq!(h.execute(command).await.status, Status::Accepted);
        assert_eq!(h.adapter.sends.load(Ordering::SeqCst), 1);
        assert_eq!(h.adapter.reads.load(Ordering::SeqCst), 0);
    }
}
