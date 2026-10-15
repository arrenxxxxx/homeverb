//! Versioned, host-independent contracts. No device I/O or persistence lives here.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

mod organization;
pub use organization::*;
mod diagnostics;
pub use diagnostics::*;
mod batch;
pub use batch::*;

pub const SCHEMA_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Error {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl Error {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable: false,
        }
    }
    pub fn invalid() -> Self {
        Self::new(
            "INVALID_ARGUMENT",
            "The request does not satisfy the supported contract.",
        )
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Lan,
    Cloud,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    Auto,
    Lan,
    Cloud,
}

impl Route {
    pub fn permits(self, transport: Transport) -> bool {
        matches!(
            (self, transport),
            (Self::Auto, _) | (Self::Lan, Transport::Lan) | (Self::Cloud, Transport::Cloud)
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WaitFor {
    Accepted,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub schema_version: String,
    pub request_id: String,
    pub device_id: String,
    pub component_id: String,
    pub capability: String,
    pub action: String,
    pub arguments: Value,
    pub connection: Route,
    pub timeout_ms: u64,
    pub wait_for: WaitFor,
}

impl Command {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != SCHEMA_VERSION
            || !(1..=180_000).contains(&self.timeout_ms)
            || [
                &self.request_id,
                &self.device_id,
                &self.component_id,
                &self.capability,
                &self.action,
            ]
            .iter()
            .any(|v| v.trim().is_empty() || v.len() > 256)
            || !self.arguments.is_object()
        {
            return Err(Error::invalid());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub schema_version: String,
    pub id: String,
    pub alias: String,
    pub brand: String,
    pub model: String,
    pub components: Vec<Component>,
    pub connections: Vec<Connection>,
    pub support_status: String,
}

impl Device {
    /// Compare established identity only; Core separately validates descriptor shape.
    pub fn validate_rediscovery(&self, incoming: &Self) -> Result<()> {
        let mut previous: Vec<_> = self.connections.iter().collect();
        let mut current: Vec<_> = incoming.connections.iter().collect();
        previous.sort_by(|left, right| left.id.cmp(&right.id));
        current.sort_by(|left, right| left.id.cmp(&right.id));
        if self.id != incoming.id || previous != current {
            return Err(Error::new(
                "DEVICE_IDENTITY_CONFLICT",
                "Discovery cannot replace the established connection identity of this device.",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    pub id: String,
    pub adapter: String,
    pub transport: Transport,
    pub account_id: Option<String>,
    pub identity_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capability {
    pub schema_version: String,
    pub id: String,
    /// Omitted for a shared definition; present only for one exact component.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "component_scope"
    )]
    pub component_id: Option<String>,
    pub revision: u32,
    pub properties: BTreeMap<String, Value>,
    pub actions: BTreeMap<String, Action>,
    pub events: BTreeMap<String, Value>,
}

fn component_scope<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error> {
    // Explicit null must not erase a component binding and become shared metadata.
    String::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub parameters: Value,
    pub transports: Vec<Transport>,
    pub idempotency: Idempotency,
    pub risk: Risk,
    pub atomic: bool,
    /// Explicit adapter declaration that setter fields may be applied independently.
    #[serde(default)]
    pub independent_fields: bool,
    pub verification: VerificationMethod,
}

impl Action {
    pub fn validate_arguments(&self, arguments: &Value) -> Result<()> {
        // External schema resolution is disabled by the dependency configuration.
        let validator = jsonschema::validator_for(&self.parameters).map_err(|_| {
            Error::new(
                "CAPABILITY_UNSUPPORTED",
                "The action schema cannot be evaluated locally.",
            )
        })?;
        if !validator.is_valid(arguments) {
            return Err(Error::invalid());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Idempotency {
    ReadOnly,
    SetValue,
    NonIdempotent,
    Unknown,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    Low,
    Elevated,
    High,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    StateMatch,
    CorrelatedCompletion,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema_version: String,
    pub device_id: String,
    pub component_id: String,
    pub from_cache: bool,
    pub observations: BTreeMap<String, Observation>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub value: Value,
    pub source: Transport,
    pub observed_at: Option<DateTime<Utc>>,
    pub received_at: DateTime<Utc>,
    pub stale: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Accepted,
    Verified,
    Failed,
    Unknown,
    Partial,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    NotSent,
    PossiblySent,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub transport: Transport,
    pub phase: Phase,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    pub kind: VerificationMethod,
    pub source: Transport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<DateTime<Utc>>,
    pub received_at: DateTime<Utc>,
    pub matches_target: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Execution {
    pub schema_version: String,
    pub request_id: String,
    pub device_id: String,
    pub status: Status,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub data: Option<Value>,
    pub error: Option<Error>,
    pub verification: Option<Verification>,
    pub attempts: Vec<Attempt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Execution>>,
}

impl Execution {
    pub fn new(command: &Command, now: DateTime<Utc>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION.into(),
            request_id: command.request_id.clone(),
            device_id: command.device_id.clone(),
            status: Status::Failed,
            started_at: now,
            completed_at: now,
            data: None,
            error: None,
            verification: None,
            attempts: Vec::new(),
            warnings: Vec::new(),
            items: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Descriptor {
    pub device: Device,
    pub capabilities: Vec<Capability>,
}

impl Descriptor {
    /// Resolve a declared component reference without guessing among ambiguous definitions.
    pub fn capability(&self, component_id: &str, id: &str) -> Option<&Capability> {
        let mut components = self
            .device
            .components
            .iter()
            .filter(|c| c.id == component_id);
        let component = components.next()?;
        if components.next().is_some()
            || !component
                .capabilities
                .iter()
                .any(|reference| reference == id)
        {
            return None;
        }
        let mut shared = false;
        let mut scoped = false;
        let mut selected = None;
        for capability in self.capabilities.iter().filter(|c| c.id == id) {
            shared |= capability.component_id.is_none();
            scoped |= capability.component_id.is_some();
            if capability
                .component_id
                .as_deref()
                .is_none_or(|scope| scope == component_id)
            {
                if selected.is_some() {
                    return None;
                }
                selected = Some(capability);
            }
        }
        if shared && scoped { None } else { selected }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Query<T> {
    pub schema_version: String,
    pub data: Option<T>,
    pub warnings: Vec<String>,
    pub error: Option<Error>,
}

impl<T> Query<T> {
    pub fn from_result(result: Result<T>) -> Self {
        match result {
            Ok(data) => Self {
                schema_version: SCHEMA_VERSION.into(),
                data: Some(data),
                warnings: vec![],
                error: None,
            },
            Err(error) => Self {
                schema_version: SCHEMA_VERSION.into(),
                data: None,
                warnings: vec![],
                error: Some(error),
            },
        }
    }
}
