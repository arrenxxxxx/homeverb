mod accounts;
mod grants;
mod mcp;
mod terminal;

use clap::{Args, Parser, Subcommand};
use homeverb_core::Operation;
use homeverb_domain::*;
use homeverb_mock::Scenario;
use serde_json::{Value, json};
use std::{path::PathBuf, process::ExitCode};
use tokio_util::sync::CancellationToken;

#[derive(Parser)]
#[command(
    name = "homeverb",
    version,
    about = "HomeVerb: explicit on-demand appliance operations"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        help = "Use the fictional appliance; no real devices are contacted"
    )]
    mock: bool,
    #[arg(long, global = true, conflicts_with = "mock", value_parser = ["midea.meiju"])]
    adapter: Option<String>,
    #[arg(
        long,
        global = true,
        requires = "adapter",
        conflicts_with = "mock",
        help = "Explicit absolute path to an owner-controlled Meiju power/fan grant file"
    )]
    meiju_grants: Option<PathBuf>,
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help = "Local profile; defaults to mock or meiju for the selected adapter"
    )]
    profile: Option<String>,
    #[arg(long, global = true, help = "Fictional fault scenario; Mock only")]
    scenario: Option<String>,
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, global = true, default_value = "auto")]
    connection: String,
    #[arg(
        long,
        global = true,
        help = "Operation budget: default 10000 ms, or 180000 ms for account connect after input"
    )]
    timeout_ms: Option<u64>,
    #[arg(long, global = true, default_value = "verified")]
    wait: String,
    #[arg(long, global = true)]
    request_id: Option<String>,
    #[arg(long, global = true)]
    dry_run: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Account {
        #[command(subcommand)]
        command: accounts::AccountCommand,
    },
    Device {
        #[command(subcommand)]
        command: DeviceCommand,
    },
    Power {
        #[command(subcommand)]
        command: PowerCommand,
    },
    Climate {
        #[command(subcommand)]
        command: ClimateCommand,
    },
    /// Execute explicit device commands from a local JSON batch file.
    Batch {
        #[arg(long)]
        file: PathBuf,
    },
    Executions {
        #[arg(long)]
        device: Option<String>,
    },
    /// Read one retained receipt without sending a command.
    Execution {
        #[arg(value_name = "REQUEST_ID")]
        receipt_id: String,
    },
    /// Preview a bounded, redacted report of this local profile.
    Diagnostics,
    Room {
        #[command(subcommand)]
        command: RoomCommand,
    },
    Mcp {
        #[arg(long, required = true)]
        stdio: bool,
    },
}
#[derive(Subcommand)]
enum DeviceCommand {
    Discover,
    List,
    Cards,
    Preferences {
        device: String,
        #[arg(long)]
        settings: String,
    },
    Reorder {
        device_ids: Vec<String>,
    },
    Describe {
        device: String,
    },
    State {
        device: String,
        #[arg(long, default_value = "main")]
        component: String,
        #[arg(long, conflicts_with = "cached")]
        fresh: bool,
        #[arg(long)]
        cached: bool,
    },
    Execute {
        device: String,
        #[arg(long, default_value = "main")]
        component: String,
        #[arg(long)]
        capability: String,
        #[arg(long)]
        action: String,
        #[arg(long)]
        arguments: String,
    },
}
#[derive(Subcommand)]
enum RoomCommand {
    List,
    Create {
        name: String,
    },
    Rename {
        room_id: String,
        name: String,
    },
    Delete {
        room_id: String,
    },
    Reorder {
        room_ids: Vec<String>,
    },
    Assign {
        #[arg(required = true)]
        device_ids: Vec<String>,
        #[arg(
            long = "room",
            required_unless_present = "unassigned",
            conflicts_with = "unassigned"
        )]
        room_id: Option<String>,
        #[arg(long)]
        unassigned: bool,
    },
}
#[derive(Subcommand)]
enum PowerCommand {
    Set(PowerSet),
}
#[derive(Args)]
#[group(skip)]
struct PowerSet {
    device: String,
    #[arg(long, required_unless_present = "off", conflicts_with = "off")]
    on: bool,
    #[arg(long, required_unless_present = "on", conflicts_with = "on")]
    off: bool,
}
#[derive(Subcommand)]
enum ClimateCommand {
    Set {
        device: String,
        #[arg(long)]
        temperature: Option<f64>,
        #[arg(long)]
        mode: Option<String>,
    },
}

