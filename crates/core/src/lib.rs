//! Embedded execution policy shared by every host.

mod account_removal;
mod approval;
pub use approval::PendingApproval;
mod accounts;
mod batch;
mod connection_link;
pub use connection_link::{PendingConnectionLink, ValidatedConnectionLink};
mod descriptors;
mod diagnostics;
mod execution;
mod observations;
mod organization;
mod policy;
mod secrets;
pub use account_removal::*;
pub use accounts::*;
pub use execution::Core;
pub use policy::{Grant, Policy};
pub use secrets::{CredentialRef, SecretBytes, SecretStore, UnavailableSecretStore};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use homeverb_domain::*;
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Clone)]
pub struct Operation {
    pub cancellation: CancellationToken,
    pub deadline: Instant,
}

impl Operation {
    pub fn new(timeout_ms: u64, cancellation: CancellationToken) -> Result<Self> {
        if !(1..=180_000).contains(&timeout_ms) {
            return Err(Error::invalid());
        }
        Ok(Self {
            cancellation,
            deadline: Instant::now() + Duration::from_millis(timeout_ms),
        })
    }
    pub async fn run<T>(&self, work: impl Future<Output = Result<T>>) -> Result<T> {
        // Synchronous validation/storage may expire the budget before the timer driver runs.
        if self.cancellation.is_cancelled() {
            return Err(Error::new("CANCELLED", "The operation was cancelled."));
        }
        if Instant::now() >= self.deadline {
            return Err(Error::new(
                "DEADLINE_EXCEEDED",
                "The total operation budget expired.",
            ));
        }
        tokio::select! {
            biased;
            _ = self.cancellation.cancelled() => Err(Error::new("CANCELLED", "The operation was cancelled.")),
            _ = tokio::time::sleep_until(self.deadline) => Err(Error::new("DEADLINE_EXCEEDED", "The total operation budget expired.")),
            result = work => result,
        }
    }
}

pub struct ValidatedCommand {
    command: Command,
    user_approved: bool,
}
impl ValidatedCommand {
    pub fn command(&self) -> &Command {
        &self.command
    }
    /// Set only by Core after consuming a valid trusted user approval.
    pub fn has_user_approval(&self) -> bool {
        self.user_approved
    }
}

/// The adapter marks transmission immediately before the first possible send.
#[derive(Clone, Default)]
pub struct WriteProgress(Arc<AtomicU8>);
impl WriteProgress {
    pub fn mark_sent(&self) {
        self.0.fetch_max(1, Ordering::SeqCst);
    }
    pub fn mark_accepted(&self) {
        self.0.fetch_max(2, Ordering::SeqCst);
    }
    pub fn phase(&self) -> Phase {
        match self.0.load(Ordering::SeqCst) {
            0 => Phase::NotSent,
            1 => Phase::PossiblySent,
            _ => Phase::Accepted,
        }
    }
}

pub struct AdapterOutcome {
    pub phase: Phase,
    pub error: Option<Error>,
    pub completion: Option<CorrelatedCompletion>,
}

/// Successful completion of the entire command, correlated by the adapter's protocol.
/// Never construct this from an enqueue acknowledgement or a local request ID alone.
pub struct CorrelatedCompletion {
    pub request_id: String,
    pub device_id: String,
    pub component_id: String,
    pub connection_id: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub received_at: DateTime<Utc>,
}

#[async_trait]
pub trait Adapter: Send + Sync {
    fn id(&self) -> &str;
    /// Trusted local evidence only: no network I/O. Opt-in requires model/identity admission
    /// and discovery that retains the complete established connection set.
    async fn validate_connection_link(
        &self,
        _: &Descriptor,
        _: &Connection,
        _: &Operation,
    ) -> Result<()> {
        Err(connection_link::unsupported())
    }
    async fn discover(&self, operation: &Operation) -> Result<Vec<Descriptor>>;
    async fn read(
        &self,
        device: &Device,
        component: &str,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Snapshot>;
    async fn execute(
        &self,
        command: &ValidatedCommand,
        connection: &Connection,
        progress: WriteProgress,
        operation: &Operation,
    ) -> Result<AdapterOutcome>;
}

pub trait DeviceLock: Send {}
impl<T: Send> DeviceLock for T {}

pub trait Store: Send + Sync {
    fn devices(&self) -> Result<Vec<Descriptor>>;
    /// Compare established identities and reject removed-account bindings in the save transaction.
    fn save_device(&self, descriptor: &Descriptor) -> Result<()>;
    /// Atomically compare the effective descriptor and append only the validated connection.
    fn link_connection(&self, _: ValidatedConnectionLink) -> Result<Descriptor> {
        Err(connection_link::unsupported())
    }
    fn snapshot(&self, device: &str, component: &str) -> Result<Option<Snapshot>>;
    /// Require a still-present device and reject removed-account bindings in the save transaction.
    fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()>;
    /// Delete currently cached snapshots without cancelling concurrent reads.
    fn clear_cached_state(&self) -> Result<usize>;
    fn lock_device(&self, device: &str) -> Result<Box<dyn DeviceLock>>;
    fn execution(&self, request_id: &str) -> Result<Option<(Command, Execution)>>;
    fn save_execution(&self, command: &Command, execution: &Execution) -> Result<()>;
    fn finish_execution(&self, command: &Command, execution: &Execution) -> Result<()>;
    fn recent_executions(&self) -> Result<Vec<Execution>>;
    /// Refuse active journals; atomically retire all detailed receipts and preserve request IDs.
    fn clear_execution_history(&self) -> Result<usize>;
    fn device_executions(&self, device_id: &str) -> Result<Vec<Execution>>;
    fn rooms(&self) -> Result<Vec<Room>>;
    fn create_room(&self, id: &str, name: &str) -> Result<Room>;
    fn rename_room(&self, id: &str, name: &str) -> Result<Room>;
    fn delete_room(&self, id: &str) -> Result<()>;
    fn reorder_rooms(&self, ids: &[String]) -> Result<Vec<Room>>;
    fn preferences(&self) -> Result<Vec<DevicePreferences>>;
    fn device_cards(&self) -> Result<Vec<DeviceCard>>;
    fn set_preferences(&self, device: &str, input: &PreferencesInput) -> Result<DevicePreferences>;
    fn reorder_devices(&self, ids: &[String]) -> Result<Vec<DevicePreferences>>;
    fn assign_devices_to_room(&self, input: &RoomAssignment) -> Result<Vec<DevicePreferences>>;
}
