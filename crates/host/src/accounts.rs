//! Public account projections contain no credential references or provider identity material.

use homeverb_core::{AccountRecord, Core, CredentialState, Operation};
use homeverb_domain::{Error, Result};
use homeverb_midea::AccountDiscovery;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub timeout_ms: u64,
    pub label: String,
    #[serde(default)]
    pub account_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct AccountSummary {
    id: Uuid,
    provider: String,
    region: String,
    label: String,
    credential_state: CredentialState,
    authorization: &'static str,
}

impl From<AccountRecord> for AccountSummary {
    fn from(record: AccountRecord) -> Self {
        Self {
            id: record.id,
            provider: record.provider,
            region: record.region,
            label: record.label,
            credential_state: record.credential_state,
            authorization: "not_checked",
        }
    }
}

pub(crate) async fn details(
    core: &Core,
    record: AccountRecord,
    operation: &Operation,
) -> Result<serde_json::Value> {
    operation
        .run(async {
            let id = record.id.to_string();
            let mut device_ids: Vec<_> = core
                .list()?
                .into_iter()
                .filter(|device| {
                    device
                        .connections
                        .iter()
                        .any(|connection| connection.account_id.as_deref() == Some(id.as_str()))
                })
                .map(|device| device.id)
                .collect();
            device_ids.sort();
            Ok(serde_json::json!({
                "account": AccountSummary::from(record),
                "device_ids": device_ids,
            }))
        })
        .await
}

pub(crate) fn inventory(inspection: homeverb_midea::AccountInventory) -> serde_json::Value {
    let devices: Vec<_> = inspection
        .devices
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "device_id": entry.device_id,
                "connection_id": entry.connection_id,
                "membership": match entry.listed {
                    Some(true) => "listed",
                    Some(false) => "not_listed",
                    None => "unconfirmed",
                },
            })
        })
        .collect();
    serde_json::json!({"account": AccountSummary::from(inspection.account), "devices": devices})
}

#[derive(Serialize)]
pub struct AccountConnection {
    account: AccountSummary,
    authentication: &'static str,
    retrieval: Retrieval,
}

#[derive(Serialize)]
struct Retrieval {
    status: &'static str,
    device_ids: Vec<String>,
    error: Option<Error>,
}

pub(crate) async fn publish(
    core: &Core,
    discovery: AccountDiscovery,
    authentication: &'static str,
    operation: &Operation,
) -> AccountConnection {
    let result: Result<Vec<String>> = async {
        let descriptors = discovery.descriptors?;
        core.publish_discovery("midea.meiju", &descriptors, operation)
            .await?;
        Ok(descriptors
            .into_iter()
            .map(|descriptor| descriptor.device.id)
            .collect())
    }
    .await;
    let retrieval = match result {
        Ok(device_ids) => Retrieval {
            status: "complete",
            device_ids,
            error: None,
        },
        Err(error) => Retrieval {
            status: if error.code == "MIDEA_INVENTORY_PARTIAL" {
                "partial"
            } else {
                "failed"
            },
            device_ids: vec![],
            error: Some(error),
        },
    };
    AccountConnection {
        account: discovery.account.into(),
        authentication,
        retrieval,
    }
}

pub(crate) fn unsupported() -> Error {
    Error::new(
        "AUTH_FLOW_UNSUPPORTED",
        "This host has no admitted account connection flow.",
    )
}
