//! Trusted local connection association; never ordinary discovery or command input.

use crate::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Display before trusted UI consent. Dropping the preview dismisses it.
/// ```compile_fail
/// fn export(value: &homeverb_core::PendingConnectionLink) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
/// ```compile_fail
/// fn duplicate(value: homeverb_core::PendingConnectionLink) {
///     let _ = value.clone();
/// }
/// ```
pub struct PendingConnectionLink {
    descriptor: Descriptor,
    anchor: Connection,
    connection: Connection,
    transport_grants: Vec<Grant>,
    context: Arc<AtomicU64>,
    generation: u64,
    deadline: Instant,
    expires_at: DateTime<Utc>,
}

impl PendingConnectionLink {
    pub fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
    pub fn anchor(&self) -> &Connection {
        &self.anchor
    }
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
    /// Existing potential authority for this transport; command validation still applies.
    pub fn transport_grants(&self) -> &[Grant] {
        &self.transport_grants
    }
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }
    fn validate(&self, core: &Core) -> Result<()> {
        if !Arc::ptr_eq(&self.context, &core.approval_context)
            || self.generation != self.context.load(Ordering::SeqCst)
        {
            return Err(invalid());
        }
        if self.is_expired() {
            return Err(Error::new(
                "CONNECTION_LINK_EXPIRED",
                "The connection preview expired; review it again before confirming.",
            ));
        }
        Ok(())
    }
}

/// Constructed only after Core consumes a valid trusted confirmation.
/// Store implementations must compare current effective metadata in their transaction.
/// ```compile_fail
/// use homeverb_core::ValidatedConnectionLink;
/// use homeverb_domain::{Connection, Descriptor};
/// fn forge(descriptor: Descriptor, connection: Connection) -> ValidatedConnectionLink {
///     ValidatedConnectionLink { descriptor, connection }
/// }
/// ```
pub struct ValidatedConnectionLink {
    descriptor: Descriptor,
    connection: Connection,
}

impl ValidatedConnectionLink {
    pub fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
    pub fn validate_current(&self, current: &Descriptor) -> Result<()> {
        same_descriptor(&self.descriptor, current)
    }
}

pub(crate) fn unsupported() -> Error {
    Error::new(
        "CONNECTION_LINK_UNSUPPORTED",
        "This adapter or store does not support trusted connection linking.",
    )
}
fn invalid() -> Error {
    Error::new(
        "CONNECTION_LINK_INVALID",
        "The connection or confirmation scope is invalid; review the saved device again.",
    )
}
fn same_descriptor(expected: &Descriptor, current: &Descriptor) -> Result<()> {
    if serde_json::to_value(expected).map_err(|_| invalid())?
        != serde_json::to_value(current).map_err(|_| invalid())?
    {
        return Err(invalid());
    }
    Ok(())
}

impl Core {
    async fn validate_link(
        &self,
        descriptor: &Descriptor,
        anchor_id: &str,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Connection> {
        let anchor = descriptor
            .device
            .connections
            .iter()
            .find(|value| value.id == anchor_id)
            .ok_or_else(invalid)?;
        if anchor.adapter != connection.adapter
            || anchor.account_id != connection.account_id
            || anchor.identity_ref != connection.identity_ref
            || descriptor
                .device
                .connections
                .iter()
                .any(|value| value.id == connection.id || value.transport == connection.transport)
        {
            return Err(invalid());
        }
        let mut linked = descriptor.clone();
        linked.device.connections.push(connection.clone());
        crate::descriptors::validate(&linked)?;
        self.adapters
            .iter()
            .find(|adapter| adapter.id() == connection.adapter)
            .ok_or_else(unsupported)?
            .validate_connection_link(descriptor, connection, operation)
            .await?;
        Ok(anchor.clone())
    }

    /// Trusted native callers only. No network access or persistence; this is not consent.
    pub async fn prepare_connection_link(
        &self,
        device_id: &str,
        anchor_id: &str,
        connection: Connection,
        operation: &Operation,
    ) -> Result<PendingConnectionLink> {
        let preview = operation
            .run(async {
                let generation = self.approval_context.load(Ordering::SeqCst);
                let descriptor = self.describe(device_id)?;
                if descriptor.device.id != device_id {
                    return Err(Error::invalid());
                }
                let anchor = self
                    .validate_link(&descriptor, anchor_id, &connection, operation)
                    .await?;
                let transport_grants = self
                    .policy
                    .grants
                    .iter()
                    .filter(|grant| {
                        grant.device_id == device_id
                            && grant.transports.contains(&connection.transport)
                    })
                    .cloned()
                    .collect();
                Ok(PendingConnectionLink {
                    descriptor,
                    anchor,
                    connection,
                    transport_grants,
                    context: self.approval_context.clone(),
                    generation,
                    deadline: Instant::now() + Duration::from_secs(60),
                    expires_at: self.clock.now() + chrono::Duration::seconds(60),
                })
            })
            .await?;
        operation.run(async { preview.validate(self) }).await?;
        Ok(preview)
    }

    /// Consume once, only after actual trusted UI consent to this exact preview.
    /// Cancellation after storage admission cannot roll back a completed local commit.
    pub async fn confirm_connection_link(
        &self,
        preview: PendingConnectionLink,
        operation: &Operation,
    ) -> Result<Descriptor> {
        operation.run(async { preview.validate(self) }).await?;
        let operation = Operation {
            cancellation: operation.cancellation.clone(),
            deadline: operation.deadline.min(preview.deadline),
        };
        let _lock = operation
            .run(async { self.store.lock_device(&preview.descriptor.device.id) })
            .await?;
        operation
            .run(async {
                preview.validate(self)?;
                let current = self.describe(&preview.descriptor.device.id)?;
                same_descriptor(&preview.descriptor, &current)?;
                self.validate_link(
                    &current,
                    &preview.anchor.id,
                    &preview.connection,
                    &operation,
                )
                .await?;
                Ok(())
            })
            .await?;
        operation
            .run(async {
                preview.validate(self)?;
                self.store.link_connection(ValidatedConnectionLink {
                    descriptor: preview.descriptor,
                    connection: preview.connection,
                })
            })
            .await
    }
}
