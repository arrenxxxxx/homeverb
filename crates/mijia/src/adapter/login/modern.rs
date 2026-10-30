//! Modern authentication and saved-session work share the existing protected account identity.
use super::*;
use crate::{
    ModernDirectory, ModernDirectoryEntry, ModernMijiaSession, ModernQrChallenge, PropertyReading,
};
use homeverb_core::CredentialRef;

/// Single-use foreground challenge bound to this adapter and protected account snapshot.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::PendingModernMijiaLogin) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct PendingModernMijiaLogin {
    owner: Uuid,
    existing: Option<Uuid>,
    label: String,
    expected: Vec<(Uuid, [u8; 32])>,
    challenge: ModernQrChallenge,
    lifetime: Operation,
}
impl fmt::Debug for PendingModernMijiaLogin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PendingModernMijiaLogin([REDACTED])")
    }
}
impl PendingModernMijiaLogin {
    pub fn scan_payload(&self) -> &[u8] {
        self.challenge.scan_payload()
    }
    pub fn expires_at(&self) -> tokio::time::Instant {
        self.lifetime.deadline
    }
}

/// Logical entry tied to one local account/credential reference; not a physical Device.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::ModernAccountEntry) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct ModernAccountEntry {
    account_id: Uuid,
    credential_ref: CredentialRef,
    entry: ModernDirectoryEntry,
}
impl fmt::Debug for ModernAccountEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernAccountEntry([REDACTED])")
    }
}
impl ModernAccountEntry {
    pub fn name(&self) -> &str {
        self.entry.name()
    }
    pub fn model(&self) -> &str {
        self.entry.model()
    }
    pub fn readable_properties(&self) -> Vec<PropertyQuery> {
        self.entry.readable_properties()
    }
}
pub struct ModernAccountDiscovery {
    pub account: AccountRecord,
    pub entries: Result<Vec<ModernAccountEntry>>,
    pub issues: Vec<Error>,
    pub complete: bool,
}
impl fmt::Debug for ModernAccountDiscovery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernAccountDiscovery([REDACTED])")
    }
}
fn project(record: AccountRecord, result: Result<ModernDirectory>) -> ModernAccountDiscovery {
    match result {
        Ok(directory) => ModernAccountDiscovery {
            entries: Ok(directory
                .entries
                .into_iter()
                .map(|entry| ModernAccountEntry {
                    account_id: record.id,
                    credential_ref: record.credential_ref,
                    entry,
                })
                .collect()),
            account: record,
            issues: directory.issues.into_iter().take(64).collect(),
            complete: false,
        },
        Err(error) => ModernAccountDiscovery {
            account: record,
            entries: Err(error),
            issues: vec![],
            complete: false,
        },
    }
}

