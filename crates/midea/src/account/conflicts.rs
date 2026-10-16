//! Local references for cross-account conflict diagnosis, never merge authority.

use super::*;

impl MeijuAccount {
    pub(crate) async fn conflict_binding(
        &self,
        device_id: Uuid,
        operation: &Operation,
    ) -> Result<(CloudId, DeviceInventoryMembership)> {
        let payload = self.load(operation).await?;
        let binding = payload
            .bindings
            .iter()
            .find(|binding| binding.local.device_id == device_id)
            .ok_or_else(identity_conflict)?;
        Ok((
            CloudId::parse(&serde_json::Value::String(binding.appliance_id.clone()))?,
            DeviceInventoryMembership {
                device_id: binding.local.device_id,
                connection_id: binding.local.connection_id,
                listed: binding.listed,
            },
        ))
    }

    pub(crate) async fn matching_binding(
        &self,
        appliance_id: &CloudId,
        operation: &Operation,
    ) -> Result<Option<DeviceInventoryMembership>> {
        let payload = self.load(operation).await?;
        Ok(payload
            .bindings
            .iter()
            .find(|binding| binding.appliance_id == appliance_id.expose())
            .map(|binding| DeviceInventoryMembership {
                device_id: binding.local.device_id,
                connection_id: binding.local.connection_id,
                listed: binding.listed,
            }))
    }
}
