use super::*;
use serde::Serialize;

/// Permission-form metadata only; not an executable descriptor, grant or approval.
#[derive(Serialize)]
pub struct ControlOption {
    component_id: &'static str,
    capability: &'static str,
    action: &'static str,
    definition: Action,
}

fn options(appliance: &Appliance) -> Vec<ControlOption> {
    let mut options = Vec::new();
    let mut add = |capability, action, definition| {
        options.push(ControlOption {
            component_id: "main",
            capability,
            action,
            definition,
        });
    };
    match appliance
        .model
        .as_deref()
        .and_then(model_family)
        .filter(|family| *family == appliance.family)
    {
        Some(ApplianceFamily::AirConditioner) => {
            add("power", "set", power_action());
            add("vendor.midea.ac", "set_fan_speed", fan_action());
            add("vendor.midea.ac", "activate_mode", mode_action());
            if let Some(profile) = appliance.product_profile.temperature() {
                add("climate", "set", temperature_action(profile));
            }
        }
        Some(ApplianceFamily::Dishwasher) => {
            if let Some(profile) = appliance.product_profile.dishwasher() {
                add("vendor.midea.dishwasher", "start_cycle", profile.action());
                for action in crate::DISHWASHER_CYCLE_ACTIONS {
                    if let Some(definition) = profile.cycle_action(action) {
                        add("vendor.midea.dishwasher", action.action_name(), definition);
                    }
                }
            }
        }
        _ => (),
    }
    options
}

impl MeijuAdapter {
    /// Inspect only saved protected evidence. This never enables adapter controls,
    /// contacts the provider, publishes descriptors or changes Core permissions.
    pub async fn inspect_control_options(
        &self,
        device: &Device,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Vec<ControlOption>> {
        let id = connection
            .account_id
            .as_deref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(crate::account::identity_conflict)?;
        self.account(id, operation).await?;
        let account = MeijuAccount::open(self.vault.lease(id, operation).await?, operation).await?;
        let appliance = account
            .bound_appliance(&device.id, connection, operation)
            .await?;
        if appliance.model.as_deref().unwrap_or("Unknown model") != device.model {
            return Err(crate::account::identity_conflict());
        }
        operation.run(async { Ok(options(&appliance)) }).await
    }
}
