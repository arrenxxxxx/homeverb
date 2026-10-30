//! Explicit modern device adoption. The protected selected protocol controls every read.
use super::*;
use crate::protected::{BindingReadProtocol, ModernBinding};

impl MijiaAdapter {
    pub async fn refresh_modern_devices(
        &self,
        id: Uuid,
        operation: &Operation,
    ) -> Result<AccountDiscovery> {
        self.modern_reads.as_ref().ok_or_else(unsupported)?;
        self.reload(id, false, InventoryProtocol::Modern, operation)
            .await
    }
    pub async fn recover_modern_devices(
        &self,
        id: Uuid,
        operation: &Operation,
    ) -> Result<AccountDiscovery> {
        self.modern_reads.as_ref().ok_or_else(unsupported)?;
        self.reload(id, true, InventoryProtocol::Modern, operation)
            .await
    }
    pub(super) async fn retrieve_modern(
        &self,
        account: &mut LoadedAccount,
        others: &[LoadedAccount],
        recover: bool,
        operation: &Operation,
    ) -> Result<BoundDiscovery> {
        let reads = self.modern_reads.as_deref().ok_or_else(unsupported)?;
        let directory = account
            .payload
            .modern_session()?
            .directory(reads, operation)
            .await?;
        let inventory = directory.reconcile(operation).await?;
        if recover && account.lease.record().credential_state == CredentialState::Pending {
            account
                .lease
                .confirm_session(&account.bytes, operation)
                .await?;
        }
        self.bind_modern_inventory(account, others, inventory, operation)
            .await
    }
    pub(in crate::adapter) async fn bind_modern_inventory(
        &self,
        account: &mut LoadedAccount,
        others: &[LoadedAccount],
        inventory: crate::ModernDeviceInventory,
        operation: &Operation,
    ) -> Result<BoundDiscovery> {
        let other_ids: HashSet<_> = others
            .iter()
            .flat_map(|a| a.payload.bindings.iter().flat_map(Binding::provider_ids))
            .collect();
        for entry in inventory
            .groups
            .iter()
            .flat_map(|g| std::iter::once(g.entry()).chain(g.logical_children()))
            .chain(inventory.unresolved.iter().map(|v| v.entry()))
        {
            if other_ids.contains(&entry.did) {
                return Err(conflict());
            }
        }
        for group in &inventory.groups {
            let device = group.entry().cloud_device();
            for old in &account.payload.bindings {
                if old
                    .modern
                    .as_ref()
                    .is_some_and(|m| m.logical_children.contains(&device.did))
                {
                    return Err(conflict());
                }
                if old.did == device.did {
                    if old.model != device.model
                        || matches!((&old.reported_spec,&device.spec_type),(Some(a),Some(b)) if a != b)
                    {
                        return Err(conflict());
                    }
                    if let Some(previous) = &old.modern {
                        let next = ModernBinding::from_group(group)?;
                        if previous.kind != next.kind || previous.parent != next.parent {
                            return Err(conflict());
                        }
                    }
                }
                for child in group.logical_children() {
                    if old.did == child.did
                        || (old.did != device.did
                            && old
                                .modern
                                .as_ref()
                                .is_some_and(|m| m.logical_children.contains(&child.did)))
                    {
                        return Err(conflict());
                    }
                }
            }
        }
        let mut selected = vec![];
        for group in inventory.groups {
            operation.run(async { Ok(()) }).await?;
            let device = group.entry().cloud_device();
            let index = match account
                .payload
                .bindings
                .iter()
                .position(|b| b.did == device.did)
            {
                Some(index) => index,
                None => {
                    if account.payload.bindings.len() >= 4096 {
                        return Err(unconfirmed());
                    }
                    account.payload.bindings.push(Binding::new(&device));
                    account.payload.bindings.len() - 1
                }
            };
            let evidence = ModernBinding::from_group(&group)?;
            let binding = &mut account.payload.bindings[index];
            if let Some(previous) = &mut binding.modern {
                previous.extend(evidence)?;
            } else {
                binding.modern = Some(evidence);
            }
            binding.read_protocol = BindingReadProtocol::Modern;
            binding.name = device.name;
            binding.reported_spec = device.spec_type;
            binding.mapping =
                ModelMapping::pinned(&binding.model, binding.reported_spec.as_deref())?;
            binding.listed = true;
            account.payload.enable_modern_bindings();
            selected.push(index);
        }
        account.save(operation).await?;
        let descriptors = selected
            .into_iter()
            .map(|i| {
                descriptor(
                    &account.payload.bindings[i],
                    account.lease.record().id,
                    false,
                    false,
                    false,
                )
            })
            .collect();
        operation
            .run(async {
                Ok(BoundDiscovery {
                    descriptors,
                    issues: inventory.issues,
                    complete: false,
                })
            })
            .await
    }
}
