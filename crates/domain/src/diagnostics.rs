//! Allowlisted diagnostic output, independent of private domain-record serialization.

use crate::{Phase, Status, Transport, VerificationMethod};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const DIAGNOSTIC_RECEIPT_LIMIT: usize = 100;
pub const DIAGNOSTIC_ATTEMPT_LIMIT: usize = 32;

#[derive(Debug, Serialize)]
pub struct Diagnostics {
    pub schema_version: &'static str,
    pub application_version: &'static str,
    pub generated_at: DateTime<Utc>,
    pub scope: &'static str,
    pub device_count: usize,
    pub room_count: usize,
    pub connections: DiagnosticConnections,
    pub receipt_limit: usize,
    pub attempt_limit: usize,
    pub recent_receipts: Vec<DiagnosticReceipt>,
}

#[derive(Debug, Default, Serialize)]
pub struct DiagnosticConnections {
    pub lan: usize,
    pub cloud: usize,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticReceipt {
    pub status: Status,
    pub error_code: Option<DiagnosticErrorCode>,
    pub verification_kind: Option<VerificationMethod>,
    pub attempts: Vec<DiagnosticAttempt>,
    pub attempts_omitted: usize,
    pub warning_count: usize,
    pub item_outcomes: DiagnosticOutcomes,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticAttempt {
    pub transport: Transport,
    pub phase: Phase,
    pub duration_ms: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct DiagnosticOutcomes {
    pub accepted: usize,
    pub verified: usize,
    pub failed: usize,
    pub unknown: usize,
    pub partial: usize,
}

/// Unknown adapter-provided codes never pass through as arbitrary strings.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosticErrorCode {
    AuthRequired,
    AuthExpired,
    AuthFlowUnsupported,
    SecretStoreUnavailable,
    SecretStoreLocked,
    SecretStoreConflict,
    SecretStoreWriteUnknown,
    DeviceNotFound,
    DeviceIdentityConflict,
    AmbiguousDevice,
    RoomNotFound,
    RoomNameConflict,
    CapabilityUnsupported,
    InvalidArgument,
    PolicyDenied,
    LanUnreachable,
    CloudUnreachable,
    ReadTimeout,
    RateLimited,
    ResourceBusy,
    DeviceRejected,
    WriteOutcomeUnknown,
    VerificationPending,
    PartialExecution,
    Cancelled,
    DeadlineExceeded,
    CacheMiss,
    InvalidObservation,
    RequestConflict,
    RequestNotFound,
    RequestRetired,
    NotExecuted,
    StorageError,
    MockStorageError,
    StorageVersionUnsupported,
    HostBackground,
    HostClosed,
    InternalError,
    McpIoError,
    Other,
}

impl DiagnosticErrorCode {
    pub fn from_code(code: &str) -> Self {
        serde_json::from_value(serde_json::Value::String(code.into())).unwrap_or(Self::Other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_allowlist_matches_the_closed_report_schema() {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../schemas/diagnostics.schema.json")).unwrap();
        for code in schema["$defs"]["receipt"]["properties"]["error_code"]["enum"]
            .as_array()
            .unwrap()
        {
            if let Some(code) = code.as_str() {
                assert_eq!(
                    serde_json::to_value(DiagnosticErrorCode::from_code(code)).unwrap(),
                    code
                );
            }
        }
        assert_eq!(
            serde_json::to_value(DiagnosticErrorCode::from_code("unreviewed-private-code"))
                .unwrap(),
            "OTHER"
        );
    }
}
