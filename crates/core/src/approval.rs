//! Trusted in-process user confirmation; never ordinary command or tool input.

use crate::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// A preview to show before trusted user consent, not proof that consent occurred.
/// ```compile_fail
/// fn export(value: &homeverb_core::PendingApproval) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
/// ```compile_fail
/// fn duplicate(value: homeverb_core::PendingApproval) {
///     let _ = value.clone();
/// }
/// ```
pub struct PendingApproval {
    command: Command,
    descriptor: Descriptor,
    action: Action,
    connection: Connection,
    context: Arc<AtomicU64>,
    generation: u64,
    pub(crate) deadline: Instant,
    expires_at: DateTime<Utc>,
}

impl PendingApproval {
    pub fn command(&self) -> &Command {
        &self.command
    }
    pub fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
    pub fn action(&self) -> &Action {
        &self.action
    }
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    /// Monotonic expiry for bounded host registries; never establishes permission.
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }

    pub(crate) fn validate(&self, core: &Core, descriptor: &Descriptor) -> Result<()> {
        if !Arc::ptr_eq(&self.context, &core.approval_context)
            || self.generation != self.context.load(Ordering::SeqCst)
            || serde_json::to_value(descriptor).map_err(|_| Error::invalid())?
                != serde_json::to_value(&self.descriptor).map_err(|_| Error::invalid())?
        {
            return Err(invalid());
        }
        if self.is_expired() {
            return Err(Error::new(
                "APPROVAL_EXPIRED",
                "The prepared approval expired; review a new preview before confirming.",
            ));
        }
        Ok(())
    }
}

pub(crate) fn denied() -> Error {
    Error::new(
        "POLICY_DENIED",
        "Trusted approval requires an elevated action, a complete matching grant and one explicit connection.",
    )
}

pub(crate) fn invalid() -> Error {
    Error::new(
        "APPROVAL_INVALID",
        "The prepared approval no longer matches this host or device scope.",
    )
}

impl Core {
    /// Trusted host only. Display the immutable preview before asking for user consent.
    pub fn prepare_user_approval(&self, command: Command) -> Result<PendingApproval> {
        let generation = self.approval_context.load(Ordering::SeqCst);
        let (descriptor, action) = self.validate_elevated(&command)?;
        if self.store.execution(&command.request_id)?.is_some() {
            return Err(Error::new(
                "REQUEST_CONFLICT",
                "This request ID already has a receipt; inspect it without another approval.",
            ));
        }
        let mut routes = self.routes(&descriptor.device, command.connection, Some(&action));
        if command.connection == Route::Auto || routes.len() != 1 {
            return Err(denied());
        }
        Ok(PendingApproval {
            command,
            descriptor,
            action,
            connection: routes.remove(0),
            context: self.approval_context.clone(),
            generation,
            deadline: Instant::now() + Duration::from_secs(60),
            expires_at: self.clock.now() + chrono::Duration::seconds(60),
        })
    }

    /// Revoke pending previews. Hosts must also cancel any in-flight operation tokens.
    pub fn invalidate_user_approvals(&self) {
        self.approval_context.fetch_add(1, Ordering::SeqCst);
    }
}
