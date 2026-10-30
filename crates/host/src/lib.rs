//! Explicit composition root for local hosts. Construction does not discover devices.

use homeverb_core::{Core, Grant, Policy, SystemClock};
use homeverb_domain::{Result, Transport};
use homeverb_mock::{DEVICE_ID, MockAdapter, Scenario};
use homeverb_storage::SqliteStore;
use std::{path::Path, sync::Arc};

mod native_approval;
pub use native_approval::{ExecuteApprovalRequest, PrepareApprovalRequest};
mod session;
pub use session::{
    ConfirmConnectionLinkRequest, ConfirmIdentityRecoveryRequest, ConfirmLanScanRequest,
    ContinueAccountLoginRequest, HostRequest, ModernAccountRequest, NativeAccountLogin,
    NativeModernQrLogin, NativeVerificationCodeRequest, NativeVerificationLogin,
    NativeVerificationPrompt, PendingMeijuLanScan, PrepareConnectionLinkRequest,
    PrepareLanScanRequest, Request, Session,
};
mod accounts;
pub use accounts::{
    AccountConnection, AccountProvider, AccountSummary, ConnectRequest, ModernAccountReport,
};
mod mijia_outlet_grants;
pub use mijia_outlet_grants::{MijiaOutletGrant, mijia_outlet_policy};
mod mijia_action_grants;
pub use mijia_action_grants::{MijiaActionGrant, mijia_action_policy};
mod mijia_setting_grants;
pub use mijia_setting_grants::{MijiaSettingGrant, mijia_setting_policy};
mod mijia_fan_grants;
pub use mijia_fan_grants::{MijiaFanGrant, mijia_fan_policy};
mod power_grants;
pub use power_grants::{MeijuPowerGrant, meiju_power_policy};
mod ac_feature_grants;
mod fan_grants;
pub use ac_feature_grants::{MeijuAcFeatureGrant, meiju_ac_feature_policy};
pub use fan_grants::{MeijuFanGrant, meiju_fan_policy};
mod mode_grants;
pub use mode_grants::{MeijuModeGrant, meiju_mode_policy};
mod dishwasher_grants;
pub use dishwasher_grants::{
    MeijuDishwasherCycleGrant, MeijuDishwasherGrant, meiju_dishwasher_cycle_policy,
    meiju_dishwasher_policy,
};
mod temperature_grants;
pub use temperature_grants::{MeijuTemperatureGrant, meiju_temperature_policy};

/// Trusted startup configuration only; ordinary requests cannot deserialize it.
#[derive(Default)]
pub struct MeijuControlGrants<'a> {
    pub ac_features: &'a [MeijuAcFeatureGrant],
    pub power: &'a [MeijuPowerGrant],
    pub fan: &'a [MeijuFanGrant],
    pub mode: &'a [MeijuModeGrant],
    pub temperature: &'a [MeijuTemperatureGrant],
    pub dishwasher: &'a [MeijuDishwasherGrant],
    pub dishwasher_cycle: &'a [MeijuDishwasherCycleGrant],
}

/// Trusted session composition; event reads do not grant any write authority.
#[derive(Default)]
pub struct MijiaSessionOptions<'a> {
    pub fan: &'a [MijiaFanGrant],
    pub settings: &'a [MijiaSettingGrant],
    pub outlets: &'a [MijiaOutletGrant],
    pub actions: &'a [MijiaActionGrant],
    pub event_history: bool,
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

/// Explicit native Mijia Core composition. Startup opens local metadata only;
/// trusted account connection/refresh is a separate foreground operation.
pub fn open_mijia_read_only(directory: &Path) -> Result<Core> {
    let store = Arc::new(SqliteStore::open(directory)?);
    let vault = Arc::new(open_accounts(directory)?);
    let clock = Arc::new(SystemClock);
    let adapter = Arc::new(homeverb_mijia::MijiaAdapter::new(
        vault,
        Arc::new(homeverb_mijia::MijiaHttps),
        clock.clone(),
    ));
    Ok(Core::new(store, vec![adapter], clock, Policy::default()))
}

/// Explicit CLI/MCP read-only composition for saved legacy or modern accounts.
/// Startup remains local; only explicit account refresh publishes event metadata.
pub fn open_mijia_read_only_with_events(directory: &Path) -> Result<Core> {
    let store = Arc::new(SqliteStore::open(directory)?);
    let vault = Arc::new(open_accounts(directory)?);
    let clock = Arc::new(SystemClock);
    let adapter = homeverb_mijia::MijiaAdapter::new(
        vault,
        Arc::new(homeverb_mijia::MijiaHttps),
        clock.clone(),
    )
    .with_modern_protocol(
        Arc::new(homeverb_mijia::MijiaHttps),
        Arc::new(homeverb_mijia::MijiaHttps),
    )
    .with_event_history();
    Ok(Core::new(
        store,
        vec![Arc::new(adapter)],
        clock,
        Policy::default(),
    ))
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
    platform_parts(directory, grants, false, MijiaSessionOptions::default())
        .map(|(core, meiju, _)| (core, meiju))
}

