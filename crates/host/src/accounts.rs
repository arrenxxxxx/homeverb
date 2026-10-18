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
    #[serde(default)]
    pub provider: Option<AccountProvider>,
}

/// Chooses an already registered account flow; never grants authority.
#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum AccountProvider {
    #[serde(rename = "midea.meiju")]
    Meiju,
    #[serde(rename = "xiaomi.mijia")]
    Mijia,
}

#[derive(Serialize)]
pub struct AccountSummary {
    id: Uuid,
    provider: String,
    region: String,
    label: String,
    credential_state: CredentialState,
    last_complete_inventory_at: Option<String>,
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
            last_complete_inventory_at: record
                .last_complete_inventory_at
                .map(|time| time.to_rfc3339()),
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

pub(crate) fn conflicts(inspection: homeverb_midea::DeviceConflictInspection) -> serde_json::Value {
    fn reference(entry: homeverb_midea::AccountDeviceReference) -> serde_json::Value {
        serde_json::json!({"account_id":entry.account_id, "device_id":entry.device.device_id,
        "connection_id":entry.device.connection_id, "membership":match entry.device.listed {
            Some(true) => "listed", Some(false) => "not_listed", None => "unconfirmed",
        }})
    }
    serde_json::json!({"source":"saved_inventory", "authorization":"not_checked", "physical_identity":"unverified",
        "selected":reference(inspection.selected), "matches":inspection.matches.into_iter().map(reference).collect::<Vec<_>>()})
}

pub(crate) fn mijia_inventory(inspection: homeverb_mijia::AccountInventory) -> serde_json::Value {
    let devices: Vec<_> = inspection
        .devices
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "device_id": entry.device_id,
                "connection_id": entry.connection_id,
                "membership": "unconfirmed",
            })
        })
        .collect();
    serde_json::json!({"account": AccountSummary::from(inspection.account), "devices": devices})
}

pub(crate) async fn publish_mijia(
    core: &Core,
    discovery: homeverb_mijia::AccountDiscovery,
    authentication: &'static str,
    operation: &Operation,
) -> AccountConnection {
    let result: Result<Vec<String>> = async {
        let descriptors = discovery.descriptors?;
        core.publish_discovery(homeverb_mijia::PROVIDER, &descriptors, operation)
            .await?;
        Ok(descriptors.into_iter().map(|d| d.device.id).collect())
    }
    .await;
    let retrieval = match result {
        Ok(device_ids) if discovery.complete && discovery.issues.is_empty() => Retrieval {
            status: "complete",
            device_ids,
            error: None,
        },
        Ok(device_ids) => Retrieval {
            status: "partial",
            device_ids,
            error: Some(Error::new(
                "MIJIA_INVENTORY_PARTIAL",
                "Returned devices are available; complete account coverage is not established.",
            )),
        },
        Err(error) => Retrieval {
            status: "failed",
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

pub(crate) enum AccountAdapter<'a> {
    Meiju(&'a homeverb_midea::MeijuAdapter),
    Mijia(&'a homeverb_mijia::MijiaAdapter),
}
impl AccountAdapter<'_> {
    pub async fn account(&self, id: Uuid, op: &Operation) -> Result<AccountRecord> {
        match self {
            Self::Meiju(a) => a.account(id, op).await,
            Self::Mijia(a) => a.account(id, op).await,
        }
    }
    pub async fn rename(&self, id: Uuid, label: &str, op: &Operation) -> Result<AccountRecord> {
        match self {
            Self::Meiju(a) => a.rename_account(id, label, op).await,
            Self::Mijia(a) => a.rename_account(id, label, op).await,
        }
    }
    pub async fn inventory(&self, id: Uuid, op: &Operation) -> Result<serde_json::Value> {
        match self {
            Self::Meiju(a) => a.inspect_inventory(id, op).await.map(inventory),
            Self::Mijia(a) => a.inspect_inventory(id, op).await.map(mijia_inventory),
        }
    }
    pub async fn retrieve(
        &self,
        core: &Core,
        id: Uuid,
        recover: bool,
        op: &Operation,
    ) -> Result<AccountConnection> {
        match self {
            Self::Meiju(a) => {
                let result = if recover {
                    a.recover_devices(id, op).await?
                } else {
                    a.refresh_devices(id, op).await?
                };
                Ok(publish(core, result, "not_attempted", op).await)
            }
            Self::Mijia(a) => {
                let result = if recover {
                    a.recover_devices(id, op).await?
                } else {
                    a.refresh_devices(id, op).await?
                };
                Ok(publish_mijia(core, result, "not_attempted", op).await)
            }
        }
    }
}
