//! Bind interactive authentication to its adapter, target identity and saved session.
use super::*;
use crate::{CaptchaChallenge, LoginOutcome};
use homeverb_core::SecretBytes;
use sha2::{Digest, Sha256};
use std::fmt;

mod verification;
pub use verification::{
    AccountLoginChallenge, PendingVerificationConnection, VerificationAccountLogin,
    VerificationAnswer,
};

pub enum AccountLogin {
    Connected(AccountDiscovery),
    Captcha(PendingMijiaLogin),
}
impl fmt::Debug for AccountLogin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AccountLogin([REDACTED])")
    }
}

/// Target and protected session evidence are immutable while the user answers.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::PendingMijiaLogin) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct PendingMijiaLogin {
    owner: Uuid,
    existing: Option<Uuid>,
    label: String,
    expected: Vec<(Uuid, [u8; 32])>,
    challenge: CaptchaChallenge,
}
impl fmt::Debug for PendingMijiaLogin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PendingMijiaLogin([REDACTED])")
    }
}
impl PendingMijiaLogin {
    pub fn image(&self) -> &[u8] {
        self.challenge.image()
    }
    pub fn media_type(&self) -> &'static str {
        self.challenge.media_type()
    }
    pub fn expires_at(&self) -> tokio::time::Instant {
        self.challenge.expires_at()
    }
}

impl MijiaAdapter {
    pub async fn begin_connection(
        &self,
        credentials: LoginCredentials,
        existing: Option<Uuid>,
        label: &str,
        operation: &Operation,
    ) -> Result<AccountLogin> {
        let label = homeverb_domain::local_name(label)?;
        let expected = self.login_preflight(existing, operation).await?;
        let outcome =
            crate::begin_login(self.transport.as_ref(), credentials, Region::Cn, operation).await?;
        self.finish_login(outcome, existing, label, expected, operation)
            .await
    }

    pub async fn continue_connection(
        &self,
        pending: PendingMijiaLogin,
        answer: SecretBytes,
        operation: &Operation,
    ) -> Result<AccountLogin> {
        if pending.owner != self.login_owner {
            return Err(unconfirmed());
        }
        if self.login_preflight(pending.existing, operation).await? != pending.expected {
            return Err(unconfirmed());
        }
        let outcome = pending
            .challenge
            .answer(self.transport.as_ref(), answer, operation)
            .await?;
        self.finish_login(
            outcome,
            pending.existing,
            pending.label,
            pending.expected,
            operation,
        )
        .await
    }

    async fn login_preflight(
        &self,
        existing: Option<Uuid>,
        operation: &Operation,
    ) -> Result<Vec<(Uuid, [u8; 32])>> {
        let _registration = self.vault.lock_registration(operation).await?;
        self.vault.check_secure_storage(operation).await?;
        if let Some(id) = existing {
            self.account(id, operation).await?;
        }
        let accounts = self.load_accounts(existing, operation).await?;
        Ok(expected_sessions(&accounts))
    }

    async fn finish_login(
        &self,
        outcome: LoginOutcome,
        existing: Option<Uuid>,
        label: String,
        expected: Vec<(Uuid, [u8; 32])>,
        operation: &Operation,
    ) -> Result<AccountLogin> {
        match outcome {
            LoginOutcome::Captcha(challenge) => Ok(AccountLogin::Captcha(PendingMijiaLogin {
                owner: self.login_owner,
                existing,
                label,
                expected,
                challenge,
            })),
            LoginOutcome::Connected(session) => {
                let _registration = self.vault.lock_registration(operation).await?;
                self.vault.check_secure_storage(operation).await?;
                if let Some(id) = existing {
                    self.account(id, operation).await?;
                }
                let accounts = self.load_accounts(existing, operation).await?;
                if expected_sessions(&accounts) != expected {
                    return Err(unconfirmed());
                }
                self.save_authenticated(session, existing, &label, accounts, operation)
                    .await
                    .map(AccountLogin::Connected)
            }
        }
    }
}

// An initially unselected login may still match a saved subject after STS. Capture
// all protected identities so removal/relogin during user input cannot be undone.
fn expected_sessions(accounts: &[LoadedAccount]) -> Vec<(Uuid, [u8; 32])> {
    let mut values: Vec<_> = accounts
        .iter()
        .map(|account| {
            (
                account.lease.record().id,
                Sha256::digest(account.bytes.expose()).into(),
            )
        })
        .collect();
    values.sort_unstable_by_key(|entry| entry.0);
    values
}