fn platform_parts(
    directory: &Path,
    grants: MeijuControlGrants<'_>,
    include_mijia: bool,
    mijia_options: MijiaSessionOptions<'_>,
) -> Result<(
    Core,
    Arc<homeverb_midea::MeijuAdapter>,
    Option<Arc<homeverb_mijia::MijiaAdapter>>,
)> {
    let MijiaSessionOptions {
        fan: mijia_grants,
        settings: setting_grants,
        outlets: outlet_grants,
        actions: action_grants,
        event_history,
    } = mijia_options;
    let action_policy = mijia_action_policy(action_grants)?;
    let outlet_policy = mijia_outlet_policy(outlet_grants)?;
    let reference_policy = mijia_setting_policy(setting_grants)?;
    let mut policy = meiju_power_policy(grants.power)?;
    policy.grants.extend(action_policy.grants.clone());
    policy.grants.extend(outlet_policy.grants.clone());
    policy.grants.extend(mijia_fan_policy(mijia_grants)?.grants);
    policy
        .grants
        .extend(mijia_setting_policy(setting_grants)?.grants);
    policy.grants.extend(meiju_fan_policy(grants.fan)?.grants);
    policy
        .grants
        .extend(meiju_ac_feature_policy(grants.ac_features)?.grants);
    policy.grants.extend(meiju_mode_policy(grants.mode)?.grants);
    policy
        .grants
        .extend(meiju_temperature_policy(grants.temperature)?.grants);
    policy
        .grants
        .extend(meiju_dishwasher_policy(grants.dishwasher)?.grants);
    policy
        .grants
        .extend(meiju_dishwasher_cycle_policy(grants.dishwasher_cycle)?.grants);
    let store = Arc::new(SqliteStore::open(directory)?);
    let vault = Arc::new(open_accounts(directory)?);
    let clock = Arc::new(SystemClock);
    let mut adapter = homeverb_midea::MeijuAdapter::new(
        vault.clone(),
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
    if !grants.ac_features.is_empty() {
        adapter = adapter
            .with_ac_feature_control()
            .with_ac_feature_devices(grants.ac_features.iter().map(|g| g.device_id.clone()));
    }
    if !grants.dishwasher.is_empty() {
        adapter = adapter.with_dishwasher_control();
    }
    if !grants.dishwasher_cycle.is_empty() {
        adapter = adapter.with_dishwasher_cycle_control();
    }
    let adapter = Arc::new(adapter);
    let mijia = include_mijia.then(|| {
        let adapter = homeverb_mijia::MijiaAdapter::new(
            vault,
            Arc::new(homeverb_mijia::MijiaHttps),
            clock.clone(),
        )
        .with_modern_protocol(
            Arc::new(homeverb_mijia::MijiaHttps),
            Arc::new(homeverb_mijia::MijiaHttps),
        )
        .with_modern_control_transport(Arc::new(homeverb_mijia::MijiaHttps));
        let adapter = if event_history {
            adapter.with_event_history()
        } else {
            adapter
        };
        let adapter = if mijia_grants.is_empty() {
            adapter
        } else {
            adapter.with_fan_controls()
        };
        let adapter = if setting_grants.is_empty() {
            adapter
        } else {
            adapter.with_reference_controls()
        };
        let adapter = if outlet_grants.is_empty() {
            adapter
        } else {
            adapter.with_outlet_controls()
        };
        let adapter = if action_grants.is_empty() {
            adapter
        } else {
            adapter.with_action_controls(action_policy)
        };
        Arc::new(adapter.with_control_policies(reference_policy, outlet_policy))
    });
    let mut adapters: Vec<Arc<dyn homeverb_core::Adapter>> = vec![adapter.clone()];
    if let Some(mijia) = &mijia {
        adapters.push(mijia.clone());
    }
    Ok((Core::new(store, adapters, clock, policy), adapter, mijia))
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

/// Foreground Mijia-only account host. Startup does not access secrets or the network.
pub fn open_mijia_session(directory: &Path) -> Result<Session> {
    open_mijia_session_with_grants(directory, &[])
}

pub fn open_mijia_session_with_grants(
    directory: &Path,
    grants: &[MijiaFanGrant],
) -> Result<Session> {
    open_mijia_session_with_settings(directory, grants, &[])
}

pub fn open_mijia_session_with_settings(
    directory: &Path,
    grants: &[MijiaFanGrant],
    settings: &[MijiaSettingGrant],
) -> Result<Session> {
    open_mijia_session_with_outlets(directory, grants, settings, &[])
}

pub fn open_mijia_session_with_outlets(
    directory: &Path,
    grants: &[MijiaFanGrant],
    settings: &[MijiaSettingGrant],
    outlets: &[MijiaOutletGrant],
) -> Result<Session> {
    open_mijia_session_with_actions(directory, grants, settings, outlets, &[])
}

pub fn open_mijia_session_with_actions(
    directory: &Path,
    grants: &[MijiaFanGrant],
    settings: &[MijiaSettingGrant],
    outlets: &[MijiaOutletGrant],
    actions: &[MijiaActionGrant],
) -> Result<Session> {
    open_mijia_session_with_options(
        directory,
        MijiaSessionOptions {
            fan: grants,
            settings,
            outlets,
            actions,
            ..Default::default()
        },
    )
}

pub fn open_mijia_session_with_options(
    directory: &Path,
    options: MijiaSessionOptions<'_>,
) -> Result<Session> {
    let MijiaSessionOptions {
        fan: grants,
        settings,
        outlets,
        actions,
        event_history,
    } = options;
    let action_policy = mijia_action_policy(actions)?;
    let outlet_policy = mijia_outlet_policy(outlets)?;
    let reference_policy = mijia_setting_policy(settings)?;
    let mut policy = mijia_fan_policy(grants)?;
    policy.grants.extend(action_policy.grants.clone());
    policy.grants.extend(outlet_policy.grants.clone());
    policy.grants.extend(mijia_setting_policy(settings)?.grants);
    let store = Arc::new(SqliteStore::open(directory)?);
    let vault = Arc::new(open_accounts(directory)?);
    let clock = Arc::new(SystemClock);
    let adapter = homeverb_mijia::MijiaAdapter::new(
        vault,
        Arc::new(homeverb_mijia::MijiaHttps),
        clock.clone(),
    )
    .with_modern_protocol(
        Arc::new(homeverb_mijia::MijiaHttps),
        Arc::new(homeverb_mijia::MijiaHttps),
    )
    .with_modern_control_transport(Arc::new(homeverb_mijia::MijiaHttps));
    let adapter = if event_history {
        adapter.with_event_history()
    } else {
        adapter
    };
    let adapter = if grants.is_empty() {
        adapter
    } else {
        adapter.with_fan_controls()
    };
    let adapter = if settings.is_empty() {
        adapter
    } else {
        adapter.with_reference_controls()
    };
    let adapter = if outlets.is_empty() {
        adapter
    } else {
        adapter.with_outlet_controls()
    };
    let adapter = if actions.is_empty() {
        adapter
    } else {
        adapter.with_action_controls(action_policy)
    };
    let adapter = Arc::new(adapter.with_control_policies(reference_policy, outlet_policy));
    let core = Core::new(store, vec![adapter.clone()], clock, policy);
    Ok(Session::with_platform_accounts(core, None, Some(adapter)))
}

/// Shared local profile with both reviewed cloud providers and independent Meiju grants.
pub fn open_platform_session(directory: &Path, grants: MeijuControlGrants<'_>) -> Result<Session> {
    open_platform_session_with_grants(directory, grants, &[])
}

pub fn open_platform_session_with_grants(
    directory: &Path,
    grants: MeijuControlGrants<'_>,
    mijia_grants: &[MijiaFanGrant],
) -> Result<Session> {
    open_platform_session_with_settings(directory, grants, mijia_grants, &[])
}

pub fn open_platform_session_with_settings(
    directory: &Path,
    grants: MeijuControlGrants<'_>,
    mijia_grants: &[MijiaFanGrant],
    settings: &[MijiaSettingGrant],
) -> Result<Session> {
    open_platform_session_with_outlets(directory, grants, mijia_grants, settings, &[])
}

pub fn open_platform_session_with_outlets(
    directory: &Path,
    grants: MeijuControlGrants<'_>,
    mijia_grants: &[MijiaFanGrant],
    settings: &[MijiaSettingGrant],
    outlets: &[MijiaOutletGrant],
) -> Result<Session> {
    open_platform_session_with_actions(directory, grants, mijia_grants, settings, outlets, &[])
}

pub fn open_platform_session_with_actions(
    directory: &Path,
    grants: MeijuControlGrants<'_>,
    mijia_grants: &[MijiaFanGrant],
    settings: &[MijiaSettingGrant],
    outlets: &[MijiaOutletGrant],
    actions: &[MijiaActionGrant],
) -> Result<Session> {
    open_platform_session_with_options(
        directory,
        grants,
        MijiaSessionOptions {
            fan: mijia_grants,
            settings,
            outlets,
            actions,
            ..Default::default()
        },
    )
}

pub fn open_platform_session_with_options(
    directory: &Path,
    grants: MeijuControlGrants<'_>,
    options: MijiaSessionOptions<'_>,
) -> Result<Session> {
    let (core, meiju, mijia) = platform_parts(directory, grants, true, options)?;
    Ok(Session::with_platform_accounts(core, Some(meiju), mijia))
}

pub use homeverb_mijia::{VerificationAnswer, VerificationMethod};
