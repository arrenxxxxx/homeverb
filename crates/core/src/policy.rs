use homeverb_domain::*;
use serde_json::Value;

impl crate::Core {
    /// Inspect local session authority without touching adapters, credentials or metadata.
    /// Matching rules still require capability, risk and execution validation.
    pub async fn inspect_session_grants(
        &self,
        device_id: &str,
        connection_id: &str,
        operation: &crate::Operation,
    ) -> Result<Value> {
        let result = operation
            .run(async {
                let descriptor = self.describe(device_id)?;
                if descriptor.device.id != device_id {
                    return Err(Error::invalid());
                }
                let connection = descriptor
                    .device
                    .connections
                    .iter()
                    .find(|connection| connection.id == connection_id)
                    .ok_or_else(Error::invalid)?;
                let grants: Vec<_> = self
                    .policy
                    .grants
                    .iter()
                    .filter(|grant| {
                        grant.device_id == device_id
                            && grant.transports.contains(&connection.transport)
                            && jsonschema::validator_for(&grant.argument_schema).is_ok()
                    })
                    .map(|grant| {
                        serde_json::json!({
                            "component_id":grant.component_id,"capability":grant.capability,
                            "action":grant.action,"argument_schema":grant.argument_schema
                        })
                    })
                    .collect();
                Ok(serde_json::json!({
                    "device_id": device_id,
                    "connection_id": connection_id,
                    "transport": connection.transport,
                    "source": "session_policy",
                    "provider_authorization": "not_checked",
                    "grants": grants
                }))
            })
            .await?;
        operation.run(async { Ok(result) }).await
    }
}

/// Constructed by the trusted host, never deserialized from a command or tool call.
#[derive(Clone, Default)]
pub struct Policy {
    pub grants: Vec<Grant>,
}

#[derive(Clone)]
pub struct Grant {
    pub device_id: String,
    pub component_id: String,
    pub capability: String,
    pub action: String,
    pub transports: Vec<Transport>,
    pub argument_schema: Value,
}

impl Policy {
    /// Return only routes authorized for the complete command by a matching grant.
    pub fn authorize(&self, command: &Command, action: &Action) -> Result<Vec<Transport>> {
        self.authorize_risk(command, action, Risk::Low)
    }

    pub(crate) fn authorize_elevated(
        &self,
        command: &Command,
        action: &Action,
    ) -> Result<Vec<Transport>> {
        self.authorize_risk(command, action, Risk::Elevated)
    }

    fn authorize_risk(
        &self,
        command: &Command,
        action: &Action,
        risk: Risk,
    ) -> Result<Vec<Transport>> {
        let mut permitted = Vec::new();
        if action.risk == risk {
            for grant in &self.grants {
                if grant.device_id != command.device_id
                    || grant.component_id != command.component_id
                    || grant.capability != command.capability
                    || grant.action != command.action
                {
                    continue;
                }
                // The dependency disables external schema retrieval. Invalid grants add no authority.
                let Ok(validator) = jsonschema::validator_for(&grant.argument_schema) else {
                    continue;
                };
                if !validator.is_valid(&command.arguments) {
                    continue;
                }
                for transport in &grant.transports {
                    if command.connection.permits(*transport)
                        && action.transports.contains(transport)
                        && !permitted.contains(transport)
                    {
                        permitted.push(*transport);
                    }
                }
            }
        }
        if permitted.is_empty() {
            return Err(if risk == Risk::Low {
                denied()
            } else {
                crate::approval::denied()
            });
        }
        Ok(permitted)
    }
}

pub(crate) fn denied() -> Error {
    Error::new(
        "POLICY_DENIED",
        "A trusted low-risk grant must permit this device, component, action, arguments, and route.",
    )
}
