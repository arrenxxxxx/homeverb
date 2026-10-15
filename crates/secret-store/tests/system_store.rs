use homeverb_core::{CredentialRef, Operation, SecretBytes, SecretStore};
use homeverb_secrets::SystemSecretStore;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

fn operation() -> Operation {
    Operation::new(10_000, CancellationToken::new()).unwrap()
}

#[test]
fn namespace_requires_an_opaque_random_id() {
    assert!(SystemSecretStore::new(Uuid::nil()).is_err());
    assert!(SystemSecretStore::new(Uuid::new_v4()).is_ok());
}

#[tokio::test]
async fn pre_cancelled_requests_never_connect_to_the_system_store() {
    let store = SystemSecretStore::new(Uuid::new_v4()).unwrap();
    let operation = operation();
    operation.cancellation.cancel();
    let reference = CredentialRef::new();
    let secret = SecretBytes::new(vec![7]);
    assert_eq!(
        store.get(&reference, &operation).await.unwrap_err().code,
        "CANCELLED"
    );
    assert_eq!(
        store
            .put(&reference, &secret, &operation)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    assert_eq!(
        store.delete(&reference, &operation).await.unwrap_err().code,
        "CANCELLED"
    );
}

#[tokio::test(start_paused = true)]
async fn expired_requests_never_connect_to_the_system_store() {
    let store = SystemSecretStore::new(Uuid::new_v4()).unwrap();
    let operation = operation();
    tokio::time::advance(std::time::Duration::from_secs(11)).await;
    let reference = CredentialRef::new();
    let secret = SecretBytes::new(vec![7]);
    assert_eq!(
        store.get(&reference, &operation).await.unwrap_err().code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(
        store
            .put(&reference, &secret, &operation)
            .await
            .unwrap_err()
            .code,
        "DEADLINE_EXCEEDED"
    );
    assert_eq!(
        store.delete(&reference, &operation).await.unwrap_err().code,
        "DEADLINE_EXCEEDED"
    );
}

/// Explicitly creates and removes synthetic entries in two fresh namespaces.
#[cfg(target_os = "linux")]
#[tokio::test]
#[ignore = "Requires an unlocked default Secret Service collection and an explicit local run"]
async fn linux_secret_service_round_trip() {
    use homeverb_domain::{Error, Result};
    use std::collections::BTreeSet;

    fn check(condition: bool) -> Result<()> {
        if condition {
            Ok(())
        } else {
            Err(Error::new(
                "TEST_ASSERTION",
                "The synthetic credential check failed.",
            ))
        }
    }
    fn sockets() -> BTreeSet<std::path::PathBuf> {
        std::fs::read_dir("/proc/self/fd")
            .unwrap()
            .filter_map(|entry| std::fs::read_link(entry.ok()?.path()).ok())
            .filter(|target| target.to_string_lossy().starts_with("socket:"))
            .collect()
    }

    let profile = Uuid::new_v4();
    let first = SystemSecretStore::new(profile).unwrap();
    let other = SystemSecretStore::new(Uuid::new_v4()).unwrap();
    let reference = CredentialRef::new();
    let original = SecretBytes::new(vec![17, 29, 0, 255]);
    let replacement = SecretBytes::new(vec![31, 41, 0, 128]);
    // Initialize Tokio's I/O driver before recording the owned socket baseline.
    assert!(first.get(&reference, &operation()).await.unwrap().is_none());
    tokio::task::yield_now().await;
    let before = sockets();

    let result: Result<()> = async {
        first.put(&reference, &original, &operation()).await?;
        let reopened = SystemSecretStore::new(profile)?;
        check(
            reopened
                .get(&reference, &operation())
                .await?
                .is_some_and(|value| value.expose() == original.expose()),
        )?;
        check(other.get(&reference, &operation()).await?.is_none())?;
        other.put(&reference, &original, &operation()).await?;
        first.put(&reference, &replacement, &operation()).await?;
        check(
            reopened
                .get(&reference, &operation())
                .await?
                .is_some_and(|value| value.expose() == replacement.expose()),
        )?;
        check(
            other
                .get(&reference, &operation())
                .await?
                .is_some_and(|value| value.expose() == original.expose()),
        )?;
        first.delete(&reference, &operation()).await?;
        check(first.get(&reference, &operation()).await?.is_none())?;
        check(other.get(&reference, &operation()).await?.is_some())?;
        first.delete(&reference, &operation()).await?;
        Ok(())
    }
    .await;

    // Cleanup runs for every returned failure, without printing any credential bytes.
    let clean_first = first.delete(&reference, &operation()).await;
    let clean_other = other.delete(&reference, &operation()).await;
    clean_first.expect("Synthetic entry cleanup failed for the first namespace");
    clean_other.expect("Synthetic entry cleanup failed for the second namespace");
    assert!(first.get(&reference, &operation()).await.unwrap().is_none());
    assert!(other.get(&reference, &operation()).await.unwrap().is_none());
    result.expect("Synthetic credential verification failed");
    // Dropping zbus aborts its Tokio reader; the runtime reclaims that task asynchronously.
    tokio::time::timeout(std::time::Duration::from_millis(100), async {
        while sockets().difference(&before).next().is_some() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("An operation retained a socket beyond bounded runtime cleanup");
}
