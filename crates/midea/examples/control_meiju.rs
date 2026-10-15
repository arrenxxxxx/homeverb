//! Explicit, exact-device AC power probe. Preview is the default; real sends need owner consent.

use clap::{Parser, ValueEnum};
use homeverb_core::{AccountStore, AccountVault, Core, Grant, Operation, Policy, SystemClock};
use homeverb_domain::{Command, Error, Result, Route, Status, Transport, WaitFor};
use homeverb_midea::{MeijuAdapter, MeijuHttps};
use serde_json::json;
use std::{path::PathBuf, sync::Arc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Clone, Copy, ValueEnum)]
enum Power {
    On,
    Off,
}

#[derive(Parser)]
#[command(
    about = "Preview one experimental Meiju AC power command; --send performs a real appliance write."
)]
struct Arguments {
    #[arg(long)]
    data_dir: PathBuf,
    /// Exact random local device ID from this profile, never a provider appliance code.
    #[arg(long)]
    device_id: Uuid,
    #[arg(long, value_enum)]
    state: Power,
    /// Keep this ID when inspecting/repeating the same invocation; never replay with a new ID.
    #[arg(long)]
    request_id: String,
    /// Requires explicit appliance-owner authorization for this exact target and state.
    #[arg(long)]
    send: bool,
}

#[tokio::main]
async fn main() {
    match run(Arguments::parse()).await {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{}: {}", error.code, error.message);
            std::process::exit(1);
        }
    }
}

async fn run(arguments: Arguments) -> Result<i32> {
    let command = Command {
        schema_version: "1.0".into(),
        request_id: arguments.request_id,
        device_id: arguments.device_id.to_string(),
        component_id: "main".into(),
        capability: "power".into(),
        action: "set".into(),
        arguments: json!({"state":matches!(arguments.state, Power::On)}),
        connection: Route::Cloud,
        timeout_ms: 20_000,
        wait_for: WaitFor::Verified,
    };
    command.validate()?;
    let operation = Operation::new(20_000, CancellationToken::new())?;
    let store = Arc::new(homeverb_storage::SqliteStore::open(&arguments.data_dir)?);
    let secrets = Arc::new(homeverb_secrets::SystemSecretStore::new(
        store.profile_namespace()?,
    )?);
    let vault = Arc::new(AccountVault::new(store.clone(), secrets));
    let adapter = Arc::new(
        MeijuAdapter::new(vault, Arc::new(MeijuHttps), Arc::new(SystemClock))
            .with_ac_power_control(),
    );
    let core = Core::new(
        store,
        vec![adapter.clone()],
        Arc::new(SystemClock),
        Policy {
            grants: vec![Grant {
                device_id: command.device_id.clone(),
                component_id: command.component_id.clone(),
                capability: command.capability.clone(),
                action: command.action.clone(),
                transports: vec![Transport::Cloud],
                argument_schema: json!({"const":command.arguments}),
            }],
        },
    );
    let work = async {
        let device = core.describe(&command.device_id)?.device;
        let account = device
            .connections
            .iter()
            .find(|c| c.adapter == "midea.meiju" && c.transport == Transport::Cloud)
            .and_then(|c| c.account_id.as_deref())
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| {
                Error::new(
                    "DEVICE_IDENTITY_CONFLICT",
                    "The selected device has no Meiju account binding.",
                )
            })?;
        let discovery = adapter.refresh_devices(account, &operation).await?;
        let descriptors = discovery.descriptors?;
        // A historical binding alone must not authorize a probe for a missing inventory member.
        if !descriptors.iter().any(|d| d.device.id == command.device_id) {
            return Err(Error::new(
                "DEVICE_NOT_FOUND",
                "The selected device is absent from the complete current inventory.",
            ));
        }
        core.publish_discovery("midea.meiju", &descriptors, &operation)
            .await?;
        if !arguments.send {
            let plan = core.plan(&command)?;
            println!(
                "{}",
                json!({"mode":"preview","model":device.model,"plan":plan,"controls_sent":0,"physically_verified":false})
            );
            return Ok(0);
        }
        let result = core.execute_with_operation(command, &operation).await;
        let code = match result.status {
            Status::Verified => 0,
            Status::Accepted => 10,
            Status::Unknown => 11,
            _ => 1,
        };
        println!(
            "{}",
            serde_json::to_string(&result).expect("Serializable execution.")
        );
        Ok(code)
    };
    tokio::pin!(work);
    tokio::select! {
        result = &mut work => result,
        _ = tokio::signal::ctrl_c() => {
            operation.cancellation.cancel();
            work.await
        }
    }
}
