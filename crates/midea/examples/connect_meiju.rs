//! Explicit Linux read-only account probe. No device controls or public credential JSON.

use clap::{Parser, Subcommand};
use homeverb_core::{AccountRecord, Core, Operation, Policy, SecretBytes, SystemClock};
use homeverb_domain::{Error, Result, Route};
use homeverb_midea::{
    ApplianceFamily, BoundInventory, LoginCredentials, MeijuAccount, MeijuAdapter, MeijuHttps,
    StateQuery, connect, read_cloud_state,
};
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
    sync::Arc,
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
    /// Read mapped cloud state for the account inventory; never sends appliance controls.
    Inspect { account_id: Uuid },
    /// Publish read-only device cards and read their components through Core for saved accounts.
    CoreInspect,
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
    let vault = Arc::new(homeverb_host::open_accounts(&arguments.data_dir)?);
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
            Command::CoreInspect => {
                let clock = Arc::new(SystemClock);
                let core = Core::new(
                    Arc::new(homeverb_storage::SqliteStore::open(&arguments.data_dir)?),
                    vec![Arc::new(MeijuAdapter::new(
                        vault.clone(),
                        Arc::new(MeijuHttps),
                        clock.clone(),
                    ))],
                    clock,
                    Policy::default(),
                );
                let devices = core.discover(&operation).await?;
                let mut failures = 0;
                let mut components_read = 0;
                for device in &devices {
                    for component in &device.components {
                        let mut report = serde_json::json!({"model":device.model,"component":component.id,"support_status":device.support_status});
                        match core
                            .read(&device.id, &component.id, Route::Cloud, false, &operation)
                            .await
                        {
                            Ok(snapshot) => {
                                components_read += 1;
                                // Report property names and evidence quality, not private state values.
                                report["properties"] = serde_json::json!(
                                    snapshot.observations.keys().collect::<Vec<_>>()
                                );
                                report["all_stale"] = serde_json::json!(
                                    snapshot.observations.values().all(|value| value.stale)
                                );
                            }
                            Err(error) => {
                                if matches!(error.code.as_str(), "CANCELLED" | "DEADLINE_EXCEEDED")
                                {
                                    return Err(error);
                                }
                                failures += 1;
                                report["error_code"] = serde_json::json!(error.code);
                            }
                        }
                        println!("{report}");
                    }
                }
                println!(
                    "{}",
                    serde_json::json!({"devices":devices.len(),"components_read":components_read,"failures":failures,"physically_verified":false})
                );
                if failures == 0 {
                    Ok(())
                } else {
                    Err(Error::new(
                        "MIDEA_REQUEST_FAILED",
                        "Some Core reads failed; no controls were sent.",
                    ))
                }
            }
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
            Command::Inspect { account_id } => {
                let mut account =
                    MeijuAccount::open(vault.lease(account_id, &operation).await?, &operation)
                        .await?;
                let bound = account.retrieve(&MeijuHttps, &operation).await?;
                let session = account.cloud_session(&operation).await?;
                let reference = account.record().credential_ref;
                let reads = session.reads(&MeijuHttps, reference);
                let mut failures = bound.inventory.failures.len();
                for (appliance, binding) in bound.inventory.appliances.iter().zip(&bound.bindings) {
                    let family = match appliance.family {
                        ApplianceFamily::AirConditioner => "air_conditioner",
                        ApplianceFamily::Dishwasher => "dishwasher",
                        ApplianceFamily::Refrigerator => "refrigerator",
                        ApplianceFamily::Other(_) => "other",
                    };
                    let mut report = serde_json::json!({"family":family,"model":appliance.model,"physically_verified":false});
                    match read_cloud_state(
                        &reads,
                        reference,
                        appliance,
                        StateQuery::Default,
                        &SystemClock,
                        &operation,
                    )
                    .await
                    {
                        Ok(state) => {
                            let snapshots: Vec<_> = state
                                .component_ids()
                                .map(|component| {
                                    state.snapshot(&binding.device_id.to_string(), component)
                                })
                                .collect::<Result<_>>()?;
                            report["snapshots"] = serde_json::json!(snapshots);
                        }
                        Err(error) => {
                            if matches!(error.code.as_str(), "CANCELLED" | "DEADLINE_EXCEEDED") {
                                return Err(error);
                            }
                            failures += 1;
                            report["error_code"] = serde_json::json!(error.code);
                        }
                    }
                    println!("{report}");
                }
                if failures == 0 {
                    Ok(())
                } else {
                    Err(Error::new(
                        "MIDEA_REQUEST_FAILED",
                        "Some inventory or state reads failed; no controls were sent.",
                    ))
                }
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
