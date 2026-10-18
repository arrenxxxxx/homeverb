//! Explicit owner-confirmed migration of an intact legacy account binding.
use super::*;
use crate::auth_trace::{Event, emit};
use std::time::Duration;
use tokio::time::Instant;

/// A bounded, single-use private preview. Only a trusted UI may confirm it.
/// ```compile_fail
/// fn export(value: &homeverb_midea::PendingIdentityRecovery) { let _ = serde_json::to_string(value); }
/// ```
pub struct PendingIdentityRecovery {
    record: AccountRecord,
    original: SecretBytes,
    session: MeijuSession,
    inventory: Inventory,
    deadline: Instant,
}
impl fmt::Debug for PendingIdentityRecovery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PendingIdentityRecovery([REDACTED])")
    }
}
impl PendingIdentityRecovery {
    pub fn account(&self) -> &AccountRecord {
        &self.record
    }
    pub fn device_count(&self) -> usize {
        self.inventory.appliances.len()
    }
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }
}
fn refused() -> Error {
    Error::new(
        "ACCOUNT_IDENTITY_CONFLICT",
        "Legacy recovery evidence does not match the saved account scope.",
    )
}

pub(crate) async fn prepare(
    vault: &AccountVault,
    transport: &dyn CloudTransport,
    credentials: LoginCredentials,
    id: Uuid,
    operation: &Operation,
) -> Result<PendingIdentityRecovery> {
    emit(Event::RecoveryRequested);
    let _registration = vault.lock_registration(operation).await?;
    let account = MeijuAccount {
        lease: vault.lease(id, operation).await?,
    };
    if account.record().provider != PROVIDER || account.record().region != "CN" {
        return Err(refused());
    }
    let original = account.reauthorization_material(operation).await?;
    let payload = Payload::decode(&original, account.record())?;
    let previous = payload.cloud_session()?;
    emit(Event::RecoveryBindingValidated);
    if previous.is_canonical() {
        emit(Event::RecoveryRequiresLegacy);
        return Err(refused());
    }
    if payload.bindings.is_empty() {
        emit(Event::RecoveryEmptyBindings);
        return Err(refused());
    }
    if payload.bindings.iter().any(|b| b.listed != Some(true)) {
        emit(Event::RecoveryMembershipUnconfirmed);
        return Err(refused());
    }
    emit(Event::RecoveryLoginStarted);
    let session = login(transport, &credentials, operation).await?;
    drop(credentials);
    emit(Event::LoginSucceeded);
    if !session.is_canonical() {
        emit(Event::RecoveryMissingCanonicalIdentity);
        return Err(refused());
    }
    if !session.same_login_input(&previous) {
        emit(Event::RecoveryLoginInputMismatch);
        return Err(refused());
    }
    emit(Event::RecoveryInventoryStarted);
    let reference = account.record().credential_ref;
    let inventory =
        retrieve_inventory(&session.reads(transport, reference), reference, operation).await?;
    if !inventory.is_complete() {
        emit(Event::RecoveryInventoryIncomplete);
        return Err(refused());
    }
    if inventory.appliances.len() != payload.bindings.len() {
        emit(Event::RecoveryInventoryCountMismatch);
        return Err(refused());
    }
    for old in &payload.bindings {
        let candidates: Vec<_> = inventory
            .appliances
            .iter()
            .filter(|a| a.id.expose() == old.appliance_id)
            .collect();
        if candidates.len() != 1 {
            emit(Event::RecoveryInventoryIdentityMismatch);
            return Err(refused());
        }
        let fresh = candidates[0];
        if family_code(fresh.family) != old.family || fresh.model != old.model {
            emit(Event::RecoveryInventoryModelMismatch);
            return Err(refused());
        }
        ProductProfile::from_stored(&old.product_profile)
            .merge(&fresh.product_profile)
            .inspect_err(|_| emit(Event::RecoveryProfileConflict))?;
    }
    operation
        .run(async {
            emit(Event::RecoveryReady);
            Ok(PendingIdentityRecovery {
                record: account.record().clone(),
                original,
                session,
                inventory,
                deadline: Instant::now() + Duration::from_secs(60),
            })
        })
        .await
}

/// Call only after actual trusted user consent; inventory matching alone is not identity proof.
pub(crate) async fn confirm(
    vault: &AccountVault,
    preview: PendingIdentityRecovery,
    operation: &Operation,
) -> Result<ConnectReport> {
    emit(Event::RecoveryConfirmRequested);
    if preview.is_expired() {
        emit(Event::RecoveryExpired);
        return Err(refused());
    }
    let _registration = vault.lock_registration(operation).await?;
    ensure_unique_subject(vault, preview.record.id, &preview.session, operation).await?;
    confirm_selected(vault, preview, operation).await
}

pub(super) async fn ensure_unique_subject(
    vault: &AccountVault,
    selected: Uuid,
    session: &MeijuSession,
    operation: &Operation,
) -> Result<()> {
    // Never merge two existing local accounts, including unconfirmed protected records.
    for record in vault.list(operation).await? {
        if record.id == selected || record.provider != PROVIDER || record.region != "CN" {
            continue;
        }
        let account = MeijuAccount {
            lease: vault.lease(record.id, operation).await?,
        };
        let original = account.reauthorization_material(operation).await?;
        let payload = Payload::decode(&original, account.record())?;
        if session.matches_saved_subject(&payload.subject) {
            emit(Event::RecoveryDuplicateSubject);
            return Err(refused());
        }
    }
    Ok(())
}

async fn confirm_selected(
    vault: &AccountVault,
    preview: PendingIdentityRecovery,
    operation: &Operation,
) -> Result<ConnectReport> {
    let mut account = MeijuAccount {
        lease: vault.lease(preview.record.id, operation).await?,
    };
    if preview.is_expired() {
        emit(Event::RecoveryExpired);
        return Err(refused());
    }
    if account.record() != &preview.record {
        emit(Event::RecoverySavedMetadataChanged);
        return Err(refused());
    }
    let current = account.reauthorization_material(operation).await?;
    if current.expose() != preview.original.expose() {
        emit(Event::RecoverySavedBytesChanged);
        return Err(unconfirmed());
    }
    let mut payload = Payload::decode(&current, account.record())?;
    payload.subject.zeroize();
    payload.subject = preview.session.subject().expose().to_vec();
    payload.session.zeroize();
    payload.session = preview.session.encode()?.expose().to_vec();
    operation
        .run(async {
            if preview.is_expired() {
                emit(Event::RecoveryExpired);
                Err(refused())
            } else {
                Ok(())
            }
        })
        .await?;
    account.save(&payload, operation).await?;
    emit(Event::RecoverySessionSaved);
    let retrieval = async {
        let bindings = account
            .bind_inventory(&preview.inventory, operation)
            .await?;
        account.lease.record_complete_inventory(operation).await?;
        Ok(BoundInventory {
            inventory: preview.inventory,
            bindings,
        })
    }
    .await;
    emit(if retrieval.is_ok() {
        Event::RecoveryComplete
    } else {
        Event::InventoryFailed
    });
    Ok(ConnectReport {
        account: account.record().clone(),
        retrieval,
    })
}
