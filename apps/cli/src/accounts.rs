//! CLI account operations reuse the native host without exposing credential JSON.

use clap::Subcommand;
use homeverb_core::Operation;
use homeverb_domain::{Error, Result, SCHEMA_VERSION, local_name};
use homeverb_host::{ConnectRequest, HostRequest, Request, Session};
use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Subcommand)]
pub enum AccountCommand {
    /// Log in through hidden terminal input and retrieve bound devices.
    Connect {
        #[arg(long)]
        label: String,
        #[arg(long)]
        account_id: Option<Uuid>,
    },
    List,
    Show {
        account_id: Uuid,
    },
    /// Inspect saved protected membership without contacting the provider.
    Inventory {
        account_id: Uuid,
    },
    /// Retrieve devices using saved authorization; never logs in automatically.
    Refresh {
        account_id: Uuid,
    },
    /// Validate interrupted saved authorization without a new login.
    Recover {
        account_id: Uuid,
    },
    Rename {
        account_id: Uuid,
        label: String,
    },
    /// Remove local authorization and the exact reviewed linked-card set.
    Remove {
        account_id: Uuid,
        #[arg(long = "expected-device-id")]
        expected_device_ids: Vec<String>,
    },
}

pub async fn run(
    directory: &Path,
    command: AccountCommand,
    timeout_ms: u64,
    cancellation: CancellationToken,
) -> Result<u8> {
    if let AccountCommand::Connect { label, account_id } = &command {
        local_name(label)?;
        if let Some(id) = account_id {
            homeverb_core::valid_account_id(*id)?;
        }
        super::terminal::require_interactive()?;
    }
    let session = homeverb_host::open_meiju_session(directory)?;
    let operation_id = Uuid::new_v4().to_string();
    let response = match command {
        AccountCommand::Connect { label, account_id } => {
            homeverb_host::open_accounts(directory)?
                .check_secure_storage(&Operation::new(10_000, cancellation.clone())?)
                .await?;
            // Human input precedes the bounded native login/inventory operation.
            let login = super::terminal::input("Meiju account (hidden): ", &cancellation).await?;
            let password =
                super::terminal::input("Meiju password (hidden): ", &cancellation).await?;
            complete(
                &session,
                session.connect_account(
                    ConnectRequest {
                        schema_version: SCHEMA_VERSION.into(),
                        operation_id,
                        timeout_ms,
                        label,
                        account_id,
                    },
                    login,
                    password,
                ),
                &cancellation,
            )
            .await?
        }
        command => {
            let request = match command {
                AccountCommand::List => Request::ListAccounts {},
                AccountCommand::Show { account_id } => Request::GetAccount { account_id },
                AccountCommand::Inventory { account_id } => {
                    Request::InspectAccountInventory { account_id }
                }
                AccountCommand::Refresh { account_id } => Request::RefreshAccount { account_id },
                AccountCommand::Recover { account_id } => Request::RecoverAccount { account_id },
                AccountCommand::Rename { account_id, label } => {
                    Request::RenameAccount { account_id, label }
                }
                AccountCommand::Remove {
                    account_id,
                    expected_device_ids,
                } => Request::RemoveAccount {
                    account_id,
                    expected_device_ids,
                },
                AccountCommand::Connect { .. } => unreachable!(),
            };
            complete(
                &session,
                session.call(HostRequest {
                    schema_version: SCHEMA_VERSION.into(),
                    operation_id,
                    timeout_ms,
                    request,
                }),
                &cancellation,
            )
            .await?
        }
    };
    let code = response_exit(&response);
    super::print_json(&response);
    Ok(code)
}

async fn complete(
    session: &Session,
    work: impl std::future::Future<Output = Value>,
    cancellation: &CancellationToken,
) -> Result<Value> {
    if cancellation.is_cancelled() {
        return Err(Error::new(
            "CANCELLED",
            "The account operation was cancelled.",
        ));
    }
    tokio::pin!(work);
    let response = tokio::select! {
        biased;
        response = &mut work => response,
        _ = cancellation.cancelled() => {
            session.close();
            // Preserve confirmed or uncertain mutations in the native result.
            work.await
        }
    };
    session.close();
    Ok(response)
}

