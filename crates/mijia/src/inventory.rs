//! Private cloud inventory and exact MIoT property correlation, without invented state.

use crate::{CloudEndpoint, CloudTransport, MijiaSession, PrivateString, decode, invalid};
use homeverb_core::{Operation, SecretBytes};
use homeverb_domain::{Error, Result};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashSet, fmt};

pub struct CloudDevice {
    pub(crate) did: PrivateString,
    pub name: String,
    pub model: String,
    pub spec_type: Option<String>,
    pub online_hint: Option<bool>,
}
impl fmt::Debug for CloudDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CloudDevice([REDACTED])")
    }
}
pub struct Inventory {
    pub devices: Vec<CloudDevice>,
    /// The legacy device-list endpoint does not prove complete home/shared inventory.
    pub complete: bool,
}
impl fmt::Debug for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Inventory([REDACTED])")
    }
}
#[derive(Deserialize)]
struct Envelope<T> {
    result: T,
}
#[derive(Deserialize)]
struct DeviceList {
    list: Vec<DeviceWire>,
    #[serde(default)]
    has_more: Option<bool>,
}
#[derive(Deserialize)]
struct DeviceWire {
    did: PrivateString,
    name: String,
    model: String,
    #[serde(rename = "spec_type")]
    spec: Option<String>,
    #[serde(rename = "isOnline")]
    online: Option<bool>,
}

impl MijiaSession {
    pub async fn inventory(
        &self,
        transport: &dyn CloudTransport,
        operation: &Operation,
    ) -> Result<Inventory> {
        let data = SecretBytes::new(br#"{"getVirtualModel":true,"getHuamiDevices":1,"get_split_device":false,"support_smart_home":true}"#.to_vec());
        let response = self
            .request(transport, CloudEndpoint::Devices, &data, operation)
            .await?;
        decode_inventory(response.expose())
    }

    pub async fn properties(
        &self,
        transport: &dyn CloudTransport,
        device: &CloudDevice,
        queries: &[PropertyQuery],
        operation: &Operation,
    ) -> Result<Vec<PropertyReading>> {
        operation.run(async {
            if queries.is_empty() || queries.len() > 64 || queries.iter().any(|q| q.siid == 0 || q.piid == 0)
                || queries.iter().collect::<HashSet<_>>().len() != queries.len() { return Err(Error::invalid()); }
            let params: Vec<_> = queries.iter().map(|q| serde_json::json!({"did":device.did.as_str(),"siid":q.siid,"piid":q.piid})).collect();
            let data = SecretBytes::new(serde_json::to_vec(&serde_json::json!({"params":params})).map_err(|_| invalid())?);
            let response = self.request(transport, CloudEndpoint::Properties, &data, operation).await?;
            decode_properties(response.expose(), device.did.as_str(), queries)
        }).await
    }
}

fn decode_inventory(bytes: &[u8]) -> Result<Inventory> {
    let result: Envelope<DeviceList> = decode(bytes)?;
    if result.result.list.len() > 4096 {
        return Err(invalid());
    }
    if result.result.has_more == Some(true) {
        return Err(Error::new(
            "MIJIA_INVENTORY_PARTIAL",
            "The device list is truncated and requires additional inventory support.",
        ));
    }
    let mut ids = HashSet::new();
    let mut devices = vec![];
    for wire in result.result.list {
        if !crate::bounded_text(wire.did.as_str(), 128)
            || !ids.insert(wire.did.as_str().to_owned())
            || !crate::bounded_text(&wire.name, 256)
            || !crate::bounded_text(&wire.model, 256)
            || wire
                .spec
                .as_ref()
                .is_some_and(|s| !crate::bounded_text(s, 512))
        {
            return Err(invalid());
        }
        devices.push(CloudDevice {
            did: wire.did,
            name: wire.name,
            model: wire.model,
            spec_type: wire.spec,
            online_hint: wire.online,
        });
    }
    Ok(Inventory {
        devices,
        complete: false,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertyQuery {
    pub siid: u32,
    pub piid: u32,
}
pub struct PropertyReading {
    pub query: PropertyQuery,
    pub value: Value,
}
impl fmt::Debug for PropertyReading {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PropertyReading([REDACTED])")
    }
}
#[derive(Deserialize)]
struct PropertyWire {
    did: PrivateString,
    siid: u32,
    piid: u32,
    code: i64,
    value: Option<Value>,
}

fn decode_properties(
    bytes: &[u8],
    did: &str,
    queries: &[PropertyQuery],
) -> Result<Vec<PropertyReading>> {
    let response: Envelope<Vec<PropertyWire>> = decode(bytes)?;
    if response.result.len() > queries.len() {
        return Err(invalid());
    }
    let mut seen = HashSet::new();
    let mut readings = vec![];
    for wire in response.result {
        let query = PropertyQuery {
            siid: wire.siid,
            piid: wire.piid,
        };
        if wire.did.as_str() != did || !queries.contains(&query) || !seen.insert(query) {
            return Err(invalid());
        }
        if wire.code != 0 {
            continue;
        }
        if let Some(value) = wire.value {
            if value.is_object()
                || value.is_array()
                || value.as_str().is_some_and(|s| s.len() > 256)
            {
                return Err(invalid());
            }
            readings.push(PropertyReading { query, value });
        }
    }
    Ok(readings)
}

#[cfg(test)]
mod tests;
