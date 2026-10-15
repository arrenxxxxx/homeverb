use async_trait::async_trait;
use homeverb_core::*;
use homeverb_domain::{Error, Result};
use homeverb_storage::SqliteStore;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, AtomicUsize, Ordering},
    },
};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("homeverb-account-test-{}", Uuid::new_v4())))
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Default)]
struct Secrets {
    values: Mutex<HashMap<String, Vec<u8>>>,
    mode: AtomicU8,
    puts: AtomicUsize,
}

#[async_trait]
impl SecretStore for Secrets {
    async fn get(
        &self,
        reference: &CredentialRef,
        operation: &Operation,
    ) -> Result<Option<SecretBytes>> {
        operation
            .run(async {
                Ok(self
                    .values
                    .lock()
                    .unwrap()
                    .get(&reference.to_string())
                    .map(|bytes| SecretBytes::new(bytes.clone())))
            })
            .await
    }
    async fn put(
        &self,
        reference: &CredentialRef,
        secret: &SecretBytes,
        operation: &Operation,
    ) -> Result<()> {
        operation
            .run(async {
                self.puts.fetch_add(1, Ordering::Relaxed);
                if self.mode.load(Ordering::Relaxed) == 1 {
                    return Err(Error::new(
                        "SECRET_STORE_UNAVAILABLE",
                        "Synthetic unavailable store.",
                    ));
                }
                self.values
                    .lock()
                    .unwrap()
                    .insert(reference.to_string(), secret.expose().to_vec());
                if self.mode.load(Ordering::Relaxed) == 2 {
                    return Err(Error::new(
                        "SECRET_STORE_WRITE_UNKNOWN",
                        "Synthetic lost store acknowledgement.",
                    ));
                }
                if self.mode.load(Ordering::Relaxed) == 3 {
                    operation.cancellation.cancel();
                    return Err(Error::new(
                        "SECRET_STORE_WRITE_UNKNOWN",
                        "Synthetic cancellation after mutation.",
                    ));
                }
                Ok(())
            })
            .await
    }
    async fn delete(&self, _: &CredentialRef, _: &Operation) -> Result<()> {
        panic!("The vault must not implicitly delete credentials.")
    }
}

fn operation() -> Operation {
    Operation::new(1000, CancellationToken::new()).unwrap()
}

