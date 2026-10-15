//! Explicit composition root for local hosts. Construction does not discover devices.

use homeverb_core::{Core, Grant, Policy, SystemClock};
use homeverb_domain::{Result, Transport};
use homeverb_mock::{DEVICE_ID, MockAdapter, Scenario};
use homeverb_storage::SqliteStore;
use std::{path::Path, sync::Arc};

mod native_approval;
pub use native_approval::{ExecuteApprovalRequest, PrepareApprovalRequest};
mod session;
pub use session::{HostRequest, Request, Session};
mod accounts;
pub use accounts::{AccountConnection, AccountSummary, ConnectRequest};
mod power_grants;
pub use power_grants::{MeijuPowerGrant, meiju_power_policy};
mod fan_grants;
pub use fan_grants::{MeijuFanGrant, meiju_fan_policy};
mod mode_grants;
pub use mode_grants::{MeijuModeGrant, meiju_mode_policy};
mod temperature_grants;
pub use temperature_grants::{MeijuTemperatureGrant, meiju_temperature_policy};

/// Trusted startup configuration only; ordinary requests cannot deserialize it.
#[derive(Default)]
pub struct MeijuControlGrants<'a> {
    pub power: &'a [MeijuPowerGrant],
    pub fan: &'a [MeijuFanGrant],
    pub mode: &'a [MeijuModeGrant],
    pub temperature: &'a [MeijuTemperatureGrant],
}

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
    open_meiju_with_control_grants(directory, &[], &[])
}

/// Trusted startup for direct Core hosts; validates all grants before opening a profile.
pub fn open_meiju_with_control_grants(
    directory: &Path,
    power: &[MeijuPowerGrant],
    fan: &[MeijuFanGrant],
) -> Result<Core> {
    open_meiju_with_scoped_grants(directory, power, fan, &[])
}

/// Trusted startup with independent power, fan and combined power-on/mode authority.
pub fn open_meiju_with_scoped_grants(
    directory: &Path,
    power: &[MeijuPowerGrant],
    fan: &[MeijuFanGrant],
    mode: &[MeijuModeGrant],
) -> Result<Core> {
    open_meiju_with_grants(
        directory,
        MeijuControlGrants {
            power,
            fan,
            mode,
            ..Default::default()
        },
    )
}

/// Compose exact independent control permissions before any profile or credential access.
pub fn open_meiju_with_grants(directory: &Path, grants: MeijuControlGrants<'_>) -> Result<Core> {
    meiju_parts(directory, grants).map(|(core, _)| core)
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
    open_meiju_session_with_control_grants(directory, grants, &[])
}

/// Trusted startup keeps exact power and vendor fan permissions independent.
pub fn open_meiju_session_with_control_grants(
    directory: &Path,
    power: &[MeijuPowerGrant],
    fan: &[MeijuFanGrant],
) -> Result<Session> {
    open_meiju_session_with_scoped_grants(directory, power, fan, &[])
}

/// Native sessions capture reviewed mode activation independently of other actions.
pub fn open_meiju_session_with_scoped_grants(
    directory: &Path,
    power: &[MeijuPowerGrant],
    fan: &[MeijuFanGrant],
    mode: &[MeijuModeGrant],
) -> Result<Session> {
    open_meiju_session_with_grants(
        directory,
        MeijuControlGrants {
            power,
            fan,
            mode,
            ..Default::default()
        },
    )
}

/// Capture trusted startup permissions for one foreground native session.
pub fn open_meiju_session_with_grants(
    directory: &Path,
    grants: MeijuControlGrants<'_>,
) -> Result<Session> {
    let (core, adapter) = meiju_parts(directory, grants)?;
    Ok(Session::with_accounts(core, adapter))
}

fn meiju_parts(
    directory: &Path,
    grants: MeijuControlGrants<'_>,
) -> Result<(Core, Arc<homeverb_midea::MeijuAdapter>)> {
    let mut policy = meiju_power_policy(grants.power)?;
    policy.grants.extend(meiju_fan_policy(grants.fan)?.grants);
    policy.grants.extend(meiju_mode_policy(grants.mode)?.grants);
    policy
        .grants
        .extend(meiju_temperature_policy(grants.temperature)?.grants);
    let store = Arc::new(SqliteStore::open(directory)?);
    let vault = Arc::new(open_accounts(directory)?);
    let clock = Arc::new(SystemClock);
    let mut adapter = homeverb_midea::MeijuAdapter::new(
        vault,
        Arc::new(homeverb_midea::MeijuHttps),
        clock.clone(),
    );
    if !grants.power.is_empty() {
        adapter = adapter.with_ac_power_control();
    }
    if !grants.fan.is_empty() {
        adapter = adapter.with_ac_fan_control();
    }
    if !grants.mode.is_empty() {
        adapter = adapter.with_ac_mode_control();
    }
    if !grants.temperature.is_empty() {
        adapter = adapter.with_ac_temperature_control();
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
    let mut grants: Vec<Grant> = if allow_writes {
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
    if allow_writes && scenario == Scenario::ApprovalRequired {
        grants.push(Grant {
            device_id: DEVICE_ID.into(),
            component_id: "main".into(),
            capability: "vendor.example.cycle".into(),
            action: "start".into(),
            transports: vec![Transport::Cloud],
            argument_schema: serde_json::json!({"const":{"program":"normal"}}),
        });
    }
    Ok(Core::new(store, vec![adapter], clock, Policy { grants }))
}
