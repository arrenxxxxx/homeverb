//! Trusted legacy-account recovery, outside the ordinary JSON dispatcher.
use super::*;

enum Stage {
    ConfirmReceived,
    PreviewUnavailable,
    PreviewTaken,
    HostUnavailable,
    PreviewExpired,
    HostSealed,
}
impl Stage {
    fn emit(self) {
        if option_env!("HOMEVERB_AUTH_TRACE") != Some("1") {
            return;
        }
        let marker = match self {
            Self::ConfirmReceived => c"recovery_host_confirm_received",
            Self::PreviewUnavailable => c"recovery_host_preview_unavailable",
            Self::PreviewTaken => c"recovery_host_preview_taken",
            Self::HostUnavailable => c"recovery_host_unavailable",
            Self::PreviewExpired => c"recovery_host_preview_expired",
            Self::HostSealed => c"recovery_host_sealed",
        };
        #[cfg(target_os = "android")]
        {
            #[link(name = "log")]
            unsafe extern "C" {
                fn __android_log_write(
                    priority: std::ffi::c_int,
                    tag: *const std::ffi::c_char,
                    text: *const std::ffi::c_char,
                ) -> std::ffi::c_int;
            }
            // Only fixed enum markers; never format request or account data.
            unsafe { __android_log_write(4, c"HomeVerbAuthTrace".as_ptr(), marker.as_ptr()) };
        }
        #[cfg(not(target_os = "android"))]
        let _ = marker;
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmIdentityRecoveryRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub timeout_ms: u64,
    pub recovery_id: u64,
}
fn invalid() -> Error {
    Error::new(
        "ACCOUNT_IDENTITY_CONFLICT",
        "The account recovery preview is unavailable; review it again.",
    )
}
impl Session {
    pub async fn prepare_native_identity_recovery(
        &self,
        input: crate::ConnectRequest,
        login: SecretBytes,
        password: SecretBytes,
    ) -> Value {
        let result = async {
            let (operation, _registration) =
                self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
            let account_id = input.account_id.ok_or_else(Error::invalid)?;
            let generation = {
                let mut state = self.state.lock().map_err(|_| invalid())?;
                state.identity_recoveries.retain(|_, p| !p.is_expired());
                if state.identity_recoveries.len() >= 8 {
                    return Err(Error::new(
                        "RESOURCE_BUSY",
                        "Dismiss pending account recovery first.",
                    ));
                }
                state.approval_generation
            };
            let adapter = self
                .accounts
                .as_ref()
                .ok_or_else(crate::accounts::unsupported)?;
            let credentials = homeverb_midea::LoginCredentials::new(login, password)?;
            let preview = adapter
                .prepare_identity_recovery(credentials, account_id, &operation)
                .await?;
            operation.run(async { Ok(()) }).await?;
            let mut state = self.state.lock().map_err(|_| invalid())?;
            if state.closed
                || !state.foreground
                || state.approval_generation != generation
                || state.identity_recoveries.len() >= 8
            {
                return Err(invalid());
            }
            let id = NEXT_PREVIEW_ID.fetch_add(1, Ordering::Relaxed);
            let value = json!({"recovery_id":id,
                "account":crate::AccountSummary::from(preview.account().clone()),
                "device_count":preview.device_count(), "expires_in_ms":60000,
                "requires_same_account_confirmation":true});
            state.identity_recoveries.insert(id, preview);
            Ok(value)
        }
        .await;
        json!(Query::from_result(result))
    }

    /// Actual UI consent is required. The confirming host is sealed before any persistence.
    pub async fn confirm_native_identity_recovery(
        &self,
        input: ConfirmIdentityRecoveryRequest,
    ) -> Value {
        Stage::ConfirmReceived.emit();
        let result = async {
            let preview = self
                .state
                .lock()
                .map_err(|_| invalid())?
                .identity_recoveries
                .remove(&input.recovery_id)
                .ok_or_else(|| {
                    Stage::PreviewUnavailable.emit();
                    invalid()
                })?;
            Stage::PreviewTaken.emit();
            let (operation, _registration) =
                self.register(&input.schema_version, input.operation_id, input.timeout_ms)?;
            let adapter = self
                .accounts
                .as_ref()
                .ok_or_else(crate::accounts::unsupported)?;
            {
                let mut state = self.state.lock().map_err(|_| invalid())?;
                if state.operations.len() != 1 {
                    return Err(Error::new(
                        "RESOURCE_BUSY",
                        "Finish other operations before account recovery.",
                    ));
                }
                if state.closed || !state.foreground {
                    Stage::HostUnavailable.emit();
                    return Err(invalid());
                }
                if preview.is_expired() {
                    Stage::PreviewExpired.emit();
                    return Err(invalid());
                }
                state.closed = true;
                state.approval_generation += 1;
                state.approvals.clear();
                state.lan_scans.clear();
                state.connection_links.clear();
                state.identity_recoveries.clear();
            }
            Stage::HostSealed.emit();
            self.core.invalidate_user_approvals();
            let result = adapter.confirm_identity_recovery(preview, &operation).await;
            let result = match result {
                Ok(report) => {
                    Ok(crate::accounts::publish(&self.core, report, "succeeded", &operation).await)
                }
                Err(error) => Err(error),
            };
            self.close();
            result
        }
        .await;
        json!(Query::from_result(result))
    }
    pub fn discard_native_identity_recovery(&self, id: u64) -> bool {
        self.state
            .lock()
            .ok()
            .is_some_and(|mut state| state.identity_recoveries.remove(&id).is_some())
    }
}