#[tokio::test]
async fn profile_account_and_credential_references_survive_reopen_without_plaintext_sessions() {
    let directory = Directory::new();
    let secrets = Arc::new(Secrets::default());
    let store = Arc::new(SqliteStore::open(&directory.0).unwrap());
    let namespace = store.profile_namespace().unwrap();
    let vault = AccountVault::new(store.clone(), secrets.clone());
    let mut lease = vault
        .prepare("midea.meiju", "CN", "Home", &operation())
        .await
        .unwrap();
    let record = lease.record().clone();
    assert_eq!(record.credential_state, CredentialState::Pending);
    assert_eq!(
        lease.read_session(&operation()).await.unwrap_err().code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    let secret = SecretBytes::new(b"PRIVATE-SESSION-BYTES".to_vec());
    lease.store_session(&secret, &operation()).await.unwrap();
    lease.rename("  Main Home  ", &operation()).await.unwrap();
    assert_eq!(
        lease.read_session(&operation()).await.unwrap().expose(),
        secret.expose()
    );
    drop(lease);
    drop(vault);
    drop(store);
    let reopened = Arc::new(SqliteStore::open(&directory.0).unwrap());
    assert_eq!(reopened.profile_namespace().unwrap(), namespace);
    let vault = AccountVault::new(reopened, secrets);
    let lease = vault.lease(record.id, &operation()).await.unwrap();
    assert_eq!(lease.record().credential_ref, record.credential_ref);
    assert_eq!(lease.record().provider, record.provider);
    assert_eq!(lease.record().region, record.region);
    assert_eq!(lease.record().label, "Main Home");
    assert_eq!(
        lease.read_session(&operation()).await.unwrap().expose(),
        secret.expose()
    );
    for filename in ["homeverb.sqlite3", "homeverb.sqlite3-wal"] {
        if let Ok(bytes) = std::fs::read(directory.0.join(filename)) {
            assert!(
                !bytes
                    .windows(secret.expose().len())
                    .any(|part| part == secret.expose())
            );
        }
    }
    let other_directory = Directory::new();
    let other = SqliteStore::open(&other_directory.0).unwrap();
    assert_ne!(other.profile_namespace().unwrap(), namespace);
    assert!(other.accounts().unwrap().is_empty());
    assert_eq!(
        other.account(record.id).unwrap_err().code,
        "ACCOUNT_NOT_FOUND"
    );
}

#[tokio::test]
async fn competing_vaults_lock_before_rereading_the_latest_session() {
    let directory = Directory::new();
    let secrets = Arc::new(Secrets::default());
    let first = AccountVault::new(
        Arc::new(SqliteStore::open(&directory.0).unwrap()),
        secrets.clone(),
    );
    let second = AccountVault::new(Arc::new(SqliteStore::open(&directory.0).unwrap()), secrets);
    let mut lease = first
        .prepare("midea.meiju", "CN", "Home", &operation())
        .await
        .unwrap();
    let id = lease.record().id;
    assert_eq!(
        second.lease(id, &operation()).await.err().unwrap().code,
        "RESOURCE_BUSY"
    );
    lease
        .store_session(&SecretBytes::new(b"latest-session".to_vec()), &operation())
        .await
        .unwrap();
    drop(lease);
    let next = second.lease(id, &operation()).await.unwrap();
    assert_eq!(
        next.read_session(&operation()).await.unwrap().expose(),
        b"latest-session"
    );
    assert_eq!(
        first.lease(id, &operation()).await.err().unwrap().code,
        "RESOURCE_BUSY"
    );
}

#[tokio::test]
async fn uncertain_mutations_remain_pending_and_recover_without_automatic_rewrite() {
    let directory = Directory::new();
    let secrets = Arc::new(Secrets::default());
    secrets.mode.store(2, Ordering::Relaxed);
    let store = Arc::new(SqliteStore::open(&directory.0).unwrap());
    let vault = AccountVault::new(store.clone(), secrets.clone());
    let mut lease = vault
        .prepare("midea.meiju", "CN", "Home", &operation())
        .await
        .unwrap();
    let id = lease.record().id;
    let bytes = SecretBytes::new(b"validated-test-session".to_vec());
    assert_eq!(
        lease
            .store_session(&bytes, &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    drop(lease);
    let mut recovered = vault.lease(id, &operation()).await.unwrap();
    assert_eq!(
        recovered.read_session(&operation()).await.unwrap_err().code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(
        recovered
            .inspect_session(&operation())
            .await
            .unwrap()
            .unwrap()
            .expose(),
        bytes.expose()
    );
    assert_eq!(
        recovered
            .confirm_session(&SecretBytes::new(b"different".to_vec()), &operation())
            .await
            .unwrap_err()
            .code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    recovered
        .confirm_session(&bytes, &operation())
        .await
        .unwrap();
    assert_eq!(
        recovered.read_session(&operation()).await.unwrap().expose(),
        bytes.expose()
    );
    assert_eq!(secrets.puts.load(Ordering::Relaxed), 1);
    assert_eq!(
        store.account(id).unwrap().credential_state,
        CredentialState::Stored
    );
}

#[tokio::test]
async fn unavailable_secure_store_never_falls_back_to_plaintext() {
    let directory = Directory::new();
    let store = Arc::new(SqliteStore::open(&directory.0).unwrap());
    let vault = AccountVault::new(store.clone(), Arc::new(UnavailableSecretStore));
    let mut lease = vault
        .prepare("midea.meiju", "CN", "Home", &operation())
        .await
        .unwrap();
    assert_eq!(
        lease
            .store_session(&SecretBytes::new(b"PRIVATE-UNSAVED".to_vec()), &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_UNAVAILABLE"
    );
    assert_eq!(
        store.account(lease.record().id).unwrap().credential_state,
        CredentialState::Pending
    );
    assert_eq!(
        lease.read_session(&operation()).await.unwrap_err().code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
}

#[tokio::test]
async fn metadata_failures_do_not_start_a_secret_write_or_falsely_confirm_one() {
    let directory = Directory::new();
    let secrets = Arc::new(Secrets::default());
    let store = Arc::new(SqliteStore::open(&directory.0).unwrap());
    let vault = AccountVault::new(store.clone(), secrets.clone());
    let mut lease = vault
        .prepare("midea.meiju", "CN", "Home", &operation())
        .await
        .unwrap();
    let old = SecretBytes::new(b"old-session".to_vec());
    lease.store_session(&old, &operation()).await.unwrap();
    let database = rusqlite::Connection::open(directory.0.join("homeverb.sqlite3")).unwrap();
    database.execute_batch("CREATE TRIGGER fail_pending BEFORE UPDATE OF credential_state ON accounts
        WHEN NEW.credential_state='pending' BEGIN SELECT RAISE(FAIL, 'Synthetic pending failure'); END;").unwrap();
    let new = SecretBytes::new(b"replacement-session".to_vec());
    assert_eq!(
        lease
            .store_session(&new, &operation())
            .await
            .unwrap_err()
            .code,
        "STORAGE_ERROR"
    );
    assert_eq!(secrets.puts.load(Ordering::Relaxed), 1);
    assert_eq!(
        lease.read_session(&operation()).await.unwrap().expose(),
        old.expose()
    );
    database.execute_batch("DROP TRIGGER fail_pending;
        CREATE TRIGGER fail_stored BEFORE UPDATE OF credential_state ON accounts
        WHEN NEW.credential_state='stored' BEGIN SELECT RAISE(FAIL, 'Synthetic finalization failure'); END;").unwrap();
    assert_eq!(
        lease
            .store_session(&new, &operation())
            .await
            .unwrap_err()
            .code,
        "STORAGE_ERROR"
    );
    let id = lease.record().id;
    drop(lease);
    assert_eq!(
        store.account(id).unwrap().credential_state,
        CredentialState::Pending
    );
    let mut lease = vault.lease(id, &operation()).await.unwrap();
    assert_eq!(
        lease.read_session(&operation()).await.unwrap_err().code,
        "ACCOUNT_SESSION_UNCONFIRMED"
    );
    assert_eq!(
        lease
            .inspect_session(&operation())
            .await
            .unwrap()
            .unwrap()
            .expose(),
        new.expose()
    );
    database.execute_batch("DROP TRIGGER fail_stored").unwrap();
    lease.confirm_session(&new, &operation()).await.unwrap();
    assert_eq!(secrets.puts.load(Ordering::Relaxed), 2);
    secrets.values.lock().unwrap().clear();
    assert_eq!(
        lease.read_session(&operation()).await.unwrap_err().code,
        "AUTH_REQUIRED"
    );
}

#[tokio::test]
async fn cancellation_before_and_after_mutation_never_promotes_an_unconfirmed_session() {
    let directory = Directory::new();
    let secrets = Arc::new(Secrets::default());
    let store = Arc::new(SqliteStore::open(&directory.0).unwrap());
    let vault = AccountVault::new(store.clone(), secrets.clone());
    let cancelled = operation();
    cancelled.cancellation.cancel();
    assert_eq!(
        vault
            .prepare("midea.meiju", "CN", "Home", &cancelled)
            .await
            .err()
            .unwrap()
            .code,
        "CANCELLED"
    );
    assert!(store.accounts().unwrap().is_empty());
    let mut lease = vault
        .prepare("midea.meiju", "CN", "Home", &operation())
        .await
        .unwrap();
    let secret = SecretBytes::new(b"session".to_vec());
    assert_eq!(
        lease
            .store_session(&secret, &cancelled)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    assert_eq!(secrets.puts.load(Ordering::Relaxed), 0);
    secrets.mode.store(3, Ordering::Relaxed);
    assert_eq!(
        lease
            .store_session(&secret, &operation())
            .await
            .unwrap_err()
            .code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert_eq!(
        store.account(lease.record().id).unwrap().credential_state,
        CredentialState::Pending
    );
}

#[tokio::test]
async fn trusted_system_vault_opens_and_lists_without_keyring_access() {
    let directory = Directory::new();
    let vault = homeverb_host::open_accounts(&directory.0).unwrap();
    assert!(vault.list(&operation()).await.unwrap().is_empty());
    assert_eq!(
        vault
            .lease(Uuid::nil(), &operation())
            .await
            .err()
            .unwrap()
            .code,
        "INVALID_ARGUMENT"
    );
    for (provider, region, label) in [
        ("bad provider", "CN", "Home"),
        ("midea.meiju", "", "Home"),
        ("midea.meiju", "CN", " "),
    ] {
        assert_eq!(
            vault
                .prepare(provider, region, label, &operation())
                .await
                .err()
                .unwrap()
                .code,
            "INVALID_ARGUMENT"
        );
    }
    assert!(vault.list(&operation()).await.unwrap().is_empty());
}
