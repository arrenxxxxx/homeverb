use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::*;
use homeverb_host::{HostRequest, Session};
use homeverb_mock::MockAdapter;
use homeverb_storage::SqliteStore;
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};
use tokio_util::sync::CancellationToken;

#[path = "connection_link/native.rs"]
mod native;

const DEVICE: &str = "dev-mock-ac";
const ANCHOR: &str = "conn-example-cloud";

fn operation() -> Operation {
    Operation::new(180_000, CancellationToken::new()).unwrap()
}
fn error<T>(result: Result<T>, code: &str) {
    assert_eq!(result.err().unwrap().code, code);
}
fn preferences(alias: Option<&str>) -> PreferencesInput {
    PreferencesInput {
        alias: alias.map(str::to_owned),
        room_id: Some("room-test".into()),
        favorite: true,
    }
}

struct Appliance {
    directory: std::path::PathBuf,
    fault: AtomicU8,
    started: tokio::sync::Notify,
}
#[async_trait]
impl Adapter for Appliance {
    fn id(&self) -> &str {
        "homeverb.mock"
    }
    async fn validate_connection_link(
        &self,
        descriptor: &Descriptor,
        _: &Connection,
        _: &Operation,
    ) -> Result<()> {
        if self.fault.load(Ordering::SeqCst) == 6 {
            self.started.notify_one();
            std::future::pending::<()>().await;
        }
        let store = SqliteStore::open(&self.directory)?;
        match self.fault.load(Ordering::SeqCst) {
            1 => {
                return Err(Error::new(
                    "CONNECTION_LINK_UNSUPPORTED",
                    "Evidence was withdrawn.",
                ));
            }
            2 => {
                let mut changed = descriptor.clone();
                changed.device.model = "Fictional-revised-model".into();
                store.save_device(&changed)?;
            }
            3 => {
                store.set_preferences(DEVICE, &preferences(Some("Changed during confirmation")))?;
            }
            4 | 5 => {
                let account = descriptor.device.connections[0]
                    .account_id
                    .as_ref()
                    .unwrap()
                    .parse()
                    .unwrap();
                store.begin_account_removal(account, &[DEVICE.into()])?;
                if self.fault.load(Ordering::SeqCst) == 5 {
                    store.finish_account_removal(account)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    async fn discover(&self, _: &Operation) -> Result<Vec<Descriptor>> {
        SqliteStore::open(&self.directory)?.devices()
    }
    async fn read(&self, _: &Device, _: &str, _: &Connection, _: &Operation) -> Result<Snapshot> {
        panic!("Connection linking must not read an appliance.")
    }
    async fn execute(
        &self,
        _: &ValidatedCommand,
        _: &Connection,
        _: WriteProgress,
        _: &Operation,
    ) -> Result<AdapterOutcome> {
        panic!("Connection linking must not send a command.")
    }
}

struct Fixture {
    core: Core,
    store: Arc<SqliteStore>,
    adapter: Arc<Appliance>,
    directory: std::path::PathBuf,
}
impl Fixture {
    fn new(transports: Vec<Transport>) -> Self {
        let directory =
            std::env::temp_dir().join(format!("homeverb-link-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&directory).unwrap());
        let account = AccountRecord {
            id: uuid::Uuid::new_v4(),
            provider: "homeverb.mock".into(),
            region: "test".into(),
            label: "Fictional account".into(),
            credential_ref: CredentialRef::new(),
            credential_state: CredentialState::Pending,
            last_complete_inventory_at: None,
        };
        store.insert_account(&account).unwrap();
        let mut descriptor = MockAdapter::descriptor();
        descriptor
            .device
            .connections
            .retain(|c| c.transport == Transport::Cloud);
        descriptor.device.connections[0].account_id = Some(account.id.to_string());
        store.save_device(&descriptor).unwrap();
        store.create_room("room-test", "Test room").unwrap();
        let adapter = Arc::new(Appliance {
            directory: directory.clone(),
            fault: AtomicU8::new(0),
            started: tokio::sync::Notify::new(),
        });
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            Arc::new(SystemClock),
            Policy {
                grants: vec![Grant {
                    device_id: DEVICE.into(),
                    component_id: "main".into(),
                    capability: "power".into(),
                    action: "set".into(),
                    transports,
                    argument_schema: json!({"const":{"state":false}}),
                }],
            },
        );
        Self {
            core,
            store,
            adapter,
            directory,
        }
    }
    fn connection(&self) -> Connection {
        let mut connection = self.store.devices().unwrap()[0].device.connections[0].clone();
        connection.id = "conn-linked-lan".into();
        connection.transport = Transport::Lan;
        connection
    }
    async fn preview(&self) -> PendingConnectionLink {
        self.core
            .prepare_connection_link(DEVICE, ANCHOR, self.connection(), &operation())
            .await
            .unwrap()
    }
    fn other_core(&self) -> Core {
        Core::new(
            self.store.clone(),
            vec![self.adapter.clone()],
            Arc::new(SystemClock),
            Policy::default(),
        )
    }
    fn assert_unlinked(&self) {
        assert_eq!(self.store.devices().unwrap()[0].device.connections.len(), 1);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[tokio::test]
async fn linking_preserves_metadata_preferences_cache_history_and_provider_alias_after_restart() {
    let f = Fixture::new(vec![Transport::Cloud]);
    let original = f.store.devices().unwrap().remove(0);
    f.store
        .set_preferences(DEVICE, &preferences(Some("My appliance")))
        .unwrap();
    let mut snapshot: Snapshot =
        serde_json::from_str(include_str!("../../../examples/state.json")).unwrap();
    snapshot.device_id = DEVICE.into();
    f.store.save_snapshot(&snapshot).unwrap();
    let mut command: Command =
        serde_json::from_str(include_str!("../../../examples/command.json")).unwrap();
    command.device_id = DEVICE.into();
    let mut receipt: Execution = serde_json::from_str(include_str!(
        "../../../examples/command-result.verified.json"
    ))
    .unwrap();
    receipt.device_id = DEVICE.into();
    f.store.save_execution(&command, &receipt).unwrap();
    let before = f.store.preferences().unwrap();
    let preview = f.preview().await;
    assert_eq!(preview.descriptor().device.alias, "My appliance");
    assert_eq!(preview.anchor().id, ANCHOR);
    assert_eq!(preview.connection(), &f.connection());
    assert!(preview.transport_grants().is_empty());
    assert!(preview.expires_at() > chrono::Utc::now());
    assert!(!preview.is_expired());
    f.assert_unlinked();
    let result = f
        .core
        .confirm_connection_link(preview, &operation())
        .await
        .unwrap();
    assert_eq!(result.device.connections.len(), 2);
    assert_eq!(result.device.alias, "My appliance");
    assert_eq!(json!(result.capabilities), json!(original.capabilities));
    let reopened = SqliteStore::open(&f.directory).unwrap();
    assert_eq!(json!(reopened.devices().unwrap()[0]), json!(result));
    assert_eq!(reopened.preferences().unwrap(), before);
    assert_eq!(
        json!(reopened.snapshot(DEVICE, "main").unwrap().unwrap()),
        json!(snapshot)
    );
    assert_eq!(
        json!(reopened.execution(&command.request_id).unwrap().unwrap().1),
        json!(receipt)
    );
    reopened
        .set_preferences(DEVICE, &preferences(None))
        .unwrap();
    assert_eq!(
        reopened.devices().unwrap()[0].device.alias,
        original.device.alias
    );
    f.core.discover(&operation()).await.unwrap();
    assert_eq!(f.store.devices().unwrap().len(), 1);
    assert_eq!(f.store.devices().unwrap()[0].device.connections.len(), 2);
    error(f.store.save_device(&original), "DEVICE_IDENTITY_CONFLICT");
    error(
        f.core
            .publish_discovery("homeverb.mock", &[original], &operation())
            .await,
        "DEVICE_IDENTITY_CONFLICT",
    );
}

#[tokio::test]
async fn proposal_requires_exact_identity_new_id_and_absent_transport() {
    let f = Fixture::new(vec![]);
    error(
        f.core
            .prepare_connection_link("mock-ac", ANCHOR, f.connection(), &operation())
            .await,
        "INVALID_ARGUMENT",
    );
    error(
        f.core
            .prepare_connection_link(DEVICE, "missing", f.connection(), &operation())
            .await,
        "CONNECTION_LINK_INVALID",
    );
    for case in ["adapter", "account", "identity", "id", "transport", "shape"] {
        let mut candidate = f.connection();
        match case {
            "adapter" => candidate.adapter = "other.adapter".into(),
            "account" => candidate.account_id = None,
            "identity" => candidate.identity_ref = "identity:other".into(),
            "id" => candidate.id = ANCHOR.into(),
            "transport" => candidate.transport = Transport::Cloud,
            _ => candidate.id.clear(),
        }
        error(
            f.core
                .prepare_connection_link(DEVICE, ANCHOR, candidate, &operation())
                .await,
            if case == "shape" {
                "CAPABILITY_UNSUPPORTED"
            } else {
                "CONNECTION_LINK_INVALID"
            },
        );
    }
    f.assert_unlinked();
}

#[tokio::test]
async fn adapter_must_be_registered_opted_in_and_revalidate_at_confirmation() {
    let f = Fixture::new(vec![]);
    let unregistered = Core::new(
        f.store.clone(),
        vec![],
        Arc::new(SystemClock),
        Policy::default(),
    );
    error(
        unregistered
            .prepare_connection_link(DEVICE, ANCHOR, f.connection(), &operation())
            .await,
        "CONNECTION_LINK_UNSUPPORTED",
    );
    let mock = MockAdapter::open(
        &f.directory.join("mock.sqlite3"),
        Arc::new(SystemClock),
        Default::default(),
    )
    .unwrap();
    let default = Core::new(
        f.store.clone(),
        vec![Arc::new(mock)],
        Arc::new(SystemClock),
        Policy::default(),
    );
    error(
        default
            .prepare_connection_link(DEVICE, ANCHOR, f.connection(), &operation())
            .await,
        "CONNECTION_LINK_UNSUPPORTED",
    );
    let preview = f.preview().await;
    f.adapter.fault.store(1, Ordering::SeqCst);
    error(
        f.core.confirm_connection_link(preview, &operation()).await,
        "CONNECTION_LINK_UNSUPPORTED",
    );
    f.assert_unlinked();
}

#[tokio::test(start_paused = true)]
async fn expiry_foreign_core_and_invalidation_require_a_new_preview() {
    for case in ["expiry", "foreign", "invalidated"] {
        let f = Fixture::new(vec![]);
        let preview = f.preview().await;
        let other = f.other_core();
        let target = if case == "foreign" { &other } else { &f.core };
        if case == "expiry" {
            tokio::time::advance(std::time::Duration::from_secs(60)).await;
            assert!(preview.is_expired());
        } else if case == "invalidated" {
            f.core.invalidate_user_approvals();
        }
        error(
            target.confirm_connection_link(preview, &operation()).await,
            if case == "expiry" {
                "CONNECTION_LINK_EXPIRED"
            } else {
                "CONNECTION_LINK_INVALID"
            },
        );
        f.assert_unlinked();
    }
}

#[tokio::test(start_paused = true)]
async fn cancellation_deadlines_and_device_locks_refuse_without_mutation() {
    for case in ["cancel", "deadline", "lock"] {
        let f = Fixture::new(vec![]);
        let preview = f.preview().await;
        let operation = operation();
        let mut guard = None;
        match case {
            "cancel" => operation.cancellation.cancel(),
            "deadline" => tokio::time::advance(std::time::Duration::from_secs(180)).await,
            _ => guard = Some(f.store.lock_device(DEVICE).unwrap()),
        }
        error(
            f.core.confirm_connection_link(preview, &operation).await,
            match case {
                "cancel" => "CANCELLED",
                "deadline" => "DEADLINE_EXCEEDED",
                _ => "RESOURCE_BUSY",
            },
        );
        if case != "lock" {
            assert!(
                f.core
                    .prepare_connection_link(DEVICE, ANCHOR, f.connection(), &operation)
                    .await
                    .is_err()
            );
        }
        drop(guard);
        f.assert_unlinked();
    }
}

#[tokio::test]
async fn metadata_and_alias_changes_after_preview_refuse_before_commit() {
    for case in ["model", "alias"] {
        let f = Fixture::new(vec![]);
        let preview = f.preview().await;
        if case == "model" {
            let mut changed = f.store.devices().unwrap().remove(0);
            changed.device.model = "Changed-model".into();
            f.store.save_device(&changed).unwrap();
        } else {
            f.store
                .set_preferences(DEVICE, &preferences(Some("Changed name")))
                .unwrap();
        }
        error(
            f.core.confirm_connection_link(preview, &operation()).await,
            "CONNECTION_LINK_INVALID",
        );
        f.assert_unlinked();
    }
}

#[tokio::test]
async fn transaction_rechecks_changes_after_core_preflight_using_a_second_store() {
    for (fault, expected) in [
        (2, "CONNECTION_LINK_INVALID"),
        (3, "CONNECTION_LINK_INVALID"),
        (4, "ACCOUNT_REMOVAL_PENDING"),
        (5, "DEVICE_NOT_FOUND"),
    ] {
        let f = Fixture::new(vec![]);
        let preview = f.preview().await;
        f.adapter.fault.store(fault, Ordering::SeqCst);
        error(
            f.core.confirm_connection_link(preview, &operation()).await,
            expected,
        );
        if fault == 5 {
            assert!(f.store.devices().unwrap().is_empty());
        } else {
            f.assert_unlinked();
        }
    }
}

#[tokio::test]
async fn a_second_preview_cannot_append_again_and_preferences_can_change_without_loss() {
    let f = Fixture::new(vec![]);
    let first = f.preview().await;
    let second = f.preview().await;
    let before = f.store.set_preferences(DEVICE, &preferences(None)).unwrap();
    f.core
        .confirm_connection_link(first, &operation())
        .await
        .unwrap();
    error(
        f.core.confirm_connection_link(second, &operation()).await,
        "CONNECTION_LINK_INVALID",
    );
    error(
        f.core
            .prepare_connection_link(DEVICE, ANCHOR, f.connection(), &operation())
            .await,
        "CONNECTION_LINK_INVALID",
    );
    assert_eq!(f.store.preferences().unwrap(), vec![before]);
    assert_eq!(f.store.devices().unwrap()[0].device.connections.len(), 2);
}

#[tokio::test]
async fn added_transport_activates_only_existing_matching_grants() {
    for transports in [vec![Transport::Lan], vec![Transport::Cloud]] {
        let lan_allowed = transports.contains(&Transport::Lan);
        let f = Fixture::new(transports);
        let mut command: Command =
            serde_json::from_str(include_str!("../../../examples/command.json")).unwrap();
        command.device_id = DEVICE.into();
        command.capability = "power".into();
        command.arguments = json!({"state":false});
        command.connection = Route::Lan;
        error(
            f.core.plan(&command),
            if lan_allowed {
                "LAN_UNREACHABLE"
            } else {
                "POLICY_DENIED"
            },
        );
        let preview = f.preview().await;
        assert_eq!(preview.transport_grants().len(), usize::from(lan_allowed));
        if lan_allowed {
            assert_eq!(
                preview.transport_grants()[0].argument_schema,
                json!({"const":{"state":false}})
            );
        }
        f.core
            .confirm_connection_link(preview, &operation())
            .await
            .unwrap();
        if lan_allowed {
            assert_eq!(f.core.plan(&command).unwrap()["transports"], json!(["lan"]));
        } else {
            error(f.core.plan(&command), "POLICY_DENIED");
        }
        command.connection = Route::Auto;
        assert_eq!(
            f.core.plan(&command).unwrap()["transports"],
            if lan_allowed {
                json!(["lan"])
            } else {
                json!(["cloud"])
            }
        );
        command.arguments = json!({"state":true});
        error(f.core.plan(&command), "POLICY_DENIED");
    }
}

#[tokio::test]
async fn session_lifecycle_and_ordinary_json_cannot_confirm_a_link() {
    for case in ["background", "resume", "close", "confirm"] {
        let f = Fixture::new(vec![]);
        let session = Session::new(f.other_core());
        let preview = session
            .prepare_connection_link("prepare".into(), DEVICE, ANCHOR, f.connection(), 1000)
            .await
            .unwrap();
        match case {
            "background" => session.set_foreground(false),
            "resume" => {
                session.set_foreground(false);
                session.set_foreground(true);
            }
            "close" => session.close(),
            _ => {}
        }
        let result = session
            .confirm_connection_link("confirm".into(), preview, 1000)
            .await;
        if case == "confirm" {
            assert_eq!(result.unwrap().device.connections.len(), 2);
        } else {
            error(
                result,
                match case {
                    "background" => "HOST_BACKGROUND",
                    "close" => "HOST_CLOSED",
                    _ => "CONNECTION_LINK_INVALID",
                },
            );
            f.assert_unlinked();
        }
    }
    for method in ["prepare_connection_link", "confirm_connection_link"] {
        assert!(
            serde_json::from_value::<HostRequest>(json!({
                "schema_version":"1.0", "operation_id":"untrusted", "timeout_ms":1000,
                "request":{"method":method,"confirmed":true}
            }))
            .is_err()
        );
    }
}

#[tokio::test(start_paused = true)]
async fn pending_evidence_validation_is_cancelled_on_lifecycle_or_budget_expiry() {
    for stage in ["prepare", "confirm"] {
        for reason in [
            "cancel",
            "background",
            "close",
            "deadline",
            "preview_expiry",
        ] {
            let f = Fixture::new(vec![]);
            let session = Session::new(f.other_core());
            let preview = session
                .prepare_connection_link("initial".into(), DEVICE, ANCHOR, f.connection(), 1000)
                .await
                .unwrap();
            f.adapter.fault.store(6, Ordering::SeqCst);
            let work = async {
                if stage == "prepare" {
                    session
                        .prepare_connection_link(
                            "pending".into(),
                            DEVICE,
                            ANCHOR,
                            f.connection(),
                            180_000,
                        )
                        .await
                        .map(|_| ())
                } else {
                    session
                        .confirm_connection_link("pending".into(), preview, 180_000)
                        .await
                        .map(|_| ())
                }
            };
            let interrupt = async {
                f.adapter.started.notified().await;
                match reason {
                    "cancel" => {
                        assert!(session.cancel("pending"));
                    }
                    "background" => {
                        session.set_foreground(false);
                        session.set_foreground(true);
                    }
                    "close" => session.close(),
                    "preview_expiry" if stage == "confirm" => {
                        tokio::time::advance(std::time::Duration::from_secs(60)).await;
                    }
                    _ => tokio::time::advance(std::time::Duration::from_secs(180)).await,
                }
            };
            let (result, ()) = tokio::join!(work, interrupt);
            error(
                result,
                if matches!(reason, "deadline" | "preview_expiry") {
                    "DEADLINE_EXCEEDED"
                } else {
                    "CANCELLED"
                },
            );
            f.assert_unlinked();
            assert!(!session.cancel("pending"));
            let _released = f.store.lock_device(DEVICE).unwrap();
        }
    }
}