fn response_exit(response: &Value) -> u8 {
    for pointer in ["/error", "/data/retrieval/error", "/data/error"] {
        if let Some(error) = response.pointer(pointer).filter(|error| !error.is_null()) {
            return serde_json::from_value(error.clone())
                .map_or(10, |error| super::error_exit(&error));
        }
    }
    if response.pointer("/data/status").and_then(Value::as_str) == Some("pending") {
        return 10;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn cancellation_reaches_the_host_and_preserves_an_uncertain_result() {
        use homeverb_domain::{Command, Route, WaitFor};
        let directory = tempfile::tempdir().unwrap();
        let core = homeverb_host::open_mock(
            directory.path(),
            homeverb_mock::Scenario::SlowAfterSend,
            true,
        )
        .unwrap();
        core.discover(&Operation::new(1000, CancellationToken::new()).unwrap())
            .await
            .unwrap();
        let session = Session::new(core);
        let cancellation = CancellationToken::new();
        let command = Command {
            schema_version: SCHEMA_VERSION.into(),
            request_id: "fictional-write".into(),
            device_id: homeverb_mock::DEVICE_ID.into(),
            component_id: "main".into(),
            capability: "power".into(),
            action: "set".into(),
            arguments: json!({"state":true}),
            connection: Route::Auto,
            timeout_ms: 1000,
            wait_for: WaitFor::Verified,
        };
        let work = complete(
            &session,
            session.call(HostRequest {
                schema_version: SCHEMA_VERSION.into(),
                operation_id: "fictional-operation".into(),
                timeout_ms: 1000,
                request: Request::Execute { command },
            }),
            &cancellation,
        );
        tokio::pin!(work);
        tokio::select! { biased;
            _ = &mut work => panic!("The fictional write should await an acknowledgement."),
            _ = tokio::task::yield_now() => {}
        }
        cancellation.cancel();
        let response = work.await.unwrap();
        assert_eq!(response["status"], "unknown");
        assert_eq!(response["attempts"].as_array().unwrap().len(), 1);
        let after = session
            .call(HostRequest {
                schema_version: SCHEMA_VERSION.into(),
                operation_id: "after-close".into(),
                timeout_ms: 1000,
                request: Request::ListAccounts {},
            })
            .await;
        assert_eq!(after["error"]["code"], "HOST_CLOSED");
        let never_polled = async { panic!("Cancelled work must not start.") };
        assert_eq!(
            complete(&session, never_polled, &cancellation)
                .await
                .err()
                .unwrap()
                .code,
            "CANCELLED"
        );
    }

    #[test]
    fn nested_outcomes_control_exit_without_hiding_saved_authorization() {
        for (pointer, code, expected) in [
            ("/error", "ACCOUNT_NOT_FOUND", 4),
            ("/data/retrieval/error", "AUTH_REQUIRED", 3),
            ("/data/retrieval/error", "MIDEA_INVENTORY_PARTIAL", 13),
            ("/data/error", "SECRET_STORE_WRITE_UNKNOWN", 10),
            ("/data/error", "CANCELLED", 130),
            ("/data/error", "SECRET_STORE_LOCKED", 3),
        ] {
            let mut response =
                json!({"error":null,"data":{"retrieval":{"error":null},"error":null}});
            *response.pointer_mut(pointer).unwrap() =
                serde_json::to_value(Error::new(code, "Fictional failure.")).unwrap();
            assert_eq!(response_exit(&response), expected);
        }
        assert_eq!(
            response_exit(&json!({"error":null,"data":{"status":"pending","error":null}})),
            10
        );
        assert_eq!(
            response_exit(&json!({"error":null,"data":{"status":"removed","error":null}})),
            0
        );
        assert_eq!(response_exit(&json!({"error":null,"data":[]})), 0);
    }
}
