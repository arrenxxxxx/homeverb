//! Independent protected action inspection and selected-protocol execution.
use super::*;
use crate::{ActionQuery, ControlOption};
use homeverb_domain::Risk;

impl MijiaAdapter {
    /// Local exact action choices, independent of grants and property setter options.
    /// No networking, saved mapping upgrade or authority mutation occurs.
    pub async fn inspect_action_options(
        &self,
        device: &Device,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<Vec<ControlOption>> {
        operation
            .run(async {
                let id = connection
                    .account_id
                    .as_deref()
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(conflict)?;
                let account =
                    LoadedAccount::load(self.vault.lease(id, operation).await?, false, operation)
                        .await?;
                let binding = account
                    .payload
                    .bindings
                    .iter()
                    .find(|b| b.device_id.to_string() == device.id)
                    .ok_or_else(conflict)?;
                validate_device(binding, id, device, connection)?;
                if !binding.listed {
                    return Err(unsupported());
                }
                if binding.modern.as_ref().is_some_and(|m| m.parent.is_some())
                    || (binding.is_modern() && self.modern_controls.is_none())
                {
                    return Ok(vec![]);
                }
                let options = binding
                    .mapping
                    .as_ref()
                    .map(|m| {
                        m.no_argument_actions()
                            .into_iter()
                            .map(|(_, o)| o)
                            .collect()
                    })
                    .unwrap_or_default();
                operation.run(async { Ok(options) }).await
            })
            .await
    }

    pub(super) async fn execute_no_argument_action(
        &self,
        account: &LoadedAccount,
        binding: &Binding,
        validated: &ValidatedCommand,
        action: &(ActionQuery, ControlOption),
        progress: &WriteProgress,
        operation: &Operation,
    ) -> Result<AdapterOutcome> {
        let command = validated.command();
        let (query, option) = action;
        // Re-derived protected risk is independent of mutable public metadata.
        match option.definition.risk {
            Risk::Low => {}
            Risk::Elevated if validated.has_user_approval() => {}
            _ => {
                return Err(Error::new(
                    "POLICY_DENIED",
                    "This power toggle requires trusted user approval.",
                ));
            }
        }
        let policy = self.action_policy.as_ref().ok_or_else(unsupported)?;
        if binding.modern.as_ref().is_some_and(|m| m.parent.is_some()) {
            return Err(unsupported());
        }
        option.definition.validate_arguments(&command.arguments)?;
        // The private approval marker was checked above. This check admits only
        // the exact action-family scope and does not substitute for user consent.
        let mut scope = option.definition.clone();
        scope.risk = Risk::Low;
        if !policy
            .authorize(command, &scope)?
            .contains(&Transport::Cloud)
        {
            return Err(Error::new(
                "POLICY_DENIED",
                "The action family has no exact grant.",
            ));
        }
        let mapping = binding.mapping.as_ref().ok_or_else(unsupported)?;
        // Protected validation already matched the complete saved source. Pinned source
        // evidence may fill an omitted inventory specification, never a conflict.
        let mut device = binding.device();
        if device
            .spec_type
            .as_ref()
            .is_some_and(|s| s != &mapping.spec_type)
        {
            return Err(conflict());
        }
        device.spec_type = Some(mapping.spec_type.clone());
        if binding.is_modern() {
            let session = account.payload.modern_session()?;
            let mut entry = session.protected_entry(&device)?;
            entry.parent = binding.modern.as_ref().ok_or_else(conflict)?.parent.clone();
            entry.kind = Some(crate::modern::reads::DirectoryKind(
                binding.modern.as_ref().ok_or_else(conflict)?.kind,
            ));
            session
                .invoke_action(
                    self.modern_controls.as_deref().ok_or_else(unsupported)?,
                    &entry,
                    query,
                    progress,
                    operation,
                )
                .await
        } else {
            account
                .payload
                .legacy_session()?
                .invoke_action(self.transport.as_ref(), &device, query, progress, operation)
                .await
        }
    }
}
