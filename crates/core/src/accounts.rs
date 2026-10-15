//! Trusted native account metadata and leased secure sessions. No login mechanism lives here.

use crate::{CredentialRef, Operation, SecretBytes, SecretStore};
use homeverb_domain::{Error, Result, local_name};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::{Uuid, Variant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialState {
    Pending,
    Stored,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountRecord {
    pub id: Uuid,
    pub provider: String,
    pub region: String,
    pub label: String,
    pub credential_ref: CredentialRef,
    pub credential_state: CredentialState,
}

impl AccountRecord {
    pub fn validate(&self) -> Result<()> {
        valid_account_id(self.id)?;
        if self.provider.is_empty()
            || self.provider.len() > 80
            || !self
                .provider
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
            || self.region.is_empty()
            || self.region.len() > 32
            || !self
                .region
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            || local_name(&self.label)? != self.label
        {
            return Err(Error::invalid());
        }
        Ok(())
    }
}

pub fn valid_account_id(id: Uuid) -> Result<()> {
    if id.get_version_num() != 4 || id.get_variant() != Variant::RFC4122 {
        return Err(Error::invalid());
    }
    Ok(())
}

/// Retained across Send adapter futures, including immutable leased session reads.
pub trait AccountLock: Send + Sync {}

pub trait AccountStore: Send + Sync {
    fn profile_namespace(&self) -> Result<Uuid>;
    fn accounts(&self) -> Result<Vec<AccountRecord>>;
    fn account(&self, id: Uuid) -> Result<AccountRecord>;
    fn insert_account(&self, account: &AccountRecord) -> Result<()>;
    fn lock_account(&self, id: Uuid) -> Result<Box<dyn AccountLock>>;
    fn set_credential_state(&self, id: Uuid, state: CredentialState) -> Result<()>;
    fn rename_account(&self, id: Uuid, label: &str) -> Result<()>;
}

pub struct AccountVault {
    store: Arc<dyn AccountStore>,
    secrets: Arc<dyn SecretStore>,
}

impl AccountVault {
    pub fn new(store: Arc<dyn AccountStore>, secrets: Arc<dyn SecretStore>) -> Self {
        Self { store, secrets }
    }

    pub async fn list(&self, operation: &Operation) -> Result<Vec<AccountRecord>> {
        operation.run(async { self.store.accounts() }).await
    }

    /// Coordinate explicit account registration/reconnection across native hosts in this profile.
    pub async fn lock_registration(&self, operation: &Operation) -> Result<Box<dyn AccountLock>> {
        operation
            .run(async { self.store.lock_account(self.store.profile_namespace()?) })
            .await
    }

    /// Check secure-store access without creating an account or reading an existing credential.
    /// A successful read preflight cannot guarantee that a later mutation will succeed.
    pub async fn check_secure_storage(&self, operation: &Operation) -> Result<()> {
        self.secrets.get(&CredentialRef::new(), operation).await?;
        Ok(())
    }

    /// Reserve stable local references before attempting a secure-store mutation.
    pub async fn prepare(
        &self,
        provider: &str,
        region: &str,
        label: &str,
        operation: &Operation,
    ) -> Result<AccountLease> {
        operation
            .run(async {
                let record = AccountRecord {
                    id: Uuid::new_v4(),
                    provider: provider.into(),
                    region: region.into(),
                    label: local_name(label)?,
                    credential_ref: CredentialRef::new(),
                    credential_state: CredentialState::Pending,
                };
                record.validate()?;
                let guard = self.store.lock_account(record.id)?;
                self.store.insert_account(&record)?;
                Ok(AccountLease {
                    record,
                    store: self.store.clone(),
                    secrets: self.secrets.clone(),
                    _guard: guard,
                })
            })
            .await
    }

    /// Acquire the process lock before rereading the latest stored metadata/session.
    pub async fn lease(&self, id: Uuid, operation: &Operation) -> Result<AccountLease> {
        operation
            .run(async {
                valid_account_id(id)?;
                let guard = self.store.lock_account(id)?;
                let record = self.store.account(id)?;
                Ok(AccountLease {
                    record,
                    store: self.store.clone(),
                    secrets: self.secrets.clone(),
                    _guard: guard,
                })
            })
            .await
    }
}

pub struct AccountLease {
    record: AccountRecord,
    store: Arc<dyn AccountStore>,
    secrets: Arc<dyn SecretStore>,
    _guard: Box<dyn AccountLock>,
}

impl AccountLease {
    pub fn record(&self) -> &AccountRecord {
        &self.record
    }

    pub async fn read_session(&self, operation: &Operation) -> Result<SecretBytes> {
        operation
            .run(async {
                if self.record.credential_state != CredentialState::Stored {
                    return Err(Error::new(
                        "ACCOUNT_SESSION_UNCONFIRMED",
                        "The account session requires explicit validation before reuse.",
                    ));
                }
                self.inspect_session(operation).await?.ok_or_else(|| {
                    Error::new(
                        "AUTH_REQUIRED",
                        "The account has no locally stored session.",
                    )
                })
            })
            .await
    }

    /// Trusted recovery only. Finding bytes does not establish provider validity.
    pub async fn inspect_session(&self, operation: &Operation) -> Result<Option<SecretBytes>> {
        self.secrets
            .get(&self.record.credential_ref, operation)
            .await
    }

    /// Call only with credentials validated for this exact provider/account by native code.
    pub async fn store_session(
        &mut self,
        secret: &SecretBytes,
        operation: &Operation,
    ) -> Result<()> {
        operation
            .run(async {
                self.store
                    .set_credential_state(self.record.id, CredentialState::Pending)?;
                self.record.credential_state = CredentialState::Pending;
                Ok(())
            })
            .await?;
        self.secrets
            .put(&self.record.credential_ref, secret, operation)
            .await?;
        operation
            .run(async {
                self.store
                    .set_credential_state(self.record.id, CredentialState::Stored)?;
                self.record.credential_state = CredentialState::Stored;
                Ok(())
            })
            .await
    }

    /// Recover only after native provider validation of these exact bytes while holding this lease.
    pub async fn confirm_session(
        &mut self,
        validated: &SecretBytes,
        operation: &Operation,
    ) -> Result<()> {
        let current = self.inspect_session(operation).await?;
        if current
            .as_ref()
            .is_none_or(|secret| secret.expose() != validated.expose())
        {
            return Err(Error::new(
                "ACCOUNT_SESSION_UNCONFIRMED",
                "The stored session differs from the validated session.",
            ));
        }
        operation
            .run(async {
                self.store
                    .set_credential_state(self.record.id, CredentialState::Stored)?;
                self.record.credential_state = CredentialState::Stored;
                Ok(())
            })
            .await
    }

    pub async fn rename(&mut self, label: &str, operation: &Operation) -> Result<()> {
        operation
            .run(async {
                let label = local_name(label)?;
                self.store.rename_account(self.record.id, &label)?;
                self.record.label = label;
                Ok(())
            })
            .await
    }
}
