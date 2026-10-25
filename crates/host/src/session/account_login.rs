//! Protected interactive account entry points, intentionally absent from Request.
use super::*;
use homeverb_mijia::AccountLogin;
use std::fmt;

/// Only a dedicated trusted UI bridge may display the image. No general serialization.
pub enum NativeAccountLogin {
    Connected(crate::AccountConnection),
    Captcha {
        challenge_id: u64,
        image: SecretBytes,
        media_type: &'static str,
        expires_in_ms: u64,
    },
}
impl fmt::Debug for NativeAccountLogin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeAccountLogin([REDACTED])")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContinueAccountLoginRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub timeout_ms: u64,
    pub challenge_id: u64,
}
fn unavailable() -> Error {
    Error::new(
        "MIJIA_AUTH_CHALLENGE_REQUIRED",
        "The login challenge is unavailable; start sign-in again.",
    )
}

impl Session {
    pub async fn begin_account_login(
        &self,
        input: crate::ConnectRequest,
        login: SecretBytes,
        password: SecretBytes,
    ) -> Result<NativeAccountLogin> {
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
        let outcome = adapter
            .begin_connection(credentials, input.account_id, &input.label, &operation)
            .await?;
        self.finish_account_login(outcome, generation, &operation)
            .await
    }

    pub async fn continue_account_login(
        &self,
        input: ContinueAccountLoginRequest,
        answer: SecretBytes,
    ) -> Result<NativeAccountLogin> {
        // Consume first: invalid metadata and operation admission cannot leave a replayable answer.
        let pending = self
            .state
            .lock()
            .map_err(|_| unavailable())?
            .account_logins
            .remove(&input.challenge_id)
            .ok_or_else(unavailable)?;
        let (operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        let adapter = self
            .mijia_accounts
            .as_ref()
            .ok_or_else(crate::accounts::unsupported)?;
        let generation = self.login_generation()?;
        let outcome = adapter
            .continue_connection(pending, answer, &operation)
            .await?;
        self.finish_account_login(outcome, generation, &operation)
            .await
    }

    pub fn discard_account_login(&self, id: u64) -> bool {
        self.state
            .lock()
            .ok()
            .is_some_and(|mut state| state.account_logins.remove(&id).is_some())
    }

    fn login_generation(&self) -> Result<u64> {
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        let now = tokio::time::Instant::now();
        state
            .account_logins
            .retain(|_, pending| pending.expires_at() > now);
        if state.account_logins.len() >= 4 {
            return Err(Error::new(
                "RESOURCE_BUSY",
                "Finish or dismiss pending sign-in first.",
            ));
        }
        Ok(state.approval_generation)
    }

    async fn finish_account_login(
        &self,
        outcome: AccountLogin,
        generation: u64,
        operation: &Operation,
    ) -> Result<NativeAccountLogin> {
        operation.run(async { Ok(()) }).await?;
        match outcome {
            AccountLogin::Connected(report) => Ok(NativeAccountLogin::Connected(
                crate::accounts::publish_mijia(&self.core, report, "succeeded", operation).await,
            )),
            AccountLogin::Captcha(pending) => {
                let mut state = self.state.lock().map_err(|_| unavailable())?;
                let now = tokio::time::Instant::now();
                if state.closed
                    || !state.foreground
                    || state.approval_generation != generation
                    || state.account_logins.len() >= 4
                    || pending.expires_at() <= now
                {
                    return Err(unavailable());
                }
                let id = NEXT_PREVIEW_ID.fetch_add(1, Ordering::Relaxed);
                let result = NativeAccountLogin::Captcha {
                    challenge_id: id,
                    image: SecretBytes::new(pending.image().to_vec()),
                    media_type: pending.media_type(),
                    expires_in_ms: pending.expires_at().duration_since(now).as_millis() as u64,
                };
                state.account_logins.insert(id, pending);
                Ok(result)
            }
        }
    }
}
