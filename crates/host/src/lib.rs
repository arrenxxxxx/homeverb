//! Explicit composition root for local hosts. Construction does not discover devices.

use homeverb_core::{Core, Grant, Policy, SystemClock};
use homeverb_domain::{Result, Transport};
use homeverb_mock::{DEVICE_ID, MockAdapter, Scenario};
use homeverb_storage::SqliteStore;
use std::{path::Path, sync::Arc};

mod session;
pub use session::{HostRequest, Request, Session};
mod accounts;
pub use accounts::{AccountConnection, AccountSummary, ConnectRequest};
mod power_grants;
pub use power_grants::{MeijuPowerGrant, meiju_power_policy};

/// Compose a native account vault without reading credentials or starting authentication.
pub fn open_accounts(directory: &Path) -> Result<homeverb_core::AccountVault> {
    use homeverb_core::AccountStore;
    let store = Arc::new(SqliteStore::open(directory)?);
    let secrets = Arc::new(homeverb_secrets::SystemSecretStore::new(
        store.profile_namespace()?,
    )?);
    Ok(homeverb_core::AccountVault::new(store, secrets))
}

/// Compose the real read-only adapter without loading credentials or contacting the provider.
/// Account connection is a separate trusted native operation; there are no write grants.
pub fn open_meiju_read_only(directory: &Path) -> Result<Core> {
    meiju_parts(directory, &[]).map(|(core, _)| core)
}

/// Compose foreground account operations and the read-only Core with one adapter instance.
pub fn open_meiju_session(directory: &Path) -> Result<Session> {
    open_meiju_session_with_power_grants(directory, &[])
}

/// Explicit trusted startup admission; no authentication, discovery or control at construction.
pub fn open_meiju_session_with_power_grants(
    directory: &Path,
    grants: &[MeijuPowerGrant],
) -> Result<Session> {
    let (core, adapter) = meiju_parts(directory, grants)?;
    Ok(Session::with_accounts(core, adapter))
}

fn meiju_parts(
    directory: &Path,
    grants: &[MeijuPowerGrant],
) -> Result<(Core, Arc<homeverb_midea::MeijuAdapter>)> {
    let policy = meiju_power_policy(grants)?;
    let store = Arc::new(SqliteStore::open(directory)?);
    let vault = Arc::new(open_accounts(directory)?);
    let clock = Arc::new(SystemClock);
    let mut adapter = homeverb_midea::MeijuAdapter::new(
        vault,
        Arc::new(homeverb_midea::MeijuHttps),
        clock.clone(),
    );
    if !policy.grants.is_empty() {
        adapter = adapter.with_ac_power_control();
    }
    let adapter = Arc::new(adapter);
    Ok((
        Core::new(store, vec![adapter.clone()], clock, policy),
        adapter,
    ))
}

pub fn open_mock(directory: &Path, scenario: Scenario, allow_writes: bool) -> Result<Core> {
    let store = Arc::new(SqliteStore::open(directory)?);
    let clock = Arc::new(SystemClock);
    let adapter = Arc::new(MockAdapter::open(
        &directory.join("mock-appliance.sqlite3"),
        clock.clone(),
        scenario,
    )?);
    let grants = if allow_writes {
        ["power", "climate"]
            .into_iter()
            .map(|capability| Grant {
                device_id: DEVICE_ID.into(),
                component_id: "main".into(),
                capability: capability.into(),
                action: "set".into(),
                transports: vec![Transport::Lan, Transport::Cloud],
                argument_schema: serde_json::json!({"type": "object"}),
            })
            .collect()
    } else {
        vec![]
    };
    Ok(Core::new(store, vec![adapter], clock, Policy { grants }))
}
