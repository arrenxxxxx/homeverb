//! Explicit Linux read-only account probe. No device controls or public credential JSON.

use clap::{Parser, Subcommand};
use homeverb_core::{AccountRecord, Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use homeverb_midea::{
    ApplianceFamily, BoundInventory, LoginCredentials, MeijuAccount, MeijuHttps, connect,
};
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Parser)]
#[command(about = "Read-only mainland Meiju account probe; never sends appliance controls.")]
struct Arguments {
    /// Private local metadata directory. Sessions are stored only in the system secret store.
    #[arg(long)]
    data_dir: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Connect using hidden interactive input, save the session and retrieve bound devices.
    Connect {
        /// Reauthorize this exact existing local account; a changed provider subject is refused.
        #[arg(long)]
        account_id: Option<Uuid>,
    },
    /// Retrieve devices with the saved session; never attempts login or automatic renewal.
    Refresh { account_id: Uuid },
    /// List only local account references; no network requests or credential reads.
    List,
}

fn input(prompt: &str) -> Result<SecretBytes> {
    rpassword::prompt_password(prompt)
        .map(|value| SecretBytes::new(value.into_bytes()))
        .map_err(|_| {
            Error::new(
                "INPUT_UNAVAILABLE",
                "Protected terminal input is unavailable.",
            )
        })
}

fn display(record: &AccountRecord, retrieval: Result<BoundInventory>) -> Result<()> {
    let mut report = serde_json::json!({
        "local_account_id": record.id,
        "credential_state": record.credential_state,
    });
    let error = match retrieval {
        Ok(bound) => {
            let mut counts = [0usize; 4];
            for appliance in &bound.inventory.appliances {
                counts[match appliance.family {
                    ApplianceFamily::AirConditioner => 0,
                    ApplianceFamily::Dishwasher => 1,
                    ApplianceFamily::Refrigerator => 2,
                    ApplianceFamily::Other(_) => 3,
                }] += 1;
            }
            let complete = bound.inventory.is_complete();
            report["retrieval"] = serde_json::json!(if complete { "complete" } else { "partial" });
            report["device_counts"] = serde_json::json!({"air_conditioner":counts[0],"dishwasher":counts[1],"refrigerator":counts[2],"other":counts[3]});
            report["failed_homes"] = serde_json::json!(bound.inventory.failures.len());
            report["physically_verified"] = serde_json::json!(false);
            if complete {
                None
            } else {
                Some(Error::new(
                    "MIDEA_REQUEST_FAILED",
                    "Device retrieval was partial; the stored session remains separate.",
                ))
            }
        }
        Err(error) => {
            report["retrieval"] = serde_json::json!("failed");
            report["error_code"] = serde_json::json!(error.code);
            Some(error)
        }
    };
    println!("{report}");
    error.map_or(Ok(()), Err)
}

#[tokio::main]
async fn main() {
    if let Err(error) = run(Arguments::parse()).await {
        eprintln!("{}: {}", error.code, error.message);
        std::process::exit(1);
    }
}

async fn run(arguments: Arguments) -> Result<()> {
    let vault = homeverb_host::open_accounts(&arguments.data_dir)?;
    let credentials = if matches!(arguments.command, Command::Connect { .. }) {
        if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
            return Err(Error::new(
                "INPUT_UNAVAILABLE",
                "Connect requires an interactive terminal; credentials cannot be piped or passed as arguments.",
            ));
        }
        // Fail before prompting when the configured secure store is unavailable or locked.
        vault
            .check_secure_storage(&Operation::new(10_000, CancellationToken::new())?)
            .await?;
        Some(LoginCredentials::new(
            input("Meiju account (hidden): ")?,
            input("Meiju password (hidden): ")?,
        )?)
    } else {
        None
    };
    // User typing precedes network work; the shared 180-second budget starts after input.
    let operation = Operation::new(180_000, CancellationToken::new())?;
    let work = async {
        match arguments.command {
            Command::Connect { account_id } => {
                let report = connect(
                    &vault,
                    &MeijuHttps,
                    credentials.expect("Connect collected protected input."),
                    account_id,
                    "Meiju account",
                    &operation,
                )
                .await?;
                display(&report.account, report.retrieval)
            }
            Command::Refresh { account_id } => {
                let mut account =
                    MeijuAccount::open(vault.lease(account_id, &operation).await?, &operation)
                        .await?;
                let retrieval = account.retrieve(&MeijuHttps, &operation).await;
                display(account.record(), retrieval)
            }
            Command::List => {
                let records = vault.list(&operation).await?;
                let summary: Vec<_> = records.into_iter().map(|record| serde_json::json!({"local_account_id": record.id,"provider":record.provider,"region":record.region,"credential_state":record.credential_state})).collect();
                println!("{}", serde_json::json!(summary));
                Ok(())
            }
        }
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
