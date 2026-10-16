//! Inspect saved identities without changing discovery, routing or authorization.

use super::*;
use crate::DeviceInventoryMembership;

pub struct AccountDeviceReference {
    pub account_id: Uuid,
    pub device: DeviceInventoryMembership,
}

pub struct DeviceConflictInspection {
    pub selected: AccountDeviceReference,
    pub matches: Vec<AccountDeviceReference>,
}

impl MeijuAdapter {
    /// Equal saved provider identifiers are candidates, not physical identity proof.
    pub async fn inspect_device_conflicts(
        &self,
        account_id: Uuid,
        device_id: Uuid,
        operation: &Operation,
    ) -> Result<DeviceConflictInspection> {
        homeverb_core::valid_account_id(account_id)?;
        homeverb_core::valid_account_id(device_id)?;
        let _registration = self.vault.lock_registration(operation).await?;
        let mut records = self.accounts(operation).await?;
        if !records.iter().any(|record| record.id == account_id) {
            return Err(Error::new(
                "ACCOUNT_NOT_FOUND",
                "The selected Meiju account does not exist.",
            ));
        }
        if records.len() > 64 {
            return Err(Error::new(
                "MIDEA_CONFLICT_LIMIT",
                "Saved device conflict inspection supports at most 64 mainland Meiju accounts.",
            ));
        }
        records.sort_by_key(|record| record.id);
        let mut accounts = Vec::with_capacity(records.len());
        for record in records {
            accounts.push(
                MeijuAccount::open(self.vault.lease(record.id, operation).await?, operation)
                    .await?,
            );
        }
        let selected = accounts
            .iter()
            .find(|account| account.record().id == account_id)
            .unwrap();
        let (identity, device) = selected.conflict_binding(device_id, operation).await?;
        let mut matches = Vec::new();
        for account in &accounts {
            if account.record().id != account_id
                && let Some(device) = account.matching_binding(&identity, operation).await?
            {
                matches.push(AccountDeviceReference {
                    account_id: account.record().id,
                    device,
                });
            }
        }
        operation
            .run(async {
                Ok(DeviceConflictInspection {
                    selected: AccountDeviceReference { account_id, device },
                    matches,
                })
            })
            .await
    }
}
