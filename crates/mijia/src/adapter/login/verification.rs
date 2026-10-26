//! Opt-in verification bound to the original adapter and protected account snapshot.
use super::*;
use crate::{VerificationChallenge, VerificationLoginOutcome, VerificationMethod};

pub enum VerificationAccountLogin {
    Connected(AccountDiscovery),
    Pending(PendingVerificationConnection),
}
impl fmt::Debug for VerificationAccountLogin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VerificationAccountLogin([REDACTED])")
    }
}

/// Borrowed protected presentation data, only for a trusted foreground UI.
pub enum AccountLoginChallenge<'a> {
    Captcha {
        image: &'a [u8],
        media_type: &'static str,
    },
    Verification {
        provider_page: &'a str,
        methods: &'a [VerificationMethod],
    },
}
impl fmt::Debug for AccountLoginChallenge<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AccountLoginChallenge([REDACTED])")
    }
}

pub enum VerificationAnswer {
    Captcha(SecretBytes),
    Ticket {
        method: VerificationMethod,
        ticket: SecretBytes,
    },
}
impl fmt::Debug for VerificationAnswer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VerificationAnswer([REDACTED])")
    }
}

enum Challenge {
    Captcha(CaptchaChallenge),
    Verification(VerificationChallenge),
}

/// Single-use account-bound challenge; no leases are held while the caller waits.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::PendingVerificationConnection) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct PendingVerificationConnection {
    owner: Uuid,
    existing: Option<Uuid>,
    label: String,
    expected: Vec<(Uuid, [u8; 32])>,
    challenge: Challenge,
}
impl fmt::Debug for PendingVerificationConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PendingVerificationConnection([REDACTED])")
    }
}
impl PendingVerificationConnection {
    pub fn expires_at(&self) -> tokio::time::Instant {
        match &self.challenge {
            Challenge::Captcha(c) => c.expires_at(),
            Challenge::Verification(c) => c.expires_at(),
        }
    }
    pub fn challenge(&self) -> AccountLoginChallenge<'_> {
        match &self.challenge {
            Challenge::Captcha(c) => AccountLoginChallenge::Captcha {
                image: c.image(),
                media_type: c.media_type(),
            },
            Challenge::Verification(c) => AccountLoginChallenge::Verification {
                provider_page: c.provider_page(),
                methods: c.methods(),
            },
        }
    }
}

impl MijiaAdapter {
    /// Opt-in protected account path; legacy connection methods stay captcha-only.
    pub async fn begin_verification_connection(
        &self,
        credentials: LoginCredentials,
        existing: Option<Uuid>,
        label: &str,
        operation: &Operation,
    ) -> Result<VerificationAccountLogin> {
        let label = homeverb_domain::local_name(label)?;
        let expected = self.login_preflight(existing, operation).await?;
        let outcome = crate::begin_verification_login(
            self.transport.as_ref(),
            credentials,
            Region::Cn,
            operation,
        )
        .await?;
        self.finish_verification_connection(outcome, existing, label, expected, operation)
            .await
    }

    pub async fn continue_verification_connection(
        &self,
        pending: PendingVerificationConnection,
        answer: VerificationAnswer,
        operation: &Operation,
    ) -> Result<VerificationAccountLogin> {
        if pending.owner != self.login_owner || pending.expires_at() <= tokio::time::Instant::now()
        {
            return Err(unconfirmed());
        }
        if self.login_preflight(pending.existing, operation).await? != pending.expected {
            return Err(unconfirmed());
        }
        let outcome = match (pending.challenge, answer) {
            (Challenge::Captcha(challenge), VerificationAnswer::Captcha(answer)) => {
                challenge
                    .answer_with_verification(self.transport.as_ref(), answer, operation)
                    .await?
            }
            (Challenge::Verification(challenge), VerificationAnswer::Ticket { method, ticket }) => {
                VerificationLoginOutcome::Connected(
                    challenge
                        .answer(self.transport.as_ref(), method, ticket, operation)
                        .await?,
                )
            }
            _ => return Err(Error::invalid()),
        };
        self.finish_verification_connection(
            outcome,
            pending.existing,
            pending.label,
            pending.expected,
            operation,
        )
        .await
    }

    async fn finish_verification_connection(
        &self,
        outcome: VerificationLoginOutcome,
        existing: Option<Uuid>,
        label: String,
        expected: Vec<(Uuid, [u8; 32])>,
        operation: &Operation,
    ) -> Result<VerificationAccountLogin> {
        let challenge = match outcome {
            VerificationLoginOutcome::Connected(session) => {
                // Reuse the final locked snapshot comparison and identity-preserving save.
                let AccountLogin::Connected(report) = self
                    .finish_login(
                        LoginOutcome::Connected(session),
                        existing,
                        label,
                        expected,
                        operation,
                    )
                    .await?
                else {
                    unreachable!()
                };
                return Ok(VerificationAccountLogin::Connected(report));
            }
            VerificationLoginOutcome::Captcha(c) => Challenge::Captcha(c),
            VerificationLoginOutcome::Verification(c) => Challenge::Verification(c),
        };
        Ok(VerificationAccountLogin::Pending(
            PendingVerificationConnection {
                owner: self.login_owner,
                existing,
                label,
                expected,
                challenge,
            },
        ))
    }
}
