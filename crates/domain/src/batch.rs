use crate::{Command, Error, Execution, Result, SCHEMA_VERSION, Status};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchCommand {
    pub schema_version: String,
    pub commands: Vec<Command>,
}

impl BatchCommand {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != SCHEMA_VERSION || !(1..=32).contains(&self.commands.len()) {
            return Err(Error::invalid());
        }
        let mut devices = HashSet::new();
        let mut requests = HashSet::new();
        for command in &self.commands {
            command.validate()?;
            if !devices.insert(&command.device_id) || !requests.insert(&command.request_id) {
                return Err(Error::invalid());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchExecution {
    pub schema_version: String,
    pub status: Status,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub items: Vec<Execution>,
}
