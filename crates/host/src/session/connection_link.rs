//! Trusted native metadata and session-owned connection-link previews.

use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareConnectionLinkRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub timeout_ms: u64,
    pub device_id: String,
    pub anchor_id: String,
    pub connection: Connection,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmConnectionLinkRequest {
    pub schema_version: String,
    pub operation_id: String,
    pub timeout_ms: u64,
    pub link_id: u64,
}

fn invalid() -> Error {
    Error::new(
        "CONNECTION_LINK_INVALID",
        "The connection preview is unavailable for this session; review it again.",
    )
}

impl Session {
    /// Trusted controller only. Public connection metadata never replaces adapter evidence.
    pub async fn prepare_native_connection_link(
        &self,
        input: PrepareConnectionLinkRequest,
    ) -> Value {
        let result = async {
            if input.schema_version != SCHEMA_VERSION {
                return Err(Error::invalid());
            }
            let generation = self
                .state
                .lock()
                .map_err(|_| invalid())?
                .approval_generation;
            let preview = self
                .prepare_connection_link(
                    input.operation_id,
                    &input.device_id,
                    &input.anchor_id,
                    input.connection,
                    input.timeout_ms,
                )
                .await?;
            let mut state = self.state.lock().map_err(|_| invalid())?;
            if state.closed || !state.foreground || state.approval_generation != generation {
                return Err(invalid());
            }
            state
                .connection_links
                .retain(|_, value| !value.is_expired());
            if state.connection_links.len() >= 8 {
                return Err(Error::new(
                    "RESOURCE_BUSY",
                    "Dismiss a pending connection link before preparing another.",
                ));
            }
            let id = NEXT_PREVIEW_ID.fetch_add(1, Ordering::Relaxed);
            let grants: Vec<_> = preview
                .transport_grants()
                .iter()
                .map(|grant| {
                    json!({
                        "device_id":grant.device_id, "component_id":grant.component_id,
                        "capability":grant.capability, "action":grant.action,
                        "transports":grant.transports, "argument_schema":grant.argument_schema
                    })
                })
                .collect();
            let value = json!({
                "link_id":id, "descriptor":preview.descriptor(),
                "anchor":preview.anchor(), "connection":preview.connection(),
                "transport_grants":grants,
                "expires_at":preview.expires_at()
            });
            state.connection_links.insert(id, preview);
            Ok(value)
        }
        .await;
        json!(Query::from_result(result))
    }

    /// Consume only after actual user consent; failures also consume a matching preview.
    pub async fn confirm_native_connection_link(
        &self,
        input: ConfirmConnectionLinkRequest,
    ) -> Value {
        let result = async {
            let preview = self
                .state
                .lock()
                .map_err(|_| invalid())?
                .connection_links
                .remove(&input.link_id)
                .ok_or_else(invalid)?;
            if input.schema_version != SCHEMA_VERSION {
                return Err(Error::invalid());
            }
            self.confirm_connection_link(input.operation_id, preview, input.timeout_ms)
                .await
        }
        .await;
        json!(Query::from_result(result))
    }

    /// Dismiss pending consent only. A committed connection is never removed by dismissal.
    pub fn discard_native_connection_link(&self, link_id: u64) -> bool {
        self.state
            .lock()
            .ok()
            .is_some_and(|mut state| state.connection_links.remove(&link_id).is_some())
    }
}
