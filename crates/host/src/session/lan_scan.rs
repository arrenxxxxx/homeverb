use super::*;
use homeverb_midea::{LanScanOutcome, lan::DiscoveryNetwork};
use std::{fmt, sync::Weak, time::Duration};
use tokio::time::Instant;

/// Closed metadata for the separate trusted scan bridge, never an ordinary request.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareLanScanRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub timeout_ms: u64,
    pub device_id: String,
    pub connection_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmLanScanRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub timeout_ms: u64,
    pub scan_id: u64,
}

/// Show the immutable device and network before trusted foreground scan consent.
/// ```compile_fail
/// fn export(value: &homeverb_host::PendingMeijuLanScan) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
/// ```compile_fail
/// fn duplicate(value: homeverb_host::PendingMeijuLanScan) {
///     let _ = value.clone();
/// }
/// ```
pub struct PendingMeijuLanScan {
    descriptor: Descriptor,
    connection: Connection,
    network: DiscoveryNetwork,
    state: Weak<Mutex<State>>,
    generation: u64,
    deadline: Instant,
}

impl fmt::Debug for PendingMeijuLanScan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PendingMeijuLanScan([REDACTED])")
    }
}

fn invalid() -> Error {
    Error::new(
        "LAN_SCAN_INVALID",
        "The device or network consent context changed; review a new scan preview.",
    )
}

impl PendingMeijuLanScan {
    pub fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
    pub fn network(&self) -> &DiscoveryNetwork {
        &self.network
    }
    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }
    fn validate(&self, session: &Session) -> Result<()> {
        if !self.state.ptr_eq(&Arc::downgrade(&session.state)) {
            return Err(invalid());
        }
        let state = session.state.lock().map_err(|_| invalid())?;
        if state.closed || !state.foreground || state.approval_generation != self.generation {
            return Err(invalid());
        }
        if self.is_expired() {
            return Err(Error::new(
                "LAN_SCAN_EXPIRED",
                "The network scan preview expired; review it again before confirming.",
            ));
        }
        Ok(())
    }
    fn validate_descriptor(&self, session: &Session) -> Result<()> {
        let current = session.core.describe(&self.descriptor.device.id)?;
        if serde_json::to_value(current).map_err(|_| invalid())?
            != serde_json::to_value(&self.descriptor).map_err(|_| invalid())?
        {
            return Err(invalid());
        }
        Ok(())
    }
}

impl Session {
    /// Trusted controller only: the platform must supply an assigned interface and
    /// permission. The address is the numeric IPv4 octets, not native-memory bytes.
    pub async fn prepare_native_lan_scan(
        &self,
        input: PrepareLanScanRequest,
        local_address: u32,
        prefix_length: u8,
    ) -> Value {
        let result = async {
            if input.schema_version != SCHEMA_VERSION {
                return Err(Error::invalid());
            }
            let network = DiscoveryNetwork::new(local_address.into(), prefix_length)?;
            let preview = self
                .prepare_meiju_lan_scan(
                    input.operation_id,
                    &input.device_id,
                    &input.connection_id,
                    network,
                    input.timeout_ms,
                )
                .await?;
            let mut state = self.state.lock().map_err(|_| invalid())?;
            if state.closed || !state.foreground || state.approval_generation != preview.generation
            {
                return Err(invalid());
            }
            state.lan_scans.retain(|_, value| !value.is_expired());
            if state.lan_scans.len() >= 8 {
                return Err(Error::new(
                    "RESOURCE_BUSY",
                    "Dismiss a pending scan before preparing another.",
                ));
            }
            let id = NEXT_PREVIEW_ID.fetch_add(1, Ordering::Relaxed);
            let value = json!({
                "scan_id":id, "device_id":preview.descriptor.device.id,
                "connection_id":preview.connection.id, "alias":preview.descriptor.device.alias,
                "network":{"local_address":preview.network.local_address().to_string(),
                           "prefix_length":preview.network.prefix_length()},
                "expires_in_ms":preview.remaining().as_millis() as u64
            });
            state.lan_scans.insert(id, preview);
            Ok(value)
        }
        .await;
        json!(Query::from_result(result))
    }

