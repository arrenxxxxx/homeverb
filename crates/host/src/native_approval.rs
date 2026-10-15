//! Closed metadata for trusted native approval exports, not ordinary host requests.

use homeverb_domain::Command;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareApprovalRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub command: Command,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecuteApprovalRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub approval_id: u64,
}
