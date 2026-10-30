//! CLI history requests preserve the shared closed DTO and Core validation.
use clap::Subcommand;
use homeverb_core::{Core, Operation};
use homeverb_domain::{Error, HistoryRequest, Result};
use serde_json::{Value, json};
use std::{io::Read, path::PathBuf};

#[derive(Subcommand)]
pub enum HistoryCommand {
    /// Inspect one exact local connection without provider traffic.
    Options {
        device_id: String,
        #[arg(long)]
        connection_id: String,
    },
    /// Read one series using a closed HistoryRequest JSON file (at most 16 KiB).
    Read {
        #[arg(long)]
        file: PathBuf,
    },
}

fn request(path: &std::path::Path) -> Result<HistoryRequest> {
    read_request(path)
}

pub(super) fn read_request<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T> {
    if !std::fs::metadata(path)
        .map_err(|_| Error::invalid())?
        .is_file()
    {
        return Err(Error::invalid());
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    let file = options.open(path).map_err(|_| Error::invalid())?;
    if !file.metadata().map_err(|_| Error::invalid())?.is_file() {
        return Err(Error::invalid());
    }
    let mut bytes = Vec::new();
    file.take(16 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::invalid())?;
    if bytes.len() > 16 * 1024 {
        return Err(Error::invalid());
    }
    serde_json::from_slice(&bytes).map_err(|_| Error::invalid())
}

pub async fn run(core: &Core, command: HistoryCommand, operation: &Operation) -> Result<Value> {
    match command {
        HistoryCommand::Options {
            device_id,
            connection_id,
        } => core
            .inspect_history_options(&device_id, &connection_id, operation)
            .await
            .map(|v| json!(v)),
        HistoryCommand::Read { file } => {
            let query = request(&file)?;
            core.read_history(&query, operation).await.map(|v| json!(v))
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