impl MijiaAdapter {
    fn modern_protocol(
        &self,
    ) -> Result<(
        &dyn crate::ModernAuthTransport,
        &dyn crate::ModernReadTransport,
    )> {
        match (&self.modern_auth, &self.modern_reads) {
            (Some(auth), Some(reads)) => Ok((auth.as_ref(), reads.as_ref())),
            _ => Err(unsupported()),
        }
    }
    pub async fn begin_modern_connection(
        &self,
        existing: Option<Uuid>,
        label: &str,
        operation: &Operation,
    ) -> Result<PendingModernMijiaLogin> {
        let (auth, _) = self.modern_protocol()?;
        let label = homeverb_domain::local_name(label)?;
        let expected = self.login_preflight(existing, operation).await?;
        let challenge = crate::begin_modern_qr_login(auth, operation).await?;
        let mut lifetime = operation.clone();
        lifetime.deadline = challenge.expires_at();
        Ok(PendingModernMijiaLogin {
            owner: self.login_owner,
            existing,
            label,
            expected,
            challenge,
            lifetime,
        })
    }
    pub async fn continue_modern_connection(
        &self,
        pending: PendingModernMijiaLogin,
        operation: &Operation,
    ) -> Result<ModernAccountDiscovery> {
        let mut lifetime = pending.lifetime.clone();
        lifetime.deadline = lifetime.deadline.min(operation.deadline);
        lifetime
            .run(operation.run(async {
                let (auth, _) = self.modern_protocol()?;
                if pending.owner != self.login_owner {
                    return Err(unconfirmed());
                }
                if self.login_preflight(pending.existing, operation).await? != pending.expected {
                    return Err(unconfirmed());
                }
                let session = pending.challenge.finish(auth, operation).await?;
                lifetime.run(async { Ok(()) }).await?;
                operation.run(async { Ok(()) }).await?;
                let _registration = self.vault.lock_registration(operation).await?;
                self.vault.check_secure_storage(operation).await?;
                if let Some(id) = pending.existing {
                    self.account(id, operation).await?;
                }
                let accounts = self.load_accounts(pending.existing, operation).await?;
                if expected_sessions(&accounts) != pending.expected {
                    return Err(unconfirmed());
                }
                self.save_modern_authenticated(
                    session,
                    pending.existing,
                    &pending.label,
                    accounts,
                    &lifetime,
                    operation,
                )
                .await
            }))
            .await
    }
    async fn save_modern_authenticated(
        &self,
        session: ModernMijiaSession,
        existing: Option<Uuid>,
        label: &str,
        mut accounts: Vec<LoadedAccount>,
        lifetime: &Operation,
        operation: &Operation,
    ) -> Result<ModernAccountDiscovery> {
        let (_, reads) = self.modern_protocol()?;
        let subject = crate::modern::StoredModernSession::from_session(session);
        // A noncanonical old identifier is ambiguous; never create a second account for it.
        if accounts.iter().any(|a| {
            a.payload.subject() != subject.subject()
                && a.payload.subject().trim_start_matches('0') == subject.subject()
        }) {
            return Err(identity_conflict());
        }
        let matches: Vec<_> = accounts
            .iter()
            .enumerate()
            .filter(|(_, a)| a.payload.subject() == subject.subject())
            .map(|(i, _)| i)
            .collect();
        if matches.len() > 1 {
            return Err(identity_conflict());
        }
        let selected = if let Some(id) = existing {
            let index = accounts
                .iter()
                .position(|a| a.lease.record().id == id)
                .ok_or_else(unconfirmed)?;
            if matches.first() != Some(&index) {
                return Err(identity_conflict());
            }
            Some(index)
        } else {
            matches.first().copied()
        };
        let session = subject.session()?;
        let account = if let Some(index) = selected {
            let mut account = accounts.remove(index);
            unchanged(&account, operation).await?;
            lifetime.run(async { Ok(()) }).await?;
            account.payload.set_modern_session(session);
            let bytes = account.payload.encode(account.lease.record())?;
            if bytes.expose() == account.bytes.expose() {
                account.lease.confirm_session(&bytes, lifetime).await?;
            } else {
                account.lease.store_session(&bytes, lifetime).await?;
                account.bytes = bytes;
            }
            account
        } else {
            if accounts.len() >= 64 {
                return Err(unconfirmed());
            }
            lifetime.run(async { Ok(()) }).await?;
            let mut lease = self.vault.prepare(PROVIDER, "CN", label, lifetime).await?;
            if lease.inspect_session(operation).await?.is_some() {
                return Err(unconfirmed());
            }
            let payload = Payload::new_modern(lease.record(), session);
            let bytes = payload.encode(lease.record())?;
            operation.run(async { Ok(()) }).await?;
            lease.store_session(&bytes, lifetime).await?;
            LoadedAccount {
                lease,
                payload,
                bytes,
            }
        };
        lifetime.run(async { Ok(()) }).await?;
        let directory = account
            .payload
            .modern_session()?
            .directory(reads, operation)
            .await;
        lifetime.run(async { Ok(()) }).await?;
        Ok(project(account.lease.record().clone(), directory))
    }
    pub async fn refresh_modern_directory(
        &self,
        id: Uuid,
        operation: &Operation,
    ) -> Result<ModernAccountDiscovery> {
        self.load_modern_directory(id, false, operation).await
    }
    pub async fn recover_modern_directory(
        &self,
        id: Uuid,
        operation: &Operation,
    ) -> Result<ModernAccountDiscovery> {
        self.load_modern_directory(id, true, operation).await
    }
    async fn load_modern_directory(
        &self,
        id: Uuid,
        recover: bool,
        operation: &Operation,
    ) -> Result<ModernAccountDiscovery> {
        let (_, reads) = self.modern_protocol()?;
        let mut account =
            LoadedAccount::load(self.vault.lease(id, operation).await?, recover, operation).await?;
        let session = account.payload.modern_session()?;
        let result = async {
            let directory = session.directory(reads, operation).await?;
            if recover && account.lease.record().credential_state == CredentialState::Pending {
                account
                    .lease
                    .confirm_session(&account.bytes, operation)
                    .await?;
            }
            Ok(directory)
        }
        .await;
        Ok(project(account.lease.record().clone(), result))
    }
    pub async fn read_modern_entry(
        &self,
        entry: &ModernAccountEntry,
        queries: &[PropertyQuery],
        operation: &Operation,
    ) -> Result<Vec<PropertyReading>> {
        let (_, reads) = self.modern_protocol()?;
        let account = LoadedAccount::load(
            self.vault.lease(entry.account_id, operation).await?,
            false,
            operation,
        )
        .await?;
        if account.lease.record().credential_ref != entry.credential_ref {
            return Err(conflict());
        }
        account
            .payload
            .modern_session()?
            .properties(reads, &entry.entry, queries, operation)
            .await
    }
    /// Explicit renewal; no inventory, automatic retry or legacy-session replacement.
    pub async fn renew_modern_account(
        &self,
        id: Uuid,
        operation: &Operation,
    ) -> Result<AccountRecord> {
        let (auth, _) = self.modern_protocol()?;
        let mut account =
            LoadedAccount::load(self.vault.lease(id, operation).await?, false, operation).await?;
        let session = account
            .payload
            .modern_session()?
            .renew(auth, operation)
            .await?;
        unchanged(&account, operation).await?;
        account.payload.set_modern_session(session);
        account.save(operation).await?;
        Ok(account.lease.record().clone())
    }
}
async fn unchanged(account: &LoadedAccount, operation: &Operation) -> Result<()> {
    let current = account
        .lease
        .inspect_session(operation)
        .await?
        .ok_or_else(unconfirmed)?;
    if current.expose() != account.bytes.expose() {
        return Err(unconfirmed());
    }
    Ok(())
}
fn identity_conflict() -> Error {
    Error::new(
        "ACCOUNT_IDENTITY_CONFLICT",
        "Modern sign-in did not match one unambiguous protected Mijia account.",
    )
}
