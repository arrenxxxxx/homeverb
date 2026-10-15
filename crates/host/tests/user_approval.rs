use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::*;
use homeverb_host::{HostRequest, Session};
use homeverb_mock::MockAdapter;
use homeverb_storage::SqliteStore;
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio_util::sync::CancellationToken;

fn validate_result(result: &Execution) {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../schemas/command-result.schema.json")).unwrap();
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&json!(result))
        .unwrap();
}

fn operation() -> Operation {
    Operation::new(180_000, CancellationToken::new()).unwrap()
}
fn command() -> Command {
    let mut value: Command = serde_json::from_str(include_str!(
        "../../../examples/command.approved-cycle.json"
    ))
    .unwrap();
    value.request_id = uuid::Uuid::new_v4().to_string();
    value
}
fn descriptor() -> Descriptor {
    let mut value = MockAdapter::descriptor();
    value.device.id = command().device_id;
    value.device.components = vec![Component {
        id: "main".into(),
        capabilities: vec!["vendor.example.cycle".into()],
    }];
    value
        .device
        .connections
        .retain(|c| c.transport == Transport::Cloud);
    value.capabilities = vec![
        serde_json::from_str(include_str!(
            "../../../examples/capability.approved-cycle.json"
        ))
        .unwrap(),
    ];
    value
}
struct Appliance {
    calls: AtomicUsize,
    sends: AtomicUsize,
    fault: AtomicUsize,
    started: tokio::sync::Notify,
}
#[async_trait]
impl Adapter for Appliance {
    fn id(&self) -> &str {
        "homeverb.mock"
    }
    async fn discover(&self, _: &Operation) -> Result<Vec<Descriptor>> {
        Ok(vec![descriptor()])
    }
    async fn read(&self, _: &Device, _: &str, _: &Connection, _: &Operation) -> Result<Snapshot> {
        panic!("An unavailable-verification action must not read or poll.");
    }
    async fn execute(
        &self,
        command: &ValidatedCommand,
        _: &Connection,
        progress: WriteProgress,
        _: &Operation,
    ) -> Result<AdapterOutcome> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if !command.has_user_approval() {
            return Err(Error::new(
                "POLICY_DENIED",
                "This synthetic consequential action requires trusted approval.",
            ));
        }
        assert_eq!(command.command().arguments, json!({"program":"normal"}));
        self.sends.fetch_add(1, Ordering::SeqCst);
        progress.mark_sent();
        self.started.notify_one();
        let phase = match self.fault.load(Ordering::SeqCst) {
            1 => Phase::PossiblySent,
            2 => Phase::Rejected,
            3 => {
                std::future::pending::<()>().await;
                unreachable!()
            }
            _ => {
                progress.mark_accepted();
                Phase::Accepted
            }
        };
        Ok(AdapterOutcome {
            phase,
            error: None,
            completion: None,
        })
    }
}
struct Fixture {
    core: Core,
    store: Arc<SqliteStore>,
    appliance: Arc<Appliance>,
    policy: Policy,
    directory: std::path::PathBuf,
}
impl Fixture {
    async fn new() -> Self {
        Self::with_descriptor(descriptor()).await
    }
    async fn with_descriptor(descriptor: Descriptor) -> Self {
        let directory =
            std::env::temp_dir().join(format!("homeverb-approval-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&directory).unwrap());
        let appliance = Arc::new(Appliance {
            calls: AtomicUsize::new(0),
            sends: AtomicUsize::new(0),
            fault: AtomicUsize::new(0),
            started: tokio::sync::Notify::new(),
        });
        let c = command();
        let policy = Policy {
            grants: vec![Grant {
                device_id: c.device_id,
                component_id: c.component_id,
                capability: c.capability,
                action: c.action,
                transports: vec![Transport::Cloud],
                argument_schema: json!({"const":{"program":"normal"}}),
            }],
        };
        let core = Core::new(
            store.clone(),
            vec![appliance.clone()],
            Arc::new(SystemClock),
            policy.clone(),
        );
        store.save_device(&descriptor).unwrap();
        Self {
            core,
            store,
            appliance,
            policy,
            directory,
        }
    }
    fn other_core(&self) -> Core {
        Core::new(
            self.store.clone(),
            vec![self.appliance.clone()],
            Arc::new(SystemClock),
            self.policy.clone(),
        )
    }
    fn sends(&self) -> usize {
        self.appliance.sends.load(Ordering::SeqCst)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[tokio::test]
async fn trusted_preview_requires_consent_entry_point_and_records_exact_one_send() {
    let f = Fixture::new().await;
    let c = command();
    assert_eq!(f.core.plan(&c).unwrap_err().code, "POLICY_DENIED");
    assert_eq!(
        f.core
            .execute(c.clone(), CancellationToken::new())
            .await
            .error
            .unwrap()
            .code,
        "POLICY_DENIED"
    );
    let preview = f.core.prepare_user_approval(c.clone()).unwrap();
    assert_eq!(preview.command(), &c);
    assert_eq!(preview.descriptor().device.id, c.device_id);
    assert_eq!(preview.action().risk, Risk::Elevated);
    assert_eq!(preview.connection().transport, Transport::Cloud);
    assert!(preview.expires_at() > chrono::Utc::now());
    assert_eq!(f.sends(), 0);
    assert!(f.store.recent_executions().unwrap().is_empty());
    let concurrent = f.core.prepare_user_approval(c.clone()).unwrap();
    let result = f.core.execute_user_approved(preview, &operation()).await;
    validate_result(&result);
    assert_eq!(result.status, Status::Accepted);
    assert_eq!(result.error.as_ref().unwrap().code, "VERIFICATION_PENDING");
    assert_eq!(
        result.warnings,
        ["User approval was supplied for this exact command."]
    );
    assert_eq!(f.sends(), 1);
    assert_eq!(
        json!(f.core.execute_user_approved(concurrent, &operation()).await),
        json!(result)
    );
    assert_eq!(f.sends(), 1);
    assert_eq!(
        f.core.prepare_user_approval(c.clone()).err().unwrap().code,
        "REQUEST_CONFLICT"
    );
    let reopened = f.other_core();
    assert_eq!(
        json!(
            reopened
                .execution(&c.request_id, &operation())
                .await
                .unwrap()
        ),
        json!(result)
    );
    assert_eq!(f.sends(), 1);
}

#[tokio::test(start_paused = true)]
async fn expiry_scope_changes_wrong_core_and_invalidation_refuse_before_send() {
    for reason in ["expired", "invalidated", "other_core", "metadata"] {
        let f = Fixture::new().await;
        let approval = f.core.prepare_user_approval(command()).unwrap();
        match reason {
            "expired" => tokio::time::advance(std::time::Duration::from_secs(60)).await,
            "invalidated" => f.core.invalidate_user_approvals(),
            "metadata" => {
                let mut d = descriptor();
                d.capabilities[0].revision += 1;
                f.store.save_device(&d).unwrap();
            }
            _ => (),
        }
        let other = f.other_core();
        let core = if reason == "other_core" {
            &other
        } else {
            &f.core
        };
        let result = core.execute_user_approved(approval, &operation()).await;
        validate_result(&result);
        assert_eq!(result.status, Status::Failed);
        assert_eq!(
            result.error.unwrap().code,
            if reason == "expired" {
                "APPROVAL_EXPIRED"
            } else {
                "APPROVAL_INVALID"
            }
        );
        assert!(result.attempts.is_empty());
        assert_eq!(f.appliance.calls.load(Ordering::SeqCst), 0);
        assert!(f.store.recent_executions().unwrap().is_empty());
    }
}

#[tokio::test]
async fn preparations_cannot_expand_grants_risk_routes_or_decompose_actions() {
    let f = Fixture::new().await;
    for change in [
        "arguments",
        "component",
        "device",
        "action",
        "route",
        "auto",
    ] {
        let mut c = command();
        match change {
            "arguments" => c.arguments = json!({"program":"other"}),
            "component" => c.component_id = "other".into(),
            "device" => c.device_id = "other".into(),
            "action" => c.action = "other".into(),
            "route" => c.connection = Route::Lan,
            _ => c.connection = Route::Auto,
        }
        assert!(f.core.prepare_user_approval(c).is_err());
    }
    for risk in [Risk::Low, Risk::High] {
        let mut d = descriptor();
        d.capabilities[0].actions.get_mut("start").unwrap().risk = risk;
        f.store.save_device(&d).unwrap();
        assert_eq!(
            f.core.prepare_user_approval(command()).err().unwrap().code,
            "POLICY_DENIED"
        );
    }
    let mut d = descriptor();
    d.capabilities[0]
        .actions
        .get_mut("start")
        .unwrap()
        .independent_fields = true;
    f.store.save_device(&d).unwrap();
    assert!(f.core.prepare_user_approval(command()).is_err());
    let mut d = descriptor();
    let mut extra = d.device.connections[0].clone();
    extra.id = "second-cloud".into();
    d.device.connections.push(extra);
    let ambiguous = Fixture::with_descriptor(d).await;
    assert_eq!(
        ambiguous
            .core
            .prepare_user_approval(command())
            .err()
            .unwrap()
            .code,
        "POLICY_DENIED"
    );
    f.store.save_device(&descriptor()).unwrap();
    let denied = Core::new(
        f.store.clone(),
        vec![f.appliance.clone()],
        Arc::new(SystemClock),
        Policy::default(),
    );
    assert_eq!(
        denied.prepare_user_approval(command()).err().unwrap().code,
        "POLICY_DENIED"
    );
    assert_eq!(f.sends(), 0);
}

#[tokio::test]
async fn weakened_risk_metadata_cannot_forge_the_adapter_approval_marker() {
    let f = Fixture::new().await;
    let mut d = descriptor();
    d.capabilities[0].actions.get_mut("start").unwrap().risk = Risk::Low;
    f.store.save_device(&d).unwrap();
    let result = f.core.execute(command(), CancellationToken::new()).await;
    validate_result(&result);
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
    assert_eq!(result.attempts[0].phase, Phase::NotSent);
    assert_eq!(f.sends(), 0);
}

#[tokio::test]
async fn approved_outcomes_keep_uncertainty_and_rejection_without_replay() {
    for (fault, status) in [(1, Status::Unknown), (2, Status::Failed)] {
        let f = Fixture::new().await;
        f.appliance.fault.store(fault, Ordering::SeqCst);
        let c = command();
        let approval = f.core.prepare_user_approval(c.clone()).unwrap();
        let concurrent = f.core.prepare_user_approval(c.clone()).unwrap();
        let result = f.core.execute_user_approved(approval, &operation()).await;
        validate_result(&result);
        assert_eq!(result.status, status);
        assert!(result.verification.is_none());
        assert_eq!(
            json!(f.core.execute_user_approved(concurrent, &operation()).await),
            json!(result)
        );
        assert_eq!(f.sends(), 1);
    }
}

#[tokio::test(start_paused = true)]
async fn host_lifecycle_invalidates_previews_and_cancels_possible_sends() {
    for end in ["background", "close", "cancel", "expiry"] {
        let f = Fixture::new().await;
        let session = Session::new(f.other_core());
        let mut c = command();
        c.timeout_ms = 180_000;
        let approval = session
            .prepare_user_approval("prepare".into(), c.clone())
            .await
            .unwrap();
        f.appliance.fault.store(3, Ordering::SeqCst);
        let work = session.execute_user_approved("execute".into(), approval);
        tokio::pin!(work);
        tokio::select! { _=f.appliance.started.notified()=>(), result=&mut work=>panic!("Expected pending send: {result:?}") }
        match end {
            "background" => session.set_foreground(false),
            "close" => session.close(),
            "cancel" => {
                assert!(session.cancel("execute"));
            }
            _ => tokio::time::advance(std::time::Duration::from_secs(60)).await,
        }
        let result = work.await.unwrap();
        validate_result(&result);
        assert_eq!(result.status, Status::Unknown);
        assert_eq!(result.attempts[0].phase, Phase::PossiblySent);
        assert_eq!(f.sends(), 1);
        drop(f.store.lock_device(&c.device_id).unwrap());
        assert_eq!(
            json!(f.core.execution(&c.request_id, &operation()).await.unwrap()),
            json!(result)
        );
    }
    let f = Fixture::new().await;
    let session = Session::new(f.other_core());
    let approval = session
        .prepare_user_approval("prepare".into(), command())
        .await
        .unwrap();
    session.set_foreground(false);
    assert_eq!(
        session
            .prepare_user_approval("hidden".into(), command())
            .await
            .err()
            .unwrap()
            .code,
        "HOST_BACKGROUND"
    );
    session.set_foreground(true);
    assert_eq!(
        session
            .execute_user_approved("execute".into(), approval)
            .await
            .unwrap()
            .error
            .unwrap()
            .code,
        "APPROVAL_INVALID"
    );
    let approval = session
        .prepare_user_approval("prepare".into(), command())
        .await
        .unwrap();
    session.close();
    assert_eq!(
        session
            .execute_user_approved("closed".into(), approval)
            .await
            .unwrap_err()
            .code,
        "HOST_CLOSED"
    );
    assert_eq!(f.sends(), 0);
}

#[test]
fn ordinary_json_cannot_prepare_approve_or_forge_a_marker() {
    for request in [
        json!({"method":"prepare_user_approval","command":command()}),
        json!({"method":"execute_user_approved","command":command()}),
        json!({"method":"execute","command":command(),"approved":true}),
    ] {
        assert!(serde_json::from_value::<HostRequest>(json!({"schema_version":"1.0","operation_id":"forged","timeout_ms":10000,"request":request})).is_err());
    }
    let mut c = json!(command());
    c["user_approved"] = json!(true);
    assert!(serde_json::from_value::<Command>(c).is_err());
}

#[tokio::test]
async fn cancelled_or_unjournaled_approved_commands_never_reach_the_appliance() {
    for failure in ["cancelled", "journal", "conflict", "retired"] {
        let f = Fixture::new().await;
        let c = command();
        let preview = f.core.prepare_user_approval(c.clone()).unwrap();
        let op = operation();
        match failure {
            "cancelled" => op.cancellation.cancel(),
            "journal" => {
                rusqlite::Connection::open(f.directory.join("homeverb.sqlite3")).unwrap()
                    .execute_batch("CREATE TRIGGER refuse_approval BEFORE INSERT ON executions BEGIN SELECT RAISE(FAIL, 'PRIVATE-STORE-DENIAL'); END;").unwrap();
            }
            "conflict" => {
                let mut other = c.clone();
                other.arguments = json!({"program":"other"});
                f.store
                    .finish_execution(&other, &Execution::new(&other, chrono::Utc::now()))
                    .unwrap();
            }
            _ => {
                f.store
                    .finish_execution(&c, &Execution::new(&c, chrono::Utc::now()))
                    .unwrap();
                f.store.clear_execution_history().unwrap();
            }
        }
        let result = f.core.execute_user_approved(preview, &op).await;
        validate_result(&result);
        assert_eq!(result.status, Status::Failed);
        assert_eq!(
            result.error.as_ref().unwrap().code,
            match failure {
                "cancelled" => "CANCELLED",
                "journal" => "STORAGE_ERROR",
                "conflict" => "REQUEST_CONFLICT",
                _ => "REQUEST_RETIRED",
            }
        );
        assert!(!json!(result).to_string().contains("PRIVATE-STORE-DENIAL"));
        assert_eq!(f.appliance.calls.load(Ordering::SeqCst), 0);
        assert_eq!(f.sends(), 0);
    }
}

async fn native_prepare(session: &Session) -> serde_json::Value {
    session
        .prepare_native_approval(homeverb_host::PrepareApprovalRequest {
            schema_version: SCHEMA_VERSION.into(),
            operation_id: uuid::Uuid::new_v4().to_string(),
            command: command(),
        })
        .await
}
fn native_execute(id: u64) -> homeverb_host::ExecuteApprovalRequest {
    homeverb_host::ExecuteApprovalRequest {
        schema_version: SCHEMA_VERSION.into(),
        operation_id: uuid::Uuid::new_v4().to_string(),
        approval_id: id,
    }
}
fn preview_id(value: &serde_json::Value) -> u64 {
    assert!(value["error"].is_null(), "{value}");
    value["data"]["approval_id"].as_u64().unwrap()
}

#[tokio::test(start_paused = true)]
async fn native_registry_bounds_prunes_and_distinguishes_retained_expiry() {
    let f = Fixture::new().await;
    let session = Session::new(f.other_core());
    let mut ids = Vec::new();
    for _ in 0..8 {
        ids.push(preview_id(&native_prepare(&session).await));
    }
    assert_eq!(
        native_prepare(&session).await["error"]["code"],
        "RESOURCE_BUSY"
    );
    assert!(session.discard_native_approval(ids[0]));
    assert!(!session.discard_native_approval(ids[0]));
    let retained = preview_id(&native_prepare(&session).await);
    tokio::time::advance(std::time::Duration::from_secs(60)).await;
    let expired = session
        .execute_native_approved(native_execute(retained))
        .await;
    assert_eq!(expired["status"], "failed");
    assert_eq!(expired["error"]["code"], "APPROVAL_EXPIRED");
    assert_eq!(expired["attempts"], json!([]));
    let fresh = preview_id(&native_prepare(&session).await);
    assert_eq!(
        session
            .execute_native_approved(native_execute(ids[1]))
            .await["error"]["code"],
        "APPROVAL_INVALID"
    );
    assert_eq!(
        session.execute_native_approved(native_execute(fresh)).await["status"],
        "accepted"
    );
    assert_eq!(f.sends(), 1);
}

#[tokio::test]
async fn native_registry_consumes_admission_failures_and_rejects_other_sessions() {
    let f = Fixture::new().await;
    let session = Session::new(f.other_core());
    let foreign = Session::new(f.other_core());
    let id = preview_id(&native_prepare(&session).await);
    assert_eq!(
        foreign.execute_native_approved(native_execute(id)).await["error"]["code"],
        "APPROVAL_INVALID"
    );
    assert!(!foreign.discard_native_approval(id));
    assert_eq!(
        session.execute_native_approved(native_execute(id)).await["status"],
        "accepted"
    );
    assert_eq!(
        session.execute_native_approved(native_execute(id)).await["error"]["code"],
        "APPROVAL_INVALID"
    );
    for failure in ["schema", "operation", "background", "close"] {
        let session = Session::new(f.other_core());
        let id = preview_id(&native_prepare(&session).await);
        let mut input = native_execute(id);
        let code = match failure {
            "schema" => {
                input.schema_version = "2.0".into();
                "INVALID_ARGUMENT"
            }
            "operation" => {
                input.operation_id.clear();
                "INVALID_ARGUMENT"
            }
            "background" => {
                session.set_foreground(false);
                "HOST_BACKGROUND"
            }
            _ => {
                session.close();
                "HOST_CLOSED"
            }
        };
        assert_eq!(
            session.execute_native_approved(input).await["error"]["code"],
            code
        );
        assert!(!session.discard_native_approval(id));
        if failure != "close" {
            session.set_foreground(true);
            assert_eq!(
                session.execute_native_approved(native_execute(id)).await["error"]["code"],
                "APPROVAL_INVALID"
            );
        }
    }
    assert_eq!(f.sends(), 1);
}

#[tokio::test]
async fn native_confirmation_race_consumes_once_without_holding_registry_lock() {
    let f = Fixture::new().await;
    let session = Session::new(f.other_core());
    let id = preview_id(&native_prepare(&session).await);
    f.appliance.fault.store(3, Ordering::SeqCst);
    let input = native_execute(id);
    let operation_id = input.operation_id.clone();
    let work = session.execute_native_approved(input);
    tokio::pin!(work);
    tokio::select! { _=f.appliance.started.notified()=>(), result=&mut work=>panic!("Expected pending send: {result:?}") }
    assert_eq!(
        session.execute_native_approved(native_execute(id)).await["error"]["code"],
        "APPROVAL_INVALID"
    );
    assert!(!session.discard_native_approval(id));
    assert!(session.cancel(&operation_id));
    let result = work.await;
    assert_eq!(result["status"], "unknown");
    assert_eq!(f.sends(), 1);
}

#[tokio::test]
async fn native_preparation_cannot_publish_across_background_foreground_race() {
    struct PausedClock {
        entered: Arc<std::sync::Barrier>,
        resume: Arc<std::sync::Barrier>,
    }
    impl Clock for PausedClock {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            self.entered.wait();
            self.resume.wait();
            chrono::Utc::now()
        }
    }
    let f = Fixture::new().await;
    let entered = Arc::new(std::sync::Barrier::new(2));
    let resume = Arc::new(std::sync::Barrier::new(2));
    let session = Arc::new(Session::new(Core::new(
        f.store.clone(),
        vec![f.appliance.clone()],
        Arc::new(PausedClock {
            entered: entered.clone(),
            resume: resume.clone(),
        }),
        f.policy.clone(),
    )));
    let worker = session.clone();
    let task = std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(native_prepare(&worker))
    });
    entered.wait();
    session.set_foreground(false);
    session.set_foreground(true);
    resume.wait();
    let result = task.join().unwrap();
    assert_eq!(result["error"]["code"], "APPROVAL_INVALID");
    assert!(result["data"].is_null());
    assert_eq!(f.sends(), 0);
}