fn main() -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => {
            eprintln!("INTERNAL_ERROR: The command runtime could not start.");
            return ExitCode::from(10);
        }
    };
    let code = runtime.block_on(async_main());
    // Tokio stdin uses a blocking read. Do not wait indefinitely for it after SIGINT.
    runtime.shutdown_timeout(std::time::Duration::from_millis(250));
    code
}

async fn async_main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) if error.use_stderr() => {
            print_json(&Query::<Value>::from_result(Err(Error::new(
                "INVALID_ARGUMENT",
                "Invalid command syntax; run homeverb --help.",
            ))));
            return ExitCode::from(2);
        }
        Err(error) => {
            let _ = error.print();
            return ExitCode::SUCCESS;
        }
    };
    let cancellation = CancellationToken::new();
    let is_mcp = matches!(cli.command, Commands::Mcp { .. });
    let signal = cancellation.clone();
    let signal_task = tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            signal.cancel();
        }
    });
    let result = run(cli, cancellation).await;
    signal_task.abort();
    match result {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let code = error_exit(&error);
            if is_mcp {
                eprintln!("{error}");
            } else {
                print_json(&Query::<Value>::from_result(Err(error)));
            }
            ExitCode::from(code)
        }
    }
}

async fn run(cli: Cli, cancellation: CancellationToken) -> Result<u8> {
    let meiju = cli.adapter.is_some();
    if !cli.mock && !meiju {
        return Err(Error::new(
            "CAPABILITY_UNSUPPORTED",
            "Select --mock or --adapter midea.meiju explicitly.",
        ));
    }
    if meiju && (cli.scenario.is_some() || matches!(cli.command, Commands::Mcp { .. })) {
        return Err(Error::new(
            "CAPABILITY_UNSUPPORTED",
            "Mock scenarios and MCP startup require --mock.",
        ));
    }
    let is_write = matches!(
        cli.command,
        Commands::Power { .. }
            | Commands::Climate { .. }
            | Commands::Batch { .. }
            | Commands::Device {
                command: DeviceCommand::Execute { .. }
            }
    );
    let grants = grants::Grants::load(cli.meiju_grants.as_deref())?;
    if meiju && is_write && grants.is_empty() {
        return Err(Error::new(
            "POLICY_DENIED",
            "The Meiju CLI requires explicit scoped startup grants for appliance writes.",
        ));
    }
    if meiju
        && is_write
        && !matches!(cli.command, Commands::Batch { .. })
        && cli.request_id.is_none()
    {
        return Err(Error::new(
            "INVALID_ARGUMENT",
            "Meiju writes and previews require an explicit --request-id.",
        ));
    }
    if matches!(cli.command, Commands::Account { .. })
        && (!meiju
            || cli.dry_run
            || cli.connection != "auto"
            || cli.wait != "verified"
            || cli.request_id.is_some())
    {
        return Err(Error::invalid());
    }
    let profile = cli
        .profile
        .as_deref()
        .unwrap_or(if meiju { "meiju" } else { "mock" });
    if profile.is_empty()
        || profile.len() > 64
        || !profile
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(Error::invalid());
    }
    let scenario: Scenario =
        serde_json::from_value(json!(cli.scenario.as_deref().unwrap_or("normal")))
            .map_err(|_| Error::invalid())?;
    let route: Route =
        serde_json::from_value(json!(cli.connection)).map_err(|_| Error::invalid())?;
    let wait_for: WaitFor =
        serde_json::from_value(json!(cli.wait)).map_err(|_| Error::invalid())?;
    let timeout_ms = cli.timeout_ms.unwrap_or(
        if matches!(
            cli.command,
            Commands::Account {
                command: accounts::AccountCommand::Connect { .. }
            }
        ) {
            180_000
        } else {
            10_000
        },
    );
    let operation = Operation::new(timeout_ms, cancellation.clone())?;
    let directory = match &cli.data_dir {
        Some(path) => path.clone(),
        None => std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
            .ok_or_else(|| {
                Error::new(
                    "STORAGE_ERROR",
                    "Set --data-dir to a private application directory.",
                )
            })?
            .join("homeverb"),
    }
    .join(profile);
    if let Commands::Account { command } = cli.command {
        return accounts::run(&directory, command, timeout_ms, cancellation, &grants).await;
    }
    let is_mcp = matches!(cli.command, Commands::Mcp { .. });
    let core = if meiju {
        homeverb_host::open_meiju_with_control_grants(
            &directory,
            &grants.meiju_power_grants,
            &grants.meiju_fan_grants,
        )?
    } else {
        homeverb_host::open_mock(&directory, scenario, !is_mcp)?
    };
    if is_mcp {
        return mcp::serve(std::sync::Arc::new(core), cancellation).await;
    }
    if let Commands::Batch { file } = &cli.command {
        if route != Route::Auto || wait_for != WaitFor::Verified || cli.request_id.is_some() {
            return Err(Error::new(
                "INVALID_ARGUMENT",
                "Batch files supply each command's route, wait condition, and request ID.",
            ));
        }
        let file = std::fs::File::open(file).map_err(|_| {
            Error::new(
                "INVALID_ARGUMENT",
                "The local batch file could not be opened.",
            )
        })?;
        let batch: BatchCommand = serde_json::from_reader(file).map_err(|_| Error::invalid())?;
        if cli.dry_run {
            let result = core.plan_batch(&batch, &operation).await;
            let code = result.as_ref().err().map_or(0, error_exit);
            print_json(&Query::from_result(result));
            return Ok(code);
        }
        let waits: Vec<_> = batch
            .commands
            .iter()
            .map(|command| command.wait_for)
            .collect();
        let result = core.execute_batch(batch, &operation).await?;
        let code = if result.status == Status::Partial {
            13
        } else {
            result
                .items
                .iter()
                .zip(waits)
                .map(|(item, wait)| execution_exit(item, wait))
                .find(|code| *code != 0)
                .unwrap_or(0)
        };
        print_json(&result);
        return Ok(if cancellation.is_cancelled() {
            130
        } else {
            code
        });
    }
    let write = match &cli.command {
        Commands::Power {
            command: PowerCommand::Set(set),
        } => Some((
            &set.device,
            "main",
            "power",
            "set",
            json!({"state": set.on}),
        )),
        Commands::Climate {
            command:
                ClimateCommand::Set {
                    device,
                    temperature,
                    mode,
                },
        } => {
            let mut arguments = serde_json::Map::new();
            if let Some(value) = temperature {
                arguments.insert("target_temperature".into(), json!(value));
            }
            if let Some(value) = mode {
                arguments.insert("mode".into(), json!(value));
            }
            Some((device, "main", "climate", "set", Value::Object(arguments)))
        }
        Commands::Device {
            command:
                DeviceCommand::Execute {
                    device,
                    component,
                    capability,
                    action,
                    arguments,
                },
        } => Some((
            device,
            component.as_str(),
            capability.as_str(),
            action.as_str(),
            serde_json::from_str(arguments).map_err(|_| Error::invalid())?,
        )),
        _ => None,
    };
    if let Some((selector, component, capability, action, arguments)) = write {
        let device = core.describe(selector)?.device;
        let command = Command {
            schema_version: SCHEMA_VERSION.into(),
            request_id: cli
                .request_id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            device_id: device.id,
            component_id: component.into(),
            capability: capability.into(),
            action: action.into(),
            arguments,
            connection: route,
            timeout_ms,
            wait_for,
        };
        if cli.dry_run {
            let result = core.plan(&command);
            let code = result.as_ref().err().map_or(0, error_exit);
            print_json(&Query::from_result(result));
            return Ok(code);
        }
        let execution = core.execute_with_operation(command, &operation).await;
        let code = execution_exit(&execution, wait_for);
        print_json(&execution);
        return Ok(if cancellation.is_cancelled() {
            130
        } else {
            code
        });
    }
    if cli.dry_run {
        return Err(Error::new(
            "INVALID_ARGUMENT",
            "--dry-run applies only to write commands.",
        ));
    }
    let query: Result<Value> = match cli.command {
        Commands::Device { command } => match command {
            DeviceCommand::List => core.list().map(|v| json!(v)),
            DeviceCommand::Cards => core.list_device_cards().map(|v| json!(v)),
            DeviceCommand::Preferences { device, settings } => {
                let id = core.resolve_device_id(&device)?;
                let input: PreferencesInput =
                    serde_json::from_str(&settings).map_err(|_| Error::invalid())?;
                core.set_device_preferences(&id, input).map(|v| json!(v))
            }
            DeviceCommand::Reorder { device_ids } => {
                core.reorder_devices(&device_ids).map(|v| json!(v))
            }
            DeviceCommand::Discover => core.discover(&operation).await.map(|v| json!(v)),
            DeviceCommand::Describe { device } => core.describe(&device).map(|v| json!(v)),
            DeviceCommand::State {
                device,
                component,
                cached,
                ..
            } => core
                .read(&device, &component, route, cached, &operation)
                .await
                .map(|v| json!(v)),
            DeviceCommand::Execute { .. } => unreachable!(),
        },
        Commands::Executions { device } => match device {
            Some(selector) => {
                let id = core.resolve_device_id(&selector)?;
                core.device_executions(&id).map(|v| json!(v))
            }
            None => core.recent_executions().map(|v| json!(v)),
        },
        Commands::Execution { receipt_id } => core
            .execution(&receipt_id, &operation)
            .await
            .map(|v| json!(v)),
        Commands::Diagnostics => core.diagnostics(&operation).await.map(|v| json!(v)),
        Commands::Room { command } => match command {
            RoomCommand::List => core.list_rooms().map(|v| json!(v)),
            RoomCommand::Create { name } => core.create_room(&name).map(|v| json!(v)),
            RoomCommand::Rename { room_id, name } => {
                core.rename_room(&room_id, &name).map(|v| json!(v))
            }
            RoomCommand::Delete { room_id } => core.delete_room(&room_id).map(|_| Value::Null),
            RoomCommand::Reorder { room_ids } => core.reorder_rooms(&room_ids).map(|v| json!(v)),
            RoomCommand::Assign {
                device_ids,
                room_id,
                ..
            } => core
                .assign_devices_to_room(RoomAssignment {
                    device_ids,
                    room_id,
                })
                .map(|v| json!(v)),
        },
        _ => unreachable!(),
    };
    let code = query.as_ref().err().map_or(0, error_exit);
    print_json(&Query::from_result(query));
    Ok(code)
}

