use crate::{Core, Operation};
use homeverb_domain::*;

fn unavailable(_: Error) -> Error {
    Error::new(
        "STORAGE_ERROR",
        "Local diagnostics could not read the profile.",
    )
}

impl Core {
    /// Build a local allowlisted report without reading device state or credentials.
    pub async fn diagnostics(&self, operation: &Operation) -> Result<Diagnostics> {
        let devices = operation
            .run(async { self.store.devices().map_err(unavailable) })
            .await?;
        let rooms = operation
            .run(async { self.store.rooms().map_err(unavailable) })
            .await?;
        let history = operation
            .run(async { self.store.recent_executions().map_err(unavailable) })
            .await?;
        // Recheck after synchronous storage, which cannot itself be preempted.
        operation
            .run(async {
                let mut connections = DiagnosticConnections::default();
                for device in &devices {
                    for connection in &device.device.connections {
                        match connection.transport {
                            Transport::Lan => connections.lan += 1,
                            Transport::Cloud => connections.cloud += 1,
                        }
                    }
                }
                Ok(Diagnostics {
                    schema_version: SCHEMA_VERSION,
                    application_version: env!("CARGO_PKG_VERSION"),
                    generated_at: self.clock.now(),
                    scope: "local_profile",
                    device_count: devices.len(),
                    room_count: rooms.len(),
                    connections,
                    receipt_limit: DIAGNOSTIC_RECEIPT_LIMIT,
                    attempt_limit: DIAGNOSTIC_ATTEMPT_LIMIT,
                    recent_receipts: history
                        .iter()
                        .take(DIAGNOSTIC_RECEIPT_LIMIT)
                        .map(summarize)
                        .collect(),
                })
            })
            .await
    }
}

fn summarize(execution: &Execution) -> DiagnosticReceipt {
    let mut item_outcomes = DiagnosticOutcomes::default();
    for item in execution.items.iter().flatten() {
        match item.status {
            Status::Accepted => item_outcomes.accepted += 1,
            Status::Verified => item_outcomes.verified += 1,
            Status::Failed => item_outcomes.failed += 1,
            Status::Unknown => item_outcomes.unknown += 1,
            Status::Partial => item_outcomes.partial += 1,
        }
    }
    DiagnosticReceipt {
        status: execution.status,
        error_code: execution
            .error
            .as_ref()
            .map(|error| DiagnosticErrorCode::from_code(&error.code)),
        verification_kind: execution
            .verification
            .as_ref()
            .map(|evidence| evidence.kind),
        attempts: execution
            .attempts
            .iter()
            .take(DIAGNOSTIC_ATTEMPT_LIMIT)
            .map(|attempt| DiagnosticAttempt {
                transport: attempt.transport,
                phase: attempt.phase,
                duration_ms: attempt.duration_ms,
            })
            .collect(),
        attempts_omitted: execution
            .attempts
            .len()
            .saturating_sub(DIAGNOSTIC_ATTEMPT_LIMIT),
        warning_count: execution.warnings.len(),
        item_outcomes,
    }
}
