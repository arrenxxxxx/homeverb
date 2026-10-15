//! Explicit composition root for local hosts. Construction does not discover devices.

use homeverb_core::{Core, Grant, Policy, SystemClock};
use homeverb_domain::{Result, Transport};
use homeverb_mock::{DEVICE_ID, MockAdapter, Scenario};
use homeverb_storage::SqliteStore;
use std::{path::Path, sync::Arc};

mod session;
pub use session::{HostRequest, Request, Session};

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
