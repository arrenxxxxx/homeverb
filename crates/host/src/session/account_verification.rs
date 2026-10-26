//! Opt-in typed verification registry; absent from C and ordinary JSON dispatch.
use super::*;
use account_login::unavailable;
use homeverb_mijia::{
    AccountLoginChallenge, VerificationAccountLogin, VerificationAnswer, VerificationMethod,
};
use std::fmt;

/// Protected presentation material for a future dedicated trusted browser bridge.
pub enum NativeVerificationPrompt {
    Captcha {
        image: SecretBytes,
        media_type: &'static str,
    },
    Verification {
        provider_page: SecretBytes,
        methods: Vec<VerificationMethod>,
    },
}
impl fmt::Debug for NativeVerificationPrompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeVerificationPrompt([REDACTED])")
    }
}

/// No serialization: provider URLs and tickets must not enter ordinary requests/results.
/// ```compile_fail
/// fn export(value: &homeverb_host::NativeVerificationLogin) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub enum NativeVerificationLogin {
    Connected(crate::AccountConnection),
    Pending {
        challenge_id: u64,
        prompt: NativeVerificationPrompt,
        expires_in_ms: u64,
    },
}
impl fmt::Debug for NativeVerificationLogin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeVerificationLogin([REDACTED])")
    }
}

/// A code-zero request acknowledgement, never proof of message delivery.
#[derive(Debug)]
pub struct NativeVerificationCodeRequest {
    pub login: NativeVerificationLogin,
    pub requests: u8,
    pub cooldown_ms: u64,
}

impl Session {
    /// Explicit trusted UI request; no ordinary JSON equivalent and no automatic resend.
    pub async fn request_account_verification_code(
        &self,
        input: ContinueAccountLoginRequest,
        method: VerificationMethod,
    ) -> Result<NativeVerificationCodeRequest> {
        let pending = self
            .state
            .lock()
            .map_err(|_| unavailable())?
            .verification_logins
            .remove(&input.challenge_id)
            .ok_or_else(unavailable)?;
        let (operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        let adapter = self
            .mijia_accounts
            .as_ref()
            .ok_or_else(crate::accounts::unsupported)?;
        let generation = self.login_generation()?;
        let pending = adapter
            .request_verification_code(pending, method, &operation)
            .await?;
        let requests = pending.code_requests();
        let cooldown_ms = pending.code_request_cooldown().as_millis() as u64;
        let login = self
            .finish_account_verification(
                VerificationAccountLogin::Pending(pending),
                generation,
                &operation,
            )
            .await?;
        Ok(NativeVerificationCodeRequest {
            login,
            requests,
            cooldown_ms,
        })
    }

    pub async fn begin_account_verification(
        &self,
        input: crate::ConnectRequest,
        login: SecretBytes,
        password: SecretBytes,
    ) -> Result<NativeVerificationLogin> {
        let (operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        if input.provider != Some(crate::AccountProvider::Mijia) {
            return Err(crate::accounts::unsupported());
        }
        let adapter = self
            .mijia_accounts
            .as_ref()
            .ok_or_else(crate::accounts::unsupported)?;
        let generation = self.login_generation()?;
        let credentials = homeverb_mijia::LoginCredentials::new(login, password)?;
        let result = adapter
            .begin_verification_connection(credentials, input.account_id, &input.label, &operation)
            .await?;
        self.finish_account_verification(result, generation, &operation)
            .await
    }

    pub async fn continue_account_verification(
        &self,
        input: ContinueAccountLoginRequest,
        answer: VerificationAnswer,
    ) -> Result<NativeVerificationLogin> {
        let pending = self
            .state
            .lock()
            .map_err(|_| unavailable())?
            .verification_logins
            .remove(&input.challenge_id)
            .ok_or_else(unavailable)?;
        let (operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        let adapter = self
            .mijia_accounts
            .as_ref()
            .ok_or_else(crate::accounts::unsupported)?;
        let generation = self.login_generation()?;
        let result = adapter
            .continue_verification_connection(pending, answer, &operation)
            .await?;
        self.finish_account_verification(result, generation, &operation)
            .await
    }

    pub fn discard_account_verification(&self, id: u64) -> bool {
        self.state
            .lock()
            .ok()
            .is_some_and(|mut state| state.verification_logins.remove(&id).is_some())
    }

    async fn finish_account_verification(
        &self,
        outcome: VerificationAccountLogin,
        generation: u64,
        operation: &Operation,
    ) -> Result<NativeVerificationLogin> {
        operation.run(async { Ok(()) }).await?;
        match outcome {
            VerificationAccountLogin::Connected(report) => Ok(NativeVerificationLogin::Connected(
                crate::accounts::publish_mijia(&self.core, report, "succeeded", operation).await,
            )),
            VerificationAccountLogin::Pending(pending) => {
                let mut state = self.state.lock().map_err(|_| unavailable())?;
                let now = tokio::time::Instant::now();
                if state.closed
                    || !state.foreground
                    || state.approval_generation != generation
                    || state.account_logins.len() + state.verification_logins.len() >= 4
                    || pending.expires_at() <= now
                {
                    return Err(unavailable());
                }
                let prompt = match pending.challenge() {
                    AccountLoginChallenge::Captcha { image, media_type } => {
                        NativeVerificationPrompt::Captcha {
                            image: SecretBytes::new(image.to_vec()),
                            media_type,
                        }
                    }
                    AccountLoginChallenge::Verification {
                        provider_page,
                        methods,
                    } => NativeVerificationPrompt::Verification {
                        provider_page: SecretBytes::new(provider_page.as_bytes().to_vec()),
                        methods: methods.to_vec(),
                    },
                };
                let id = NEXT_PREVIEW_ID.fetch_add(1, Ordering::Relaxed);
                let result = NativeVerificationLogin::Pending {
                    challenge_id: id,
                    prompt,
                    expires_in_ms: pending.expires_at().duration_since(now).as_millis() as u64,
                };
                state.verification_logins.insert(id, pending);
                Ok(result)
            }
        }
    }
}
