//! Foreground modern account work; protected QR data never enters ordinary dispatch.
use super::*;
use account_login::unavailable;
use homeverb_mijia::{MijiaAdapter, ModernAccountDiscovery};
use std::fmt;

/// Protected QR bytes for trusted local rendering only, never logs or general JSON.
/// ```compile_fail
/// fn export(value: &homeverb_host::NativeModernQrLogin) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct NativeModernQrLogin {
    pub challenge_id: u64,
    pub scan_payload: SecretBytes,
    pub expires_in_ms: u64,
}
impl fmt::Debug for NativeModernQrLogin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeModernQrLogin([REDACTED])")
    }
}

/// Closed metadata for explicit typed saved-session work; no protocol or origin selection.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModernAccountRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub timeout_ms: u64,
    pub account_id: uuid::Uuid,
}

impl Session {
    pub async fn begin_modern_account_login(
        &self,
        input: crate::ConnectRequest,
    ) -> Result<NativeModernQrLogin> {
        let (operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        if input.provider != Some(crate::AccountProvider::Mijia) {
            return Err(crate::accounts::unsupported());
        }
        let adapter = self.modern_account_adapter()?;
        let generation = self.login_generation()?;
        let pending = adapter
            .begin_modern_connection(input.account_id, &input.label, &operation)
            .await?;
        operation.run(async { Ok(()) }).await?;
        let mut state = self.state.lock().map_err(|_| unavailable())?;
        let now = tokio::time::Instant::now();
        if state.closed
            || !state.foreground
            || state.approval_generation != generation
            || state.pending_login_count() >= 4
            || pending.expires_at() <= now
        {
            return Err(unavailable());
        }
        let id = NEXT_PREVIEW_ID.fetch_add(1, Ordering::Relaxed);
        let result = NativeModernQrLogin {
            challenge_id: id,
            scan_payload: SecretBytes::new(pending.scan_payload().to_vec()),
            expires_in_ms: pending.expires_at().duration_since(now).as_millis() as u64,
        };
        state.modern_logins.insert(id, pending);
        Ok(result)
    }

    /// One explicit bounded poll/exchange. No ordinary serialization or device publication.
    pub async fn continue_modern_account_login(
        &self,
        input: ContinueAccountLoginRequest,
    ) -> Result<ModernAccountDiscovery> {
        let pending = self
            .state
            .lock()
            .map_err(|_| unavailable())?
            .modern_logins
            .remove(&input.challenge_id)
            .ok_or_else(unavailable)?;
        let (operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        let generation = self.modern_generation()?;
        let result = self
            .modern_account_adapter()?
            .continue_modern_connection(pending, &operation)
            .await?;
        self.complete_modern_operation(generation, &operation)
            .await?;
        Ok(result)
    }

    /// Publish protected device bindings from the original bounded QR directory read.
    pub async fn continue_modern_device_login(
        &self,
        input: ContinueAccountLoginRequest,
    ) -> Result<crate::AccountConnection> {
        let pending = self
            .state
            .lock()
            .map_err(|_| unavailable())?
            .modern_logins
            .remove(&input.challenge_id)
            .ok_or_else(unavailable)?;
        let (mut operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        operation.deadline = operation.deadline.min(pending.expires_at());
        let generation = self.modern_generation()?;
        let discovery = self
            .modern_account_adapter()?
            .continue_modern_device_connection(pending, &operation)
            .await?;
        self.complete_modern_operation(generation, &operation)
            .await?;
        let result =
            crate::accounts::publish_mijia(&self.core, discovery, "succeeded", &operation).await;
        self.complete_modern_operation(generation, &operation)
            .await?;
        Ok(result)
    }

    pub async fn refresh_modern_account_devices(
        &self,
        input: ModernAccountRequest,
    ) -> Result<crate::AccountConnection> {
        self.modern_devices(input, false).await
    }

    pub async fn recover_modern_account_devices(
        &self,
        input: ModernAccountRequest,
    ) -> Result<crate::AccountConnection> {
        self.modern_devices(input, true).await
    }

    async fn modern_devices(
        &self,
        input: ModernAccountRequest,
        recover: bool,
    ) -> Result<crate::AccountConnection> {
        let (operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        homeverb_core::valid_account_id(input.account_id)?;
        let generation = self.modern_generation()?;
        let adapter = self.modern_account_adapter()?;
        let discovery = if recover {
            adapter
                .recover_modern_devices(input.account_id, &operation)
                .await?
        } else {
            adapter
                .refresh_modern_devices(input.account_id, &operation)
                .await?
        };
        self.complete_modern_operation(generation, &operation)
            .await?;
        let result =
            crate::accounts::publish_mijia(&self.core, discovery, "not_attempted", &operation)
                .await;
        self.complete_modern_operation(generation, &operation)
            .await?;
        Ok(result)
    }

    pub fn discard_modern_account_login(&self, id: u64) -> bool {
        self.state
            .lock()
            .ok()
            .is_some_and(|mut state| state.modern_logins.remove(&id).is_some())
    }

    pub async fn refresh_modern_account(
        &self,
        input: ModernAccountRequest,
    ) -> Result<ModernAccountDiscovery> {
        self.modern_directory(input, false).await
    }

    pub async fn recover_modern_account(
        &self,
        input: ModernAccountRequest,
    ) -> Result<ModernAccountDiscovery> {
        self.modern_directory(input, true).await
    }

    async fn modern_directory(
        &self,
        input: ModernAccountRequest,
        recover: bool,
    ) -> Result<ModernAccountDiscovery> {
        let (operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        homeverb_core::valid_account_id(input.account_id)?;
        let generation = self.modern_generation()?;
        let adapter = self.modern_account_adapter()?;
        let result = if recover {
            adapter
                .recover_modern_directory(input.account_id, &operation)
                .await?
        } else {
            adapter
                .refresh_modern_directory(input.account_id, &operation)
                .await?
        };
        self.complete_modern_operation(generation, &operation)
            .await?;
        Ok(result)
    }

    /// Explicit authentication renewal only; returns no protected identity or credentials.
    pub async fn renew_modern_account(
        &self,
        input: ModernAccountRequest,
    ) -> Result<crate::AccountSummary> {
        let (operation, _registration) =
            self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
        homeverb_core::valid_account_id(input.account_id)?;
        let generation = self.modern_generation()?;
        let account = self
            .modern_account_adapter()?
            .renew_modern_account(input.account_id, &operation)
            .await?;
        self.complete_modern_operation(generation, &operation)
            .await?;
        Ok(account.into())
    }

    fn modern_account_adapter(&self) -> Result<&MijiaAdapter> {
        self.mijia_accounts
            .as_deref()
            .ok_or_else(crate::accounts::unsupported)
    }

    fn modern_generation(&self) -> Result<u64> {
        let state = self.state.lock().map_err(|_| unavailable())?;
        if state.closed || !state.foreground {
            return Err(unavailable());
        }
        Ok(state.approval_generation)
    }

    async fn complete_modern_operation(
        &self,
        generation: u64,
        operation: &Operation,
    ) -> Result<()> {
        operation.run(async { Ok(()) }).await?;
        if self.modern_generation()? != generation {
            return Err(unavailable());
        }
        Ok(())
    }
}
