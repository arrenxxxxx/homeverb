//! Explicit occurrence-time history, independent of numeric series and live state.
use clap::Subcommand;
use homeverb_core::{Core, Operation};
use homeverb_domain::{EventHistoryRequest, Result};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum EventHistoryCommand {
    /// Inspect exact local event selectors without provider traffic.
    Options {
        device_id: String,
        #[arg(long)]
        connection_id: String,
    },
    /// Read one event window from a closed EventHistoryRequest file, at most 16 KiB.
    Read {
        #[arg(long)]
        file: PathBuf,
    },
}
fn request(path: &std::path::Path) -> Result<EventHistoryRequest> {
    super::history::read_request(path)
}
pub async fn run(
    core: &Core,
    command: EventHistoryCommand,
    operation: &Operation,
) -> Result<Value> {
    match command {
        EventHistoryCommand::Options {
            device_id,
            connection_id,
        } => core
            .inspect_event_history_options(&device_id, &connection_id, operation)
            .await
            .map(|v| json!(v)),
        EventHistoryCommand::Read { file } => core
            .read_event_history(&request(&file)?, operation)
            .await
            .map(|v| json!(v)),
    }
}
#[cfg(test)]
pub(crate) mod tests;
