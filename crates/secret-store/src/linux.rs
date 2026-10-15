use crate::SystemSecretStore;
use homeverb_core::{CredentialRef, Operation, SecretBytes, WriteProgress};
use homeverb_domain::{Error, Phase, Result};
use secret_service::{EncryptionType, SecretService};
use std::{collections::HashMap, future::Future, time::Duration};

pub(crate) enum Request<'a> {
    Get,
    Put(&'a SecretBytes),
    Delete,
}

fn unavailable() -> Error {
    Error::new(
        "SECRET_STORE_UNAVAILABLE",
        "The system credential store is unavailable.",
    )
}

fn backend_error(error: secret_service::Error) -> Error {
    // Backend errors may include private paths or service-provided text.
    match error {
        secret_service::Error::Locked => Error::new(
            "SECRET_STORE_LOCKED",
            "Unlock the system credential store before trying again.",
        ),
        _ => unavailable(),
    }
}

fn check_matches(count: usize) -> Result<()> {
    if count > 1 {
        Err(Error::new(
            "SECRET_STORE_CONFLICT",
            "Multiple entries match this credential reference.",
        ))
    } else {
        Ok(())
    }
}

async fn bounded<T>(
    operation: &Operation,
    progress: &WriteProgress,
    work: impl Future<Output = Result<T>>,
) -> Result<T> {
    let result = operation.run(work).await;
    if result.is_err() && progress.phase() != Phase::NotSent {
        Err(Error::new(
            "SECRET_STORE_WRITE_UNKNOWN",
            "The credential change may have completed; inspect the reference before recovery.",
        ))
    } else {
        result
    }
}

impl SystemSecretStore {
    pub(crate) async fn perform(
        &self,
        reference: &CredentialRef,
        request: Request<'_>,
        operation: &Operation,
    ) -> Result<Option<SecretBytes>> {
        let progress = WriteProgress::default();
        let mut connection = None;
        let result = bounded(operation, &progress, async {
            connection = Some(
                zbus::Connection::session()
                    .await
                    .map_err(|_| unavailable())?,
            );
            let service = SecretService::connect_with_existing(
                EncryptionType::Dh,
                connection
                    .as_ref()
                    .expect("The connection was initialized")
                    .clone(),
            )
            .await
            .map_err(backend_error)?;
            let collection = service
                .get_default_collection()
                .await
                .map_err(backend_error)?;
            collection.ensure_unlocked().await.map_err(backend_error)?;
            let profile = self.profile.to_string();
            let reference = reference.to_string();
            let attributes = HashMap::from([
                ("application", "homeverb"),
                ("profile", profile.as_str()),
                ("credential", reference.as_str()),
            ]);
            let mut matches = collection
                .search_items(attributes.clone())
                .await
                .map_err(backend_error)?;
            check_matches(matches.len())?;
            let item = matches.pop();
            if let Some(item) = &item {
                item.ensure_unlocked().await.map_err(backend_error)?;
            }
            match request {
                Request::Get => match item {
                    Some(item) => Ok(Some(SecretBytes::new(
                        item.get_secret().await.map_err(backend_error)?,
                    ))),
                    None => Ok(None),
                },
                Request::Put(secret) => {
                    // Recheck the shared budget after all preflight I/O, before mutation.
                    operation
                        .run(async {
                            progress.mark_sent();
                            collection
                                .create_item(
                                    "HomeVerb credential",
                                    attributes,
                                    secret.expose(),
                                    true,
                                    "application/octet-stream",
                                )
                                .await
                                .map_err(backend_error)?;
                            Ok(None)
                        })
                        .await
                }
                Request::Delete => {
                    if let Some(item) = item {
                        operation
                            .run(async {
                                progress.mark_sent();
                                item.delete().await.map_err(backend_error)
                            })
                            .await?;
                    }
                    Ok(None)
                }
            }
        })
        .await;

        if let Some(connection) = connection {
            // A cancelled/expired operation still releases its socket. Cleanup never retries a
            // credential mutation or replaces its known result. Drop follows the bounded close.
            let _ = tokio::time::timeout(Duration::from_millis(100), async {
                let _ = connection.clone().close().await;
                connection.graceful_shutdown().await;
            })
            .await;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::sync::CancellationToken;

    #[tokio::test(start_paused = true)]
    async fn deadline_distinguishes_read_or_preflight_from_started_mutation() {
        for sent in [false, true] {
            let operation = Operation::new(10, CancellationToken::new()).unwrap();
            let progress = WriteProgress::default();
            let result: Result<()> = bounded(&operation, &progress, async {
                if sent {
                    progress.mark_sent();
                }
                std::future::pending().await
            })
            .await;
            assert_eq!(
                result.unwrap_err().code,
                if sent {
                    "SECRET_STORE_WRITE_UNKNOWN"
                } else {
                    "DEADLINE_EXCEEDED"
                }
            );
        }
    }

    #[tokio::test]
    async fn cancellation_and_lost_reply_after_mutation_are_uncertain() {
        for cancel in [false, true] {
            let operation = Operation::new(1000, CancellationToken::new()).unwrap();
            let progress = WriteProgress::default();
            let result: Result<()> = bounded(&operation, &progress, async {
                progress.mark_sent();
                if cancel {
                    operation.cancellation.cancel();
                    std::future::pending().await
                } else {
                    Err(unavailable())
                }
            })
            .await;
            let error = result.unwrap_err();
            assert_eq!(error.code, "SECRET_STORE_WRITE_UNKNOWN");
            assert!(!error.retryable);
        }
    }

    #[tokio::test]
    async fn confirmed_mutation_preserves_success() {
        let operation = Operation::new(1000, CancellationToken::new()).unwrap();
        let progress = WriteProgress::default();
        let result = bounded(&operation, &progress, async {
            progress.mark_sent();
            Ok(())
        })
        .await;
        assert!(result.is_ok());
    }

    #[test]
    fn locked_missing_and_duplicate_stores_fail_closed_without_backend_text() {
        assert_eq!(
            backend_error(secret_service::Error::Locked).code,
            "SECRET_STORE_LOCKED"
        );
        for error in [
            secret_service::Error::Unavailable,
            secret_service::Error::NoResult,
            secret_service::Error::Crypto("private backend text"),
        ] {
            let sanitized = backend_error(error);
            assert_eq!(sanitized.code, "SECRET_STORE_UNAVAILABLE");
            assert!(!sanitized.message.contains("private backend text"));
            assert!(!sanitized.retryable);
        }
        assert!(check_matches(0).is_ok());
        assert!(check_matches(1).is_ok());
        assert_eq!(check_matches(2).unwrap_err().code, "SECRET_STORE_CONFLICT");
    }
}
