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
    /// Sanitized failures retain positively returned devices without removing prior cards.
    pub issues: Vec<Error>,
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
            || !ids.insert(wire.did.clone())
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
        issues: vec![],
    })
}

#[derive(Deserialize)]
struct Homes {
    homelist: Vec<Home>,
    #[serde(default)]
    has_more: Option<bool>,
}
#[derive(Deserialize)]
struct Home {
    id: Id,
    uid: Id,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Id {
    Number(u64),
    Text(PrivateString),
}
impl Id {
    fn number(&self) -> Result<u64> {
        let n = match self {
            Self::Number(n) => *n,
            Self::Text(text) => text.as_str().parse().map_err(|_| invalid())?,
        };
        if n == 0 || n > i64::MAX as u64 {
            return Err(invalid());
        }
        Ok(n)
    }
}
#[derive(Deserialize)]
struct HomePage {
    device_info: Vec<DeviceWire>,
    has_more: bool,
    max_did: Option<PrivateString>,
}

impl MijiaSession {
    /// Explicit bounded union of account and accessible-home devices. Positive partial
    /// results remain usable as inventory evidence, never as physical online/state proof.
    pub async fn enumerate(
        &self,
        transport: &dyn CloudTransport,
        operation: &Operation,
    ) -> Result<Inventory> {
        operation.run(async {
            let mut result = self.inventory(transport,operation).await?;
            let homes_data=SecretBytes::new(br#"{"fg":true,"fetch_share":true,"fetch_share_dev":true,"fetch_cariot":true,"limit":300,"app_ver":7,"plat_form":0}"#.to_vec());
            let homes: Result<Envelope<Homes>>=async {
                let response=self.request(transport,CloudEndpoint::Homes,&homes_data,operation).await?;
                decode(response.expose())
            }.await;
            let homes=match homes { Ok(value)=>value.result, Err(error)=>{
                if stopped(&error) {return Err(error);}
                result.issues.push(error);return Ok(result);
            }};
            if homes.homelist.len()>300 {return Err(invalid());}
            if homes.has_more!=Some(false) {
                result.issues.push(Error::new("MIJIA_INVENTORY_PARTIAL","The home list does not prove complete account membership."));
            }
            let mut seen_homes=HashSet::new();
            for home in homes.homelist {
                let id=home.id.number()?;let owner=home.uid.number()?;
                if !seen_homes.insert((owner,id)) {return Err(invalid());}
                let mut cursor=PrivateString::new(String::new());
                let mut cursors=HashSet::new();
                for page_number in 0..16 {
                    #[derive(serde::Serialize)]
                    struct PageRequest<'a> { home_owner:u64,home_id:u64,limit:u32,start_did:&'a str,get_split_device:bool,support_smart_home:bool,get_cariot_device:bool,get_third_device:bool }
                    let input=PageRequest{home_owner:owner,home_id:id,limit:300,start_did:cursor.as_str(),get_split_device:false,support_smart_home:true,get_cariot_device:true,get_third_device:true};
                    let body=SecretBytes::new(serde_json::to_vec(&input).map_err(|_|invalid())?);
                    let page:Result<Envelope<HomePage>>=async {
                        let response=self.request(transport,CloudEndpoint::HomeDevices,&body,operation).await?;
                        decode(response.expose())
                    }.await;
                    let page=match page {Ok(value)=>value.result,Err(error)=>{
                        if stopped(&error) {return Err(error);}
                        result.issues.push(error);break;
                    }};
                    if page.device_info.len()>300 {return Err(invalid());}
                    let mut page_ids=HashSet::new();
                    for device in page.device_info {
                        if !page_ids.insert(device.did.clone()) {return Err(invalid());}
                        merge_device(&mut result.devices,device)?;
                    }
                    if !page.has_more {break;}
                    let next=page.max_did.ok_or_else(invalid)?;
                    if !crate::bounded_text(next.as_str(),128) || !cursors.insert(next.clone()) {return Err(invalid());}
                    cursor=next;
                    if page_number==15 {result.issues.push(Error::new("MIJIA_INVENTORY_PARTIAL","The device-page limit was reached."));}
                }
            }
            // The merged-home endpoint is community protocol evidence, not a guarantee
            // that every shared/legacy family is covered. Never invalidate absent devices.
            Ok(result)
        }).await
    }
}

fn stopped(error: &Error) -> bool {
    matches!(
        error.code.as_str(),
        "CANCELLED" | "DEADLINE_EXCEEDED" | "AUTH_REQUIRED"
    )
}

fn merge_device(devices: &mut Vec<CloudDevice>, wire: DeviceWire) -> Result<()> {
    if !crate::bounded_text(wire.did.as_str(), 128)
        || !crate::bounded_text(&wire.name, 256)
        || !crate::bounded_text(&wire.model, 256)
        || wire
            .spec
            .as_ref()
            .is_some_and(|s| !crate::bounded_text(s, 512))
    {
        return Err(invalid());
    }
    if let Some(existing) = devices.iter_mut().find(|d| d.did == wire.did) {
        if existing.model != wire.model
            || (existing.spec_type.is_some()
                && wire.spec.is_some()
                && existing.spec_type != wire.spec)
        {
            return Err(Error::new(
                "DEVICE_IDENTITY_CONFLICT",
                "Mijia inventory returned conflicting device-model evidence.",
            ));
        }
        if existing.spec_type.is_none() {
            existing.spec_type = wire.spec;
        }
    } else {
        if devices.len() >= 4096 {
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
    Ok(())
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
