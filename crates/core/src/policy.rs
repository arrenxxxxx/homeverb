use homeverb_domain::*;
use serde_json::Value;

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
        let mut permitted = Vec::new();
        if action.risk == Risk::Low {
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
            return Err(denied());
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