fn print_json(value: &impl serde::Serialize) {
    println!(
        "{}",
        serde_json::to_string(value).expect("public DTOs serialize to JSON")
    );
}

fn error_exit(error: &Error) -> u8 {
    match error.code.as_str() {
        "INVALID_ARGUMENT" | "ROOM_NAME_CONFLICT" | "MIDEA_LOGIN_INPUT_UNSUPPORTED" => 2,
        "AUTH_REQUIRED"
        | "AUTH_EXPIRED"
        | "SECRET_STORE_UNAVAILABLE"
        | "SECRET_STORE_LOCKED"
        | "INPUT_UNAVAILABLE"
        | "ACCOUNT_SESSION_UNCONFIRMED"
        | "MIDEA_INVENTORY_REQUIRED" => 3,
        "DEVICE_NOT_FOUND"
        | "REQUEST_NOT_FOUND"
        | "DEVICE_IDENTITY_CONFLICT"
        | "CAPABILITY_UNSUPPORTED"
        | "AMBIGUOUS_DEVICE"
        | "ROOM_NOT_FOUND"
        | "ACCOUNT_NOT_FOUND"
        | "ACCOUNT_IDENTITY_CONFLICT"
        | "ACCOUNT_REMOVAL_UNSUPPORTED"
        | "AUTH_FLOW_UNSUPPORTED"
        | "MIDEA_DEVICE_NOT_LISTED" => 4,
        "LAN_UNREACHABLE" | "CLOUD_UNREACHABLE" | "DEADLINE_EXCEEDED" => 5,
        "POLICY_DENIED" => 6,
        "RESOURCE_BUSY"
        | "REQUEST_CONFLICT"
        | "REQUEST_RETIRED"
        | "SECRET_STORE_CONFLICT"
        | "ACCOUNT_SCOPE_CHANGED"
        | "ACCOUNT_REMOVAL_PENDING" => 7,
        "MIDEA_INVENTORY_PARTIAL" => 13,
        "CANCELLED" => 130,
        _ => 10,
    }
}

fn execution_exit(execution: &Execution, wait: WaitFor) -> u8 {
    match execution.status {
        Status::Verified => 0,
        Status::Accepted if wait == WaitFor::Accepted => 0,
        Status::Accepted => 12,
        Status::Unknown => 11,
        Status::Partial => 13,
        Status::Failed => execution.error.as_ref().map_or(10, error_exit),
    }
}