    /// Call only after actual user consent and rechecking platform network permission.
    /// Consume the pending scope even when decoded metadata fails admission.
    pub async fn confirm_native_lan_scan(&self, input: ConfirmLanScanRequest) -> Value {
        let result = async {
            let preview = self
                .state
                .lock()
                .map_err(|_| invalid())?
                .lan_scans
                .remove(&input.scan_id)
                .ok_or_else(invalid)?;
            if input.schema_version != SCHEMA_VERSION {
                return Err(Error::invalid());
            }
            let device_id = preview.descriptor.device.id.clone();
            let connection_id = preview.connection.id.clone();
            let outcome = self
                .scan_meiju_lan_candidate(input.operation_id, preview, input.timeout_ms)
                .await?;
            let status = match outcome {
                LanScanOutcome::NotObserved => "not_observed",
                LanScanOutcome::Incomplete => "incomplete",
                LanScanOutcome::Ambiguous => "ambiguous",
                LanScanOutcome::UniqueCandidate(_) => "unique_candidate",
            };
            Ok(
                json!({"device_id":device_id, "connection_id":connection_id, "status":status,
                      "authenticated":false, "route_added":false}),
            )
        }
        .await;
        json!(Query::from_result(result))
    }

    /// Drop pending consent only; use operation cancellation for an active scan.
    pub fn discard_native_lan_scan(&self, scan_id: u64) -> bool {
        self.state
            .lock()
            .ok()
            .is_some_and(|mut state| state.lan_scans.remove(&scan_id).is_some())
    }

    /// Trusted native host only. The host selects an assigned interface and handles
    /// platform permission. This performs protected local preflight, never a scan.
    pub async fn prepare_meiju_lan_scan(
        &self,
        operation_id: String,
        device_id: &str,
        connection_id: &str,
        network: DiscoveryNetwork,
        timeout_ms: u64,
    ) -> Result<PendingMeijuLanScan> {
        let (operation, _registration) = self.register(SCHEMA_VERSION, operation_id, timeout_ms)?;
        let generation = self
            .state
            .lock()
            .map_err(|_| invalid())?
            .approval_generation;
        let descriptor = operation
            .run(async { self.core.describe(device_id) })
            .await?;
        if descriptor.device.id != device_id {
            return Err(Error::invalid());
        }
        let connection = descriptor
            .device
            .connections
            .iter()
            .find(|value| value.id == connection_id)
            .cloned()
            .ok_or_else(invalid)?;
        let adapter = self
            .accounts
            .as_ref()
            .ok_or_else(crate::accounts::unsupported)?;
        operation
            .run(adapter.validate_lan_scan(&descriptor.device, &connection, &operation))
            .await?;
        let preview = PendingMeijuLanScan {
            descriptor,
            connection,
            network,
            state: Arc::downgrade(&self.state),
            generation,
            deadline: Instant::now() + Duration::from_secs(60),
        };
        operation.run(async { preview.validate(self) }).await?;
        Ok(preview)
    }

    /// Consume once, only after actual trusted UI consent. This sends only bounded
    /// discovery probes; it never fetches keys, reads state or links a connection.
    pub async fn scan_meiju_lan_candidate(
        &self,
        operation_id: String,
        preview: PendingMeijuLanScan,
        timeout_ms: u64,
    ) -> Result<LanScanOutcome> {
        let (mut operation, _registration) =
            self.register(SCHEMA_VERSION, operation_id, timeout_ms)?;
        operation.run(async { preview.validate(self) }).await?;
        operation.deadline = operation.deadline.min(preview.deadline);
        operation
            .run(async { preview.validate_descriptor(self) })
            .await?;
        let adapter = self
            .accounts
            .as_ref()
            .ok_or_else(crate::accounts::unsupported)?;
        let outcome = operation
            .run(adapter.scan_lan_candidate(
                &preview.descriptor.device,
                &preview.connection,
                &preview.network,
                &operation,
            ))
            .await?;
        operation
            .run(async {
                preview.validate(self)?;
                preview.validate_descriptor(self)?;
                Ok(outcome)
            })
            .await
    }
}
