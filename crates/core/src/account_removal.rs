//! Durable local cleanup; never provider revocation or appliance control.

use crate::{AccountVault, Operation, valid_account_id};
use homeverb_domain::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountRemoval {
    pub account_id: Uuid,
    pub provider: String,
    pub region: String,
    pub device_ids: Vec<String>,
    pub completed: bool,
}

#[derive(Debug, Serialize)]
pub struct AccountRemovalResult {
    pub account_id: Uuid,
    pub device_ids: Vec<String>,
    pub status: &'static str,
    pub remote_revocation: &'static str,
    pub error: Option<Error>,
}

pub fn account_removal_pending() -> Error {
    Error::new(
        "ACCOUNT_REMOVAL_PENDING",
        "This local account is being removed and cannot be used.",
    )
}

pub fn account_scope_changed() -> Error {
    Error::new(
        "ACCOUNT_SCOPE_CHANGED",
        "The linked device set changed; review local account details again.",
    )
}

pub fn removal_device_ids(ids: &[String]) -> Result<Vec<String>> {
    if ids.len() > 4096
        || ids
            .iter()
            .any(|id| id.is_empty() || id.len() > 256 || id.chars().any(char::is_control))
    {
        return Err(Error::invalid());
    }
    let unique: BTreeSet<_> = ids.iter().cloned().collect();
    if unique.len() != ids.len() {
        return Err(Error::invalid());
    }
    Ok(unique.into_iter().collect())
}

impl AccountVault {
    /// Requires an explicit local cleanup request and the user-reviewed device set.
    pub async fn remove_account(
        &self,
        id: Uuid,
        provider: &str,
        region: &str,
        expected_devices: &[String],
        operation: &Operation,
    ) -> Result<AccountRemovalResult> {
        valid_account_id(id)?;
        let expected = removal_device_ids(expected_devices)?;
        let _registration = self.lock_registration(operation).await?;
        let _account = operation.run(async { self.store.lock_account(id) }).await?;
        let previous = operation
            .run(async { self.store.account_removal(id) })
            .await?;
        if let Some(previous) = previous {
            if previous.provider != provider || previous.region != region {
                return Err(Error::new(
                    "ACCOUNT_NOT_FOUND",
                    "The selected local account does not exist.",
                ));
            }
            if previous.device_ids != expected {
                return Err(account_scope_changed());
            }
            if previous.completed {
                return Ok(removal_result(previous, None));
            }
        }
        let record = operation.run(async { self.store.account(id) }).await?;
        if record.provider != provider || record.region != region {
            return Err(Error::new(
                "ACCOUNT_NOT_FOUND",
                "The selected local account does not exist.",
            ));
        }
        let removal = operation
            .run(async { self.store.begin_account_removal(id, &expected) })
            .await?;
        let cleanup: Result<()> = async {
            let present = self
                .secrets
                .get(&record.credential_ref, operation)
                .await?
                .is_some();
            if present {
                self.secrets
                    .delete(&record.credential_ref, operation)
                    .await?;
            }
            operation
                .run(async { self.store.finish_account_removal(id) })
                .await
        }
        .await;
        Ok(removal_result(removal, cleanup.err()))
    }
}

fn removal_result(removal: AccountRemoval, error: Option<Error>) -> AccountRemovalResult {
    AccountRemovalResult {
        account_id: removal.account_id,
        device_ids: removal.device_ids,
        status: if error.is_none() {
            "removed"
        } else {
            "pending"
        },
        remote_revocation: "not_attempted",
        error,
    }
}
