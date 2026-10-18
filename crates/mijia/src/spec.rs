//! Exact-model public MIoT evidence. Only reviewed readable semantics are projected.

use crate::{CloudEndpoint, CloudRequest, CloudTransport, PrivateString, Region, invalid};
use homeverb_core::{Operation, SecretBytes};
use homeverb_domain::{Capability, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};

pub(crate) const MAX_INDEX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Deserialize)]
struct IndexEnvelope {
    instances: Vec<Instance>,
}
#[derive(Deserialize)]
struct Instance {
    status: String,
    model: String,
    version: u32,
    #[serde(rename = "type")]
    kind: String,
}
pub struct SpecIndex {
    entries: Vec<Instance>,
}

fn request(endpoint: CloudEndpoint, kind: Option<&str>) -> CloudRequest {
    CloudRequest {
        endpoint,
        region: Region::Cn,
        location: kind.map(|s| PrivateString::new(s.to_owned())),
        cookie: PrivateString::new(String::new()),
        body: SecretBytes::new(vec![]),
    }
}

impl SpecIndex {
    /// Explicit foreground metadata fetch, with no account cookies or retained worker.
    pub async fn fetch(transport: &dyn CloudTransport, operation: &Operation) -> Result<Self> {
        operation
            .run(async {
                let response = transport
                    .send(&request(CloudEndpoint::SpecIndex, None), operation)
                    .await?;
                Self::decode(response.body.expose())
            })
            .await
    }
    fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_INDEX_BYTES {
            return Err(invalid());
        }
        let index: IndexEnvelope = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        if index.instances.len() > 100_000 {
            return Err(invalid());
        }
        Ok(Self {
            entries: index.instances,
        })
    }
    pub fn resolve(&self, model: &str, reported_type: Option<&str>) -> Result<Option<String>> {
        let entries: Vec<_> = self
            .entries
            .iter()
            .filter(|v| v.model == model && v.status == "released")
            .collect();
        let selected: Vec<_> = if let Some(kind) = reported_type {
            entries.iter().filter(|v| v.kind == kind).copied().collect()
        } else {
            let version = entries.iter().map(|v| v.version).max();
            entries
                .iter()
                .filter(|v| Some(v.version) == version)
                .copied()
                .collect()
        };
        if selected.is_empty() {
            return Ok(None);
        }
        let kinds: HashSet<_> = selected.iter().map(|v| &v.kind).collect();
        if kinds.len() != 1 {
            return Err(invalid());
        }
        let kind = &selected[0].kind;
        validate_device_type(kind)?;
        Ok(Some(kind.to_owned()))
    }
    pub async fn mapping(
        &self,
        model: &str,
        reported_type: Option<&str>,
        transport: &dyn CloudTransport,
        operation: &Operation,
    ) -> Result<Option<ModelMapping>> {
        operation
            .run(async {
                let Some(kind) = self.resolve(model, reported_type)? else {
                    return Ok(None);
                };
                if !matches!(urn(&kind, "device")?.0, "fan" | "doorbell" | "lock") {
                    return Ok(None);
                }
                let response = transport
                    .send(
                        &request(CloudEndpoint::SpecInstance, Some(&kind)),
                        operation,
                    )
                    .await?;
                Ok(Some(ModelMapping::decode(
                    model,
                    &kind,
                    response.body.expose(),
                )?))
            })
            .await
    }
}

fn urn<'a>(value: &'a str, category: &str) -> Result<(&'a str, &'a str)> {
    let parts: Vec<_> = value.split(':').collect();
    if value.len() > 512
        || parts.len() != 7
        || parts[0] != "urn"
        || parts[1] != "miot-spec-v2"
        || parts[2] != category
        || parts[3].is_empty()
        || !parts[3]
            .bytes()
            .all(|v| v.is_ascii_lowercase() || v.is_ascii_digit() || v == b'-')
        || parts[4].len() != 8
        || !parts[4].bytes().all(|b| b.is_ascii_hexdigit())
        || parts[5].is_empty()
        || !parts[5]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        || parts[6].parse::<u32>().ok().filter(|v| *v > 0).is_none()
    {
        return Err(invalid());
    }
    Ok((parts[3], parts[4]))
}
pub(crate) fn validate_device_type(value: &str) -> Result<()> {
    urn(value, "device").map(|_| ())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MappedProperty {
    pub siid: u32,
    pub piid: u32,
    pub component: String,
    pub capability: String,
    pub property: String,
    pub definition: Value,
    /// Native wire permission and domain are evidence only, never a Core grant.
    pub(crate) write_domain: Option<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ModelMapping {
    pub model: String,
    pub spec_type: String,
    pub properties: Vec<MappedProperty>,
}
#[derive(Deserialize)]
struct Spec {
    #[serde(rename = "type")]
    kind: String,
    services: Vec<Service>,
}
#[derive(Deserialize)]
struct Service {
    iid: u32,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    properties: Vec<Property>,
}
#[derive(Deserialize)]
struct Property {
    iid: u32,
    #[serde(rename = "type")]
    kind: String,
    format: String,
    access: Vec<String>,
    unit: Option<String>,
    #[serde(rename = "value-range")]
    range: Option<Vec<f64>>,
    #[serde(rename = "value-list")]
    values: Option<Vec<EnumValue>>,
}
#[derive(Deserialize)]
struct EnumValue {
    value: Value,
}

impl ModelMapping {
    fn decode(model: &str, kind: &str, bytes: &[u8]) -> Result<Self> {
        let (device, code) = urn(kind, "device")?;
        if !matches!(
            (device, code),
            ("fan", "0000A005") | ("lock", "0000A038") | ("doorbell", "0000A03B")
        ) {
            return Err(invalid());
        }
        let spec: Spec = crate::decode(bytes)?;
        if spec.kind != kind || spec.services.len() > 64 {
            return Err(invalid());
        }
        let mut services = HashSet::new();
        let mut properties = vec![];
        for service in spec.services {
            if service.iid == 0 || !services.insert(service.iid) || service.properties.len() > 128 {
                return Err(invalid());
            }
            // Vendor URNs and identity/device-information services are deliberately not read.
            let Ok((name, service_code)) = urn(&service.kind, "service") else {
                continue;
            };
            let admitted = matches!(
                (name, service_code),
                ("fan", "00007808")
                    | ("battery", "00007805")
                    | ("lock", "00007855")
                    | ("doorbell", "00007857")
            );
            if !admitted || (name != device && name != "battery") {
                continue;
            }
            let component = if name == device {
                format!("{device}-{}", service.iid)
            } else {
                format!("battery-{}", service.iid)
            };
            let mut ids = HashSet::new();
            let mut semantics = HashSet::new();
            for property in service.properties {
                if property.iid == 0 || !ids.insert(property.iid) {
                    return Err(invalid());
                }
                if !property.access.iter().any(|a| a == "read") {
                    continue;
                }
                let Ok((prop, prop_code)) = urn(&property.kind, "property") else {
                    continue;
                };
                let mapping = match (name, prop, prop_code) {
                    ("fan", "on", "00000006") if property.format == "bool" => {
                        Some(("power", "state"))
                    }
                    ("fan", "fan-level", "00000016") => Some(("vendor.xiaomi.mijia.fan", "level")),
                    ("fan", "mode", "00000008") => Some(("vendor.xiaomi.mijia.fan", "mode")),
                    ("fan", "horizontal-swing", "00000017") if property.format == "bool" => {
                        Some(("vendor.xiaomi.mijia.fan", "horizontal_swing"))
                    }
                    ("fan", "horizontal-angle", "00000019") => {
                        Some(("vendor.xiaomi.mijia.fan", "horizontal_angle"))
                    }
                    ("lock", "status", "00000007") => Some(("vendor.xiaomi.mijia.lock", "status")),
                    ("doorbell", "volume", "00000013") => {
                        Some(("vendor.xiaomi.mijia.doorbell", "volume"))
                    }
                    ("battery", "battery-level", "00000014") => {
                        Some(("vendor.xiaomi.mijia.battery", "level"))
                    }
                    _ => None,
                };
                let Some((capability, semantic)) = mapping else {
                    continue;
                };
                if !matches!(semantic, "state" | "horizontal_swing")
                    && !matches!(
                        property.format.as_str(),
                        "uint8" | "uint16" | "uint32" | "int8" | "int16" | "int32" | "float"
                    )
                {
                    return Err(invalid());
                }
                if !semantics.insert(semantic) {
                    return Err(invalid());
                }
                let Some(mut definition) = property.definition()? else {
                    continue;
                };
                // Scalar status codes keep their native meaning, including unknown values.
                definition["readable"] = json!(true);
                definition["writable"] = json!(false);
                let write_domain = if name == "fan" && property.access.iter().any(|a| a == "write")
                {
                    property.write_domain()?
                } else {
                    None
                };
                properties.push(MappedProperty {
                    siid: service.iid,
                    piid: property.iid,
                    component: component.clone(),
                    capability: capability.into(),
                    property: semantic.into(),
                    definition,
                    write_domain,
                });
            }
        }
        if properties.len() > 64 {
            return Err(invalid());
        }
        Ok(Self {
            model: model.into(),
            spec_type: kind.into(),
            properties,
        })
    }

    pub fn capabilities(&self) -> Vec<Capability> {
        let mut groups: BTreeMap<(String, String), BTreeMap<String, Value>> = BTreeMap::new();
        for p in &self.properties {
            groups
                .entry((p.component.clone(), p.capability.clone()))
                .or_default()
                .insert(p.property.clone(), p.definition.clone());
        }
        groups
            .into_iter()
            .map(|((component, id), properties)| Capability {
                schema_version: "1.0".into(),
                id,
                component_id: Some(component),
                revision: 1,
                properties,
                actions: BTreeMap::new(),
                events: BTreeMap::new(),
            })
            .collect()
    }
}

impl Property {
    fn definition(&self) -> Result<Option<Value>> {
        let value = match self.format.as_str() {
            "bool" => json!({"type":"boolean"}),
            "uint8" => json!({"type":"integer","minimum":0,"maximum":255}),
            "uint16" => json!({"type":"integer","minimum":0,"maximum":65535}),
            "uint32" => json!({"type":"integer","minimum":0,"maximum":4294967295u64}),
            "int8" => json!({"type":"integer","minimum":-128,"maximum":127}),
            "int16" => json!({"type":"integer","minimum":-32768,"maximum":32767}),
            "int32" => json!({"type":"integer","minimum":-2147483648i64,"maximum":2147483647}),
            "float" => json!({"type":"number"}),
            // String fields can carry identity or credentials; no selected semantic needs them.
            _ => return Ok(None),
        };
        let mut value = value;
        if let Some(unit) = &self.unit
            && ["percentage", "arcdegrees"].contains(&unit.as_str())
        {
            value["unit"] = json!(unit);
        }
        Ok(Some(value))
    }
    fn write_domain(&self) -> Result<Option<Value>> {
        let Some(mut value) = self.definition()? else {
            return Ok(None);
        };
        value.as_object_mut().unwrap().remove("unit");
        if self.range.is_some() && self.values.is_some() {
            return Err(invalid());
        }
        if let Some(values) = &self.values {
            if values.is_empty() || values.len() > 128 {
                return Err(invalid());
            }
            for (index, item) in values.iter().enumerate() {
                if !within_format(&item.value, &value)
                    || values[..index]
                        .iter()
                        .any(|earlier| earlier.value == item.value)
                {
                    return Err(invalid());
                }
            }
            value["enum"] = json!(values.iter().map(|v| &v.value).collect::<Vec<_>>());
        } else if let Some(range) = &self.range {
            if range.len() != 3
                || !range.iter().all(|v| v.is_finite())
                || range[0] > range[1]
                || range[2] <= 0.0
            {
                return Err(invalid());
            }
            if !within_format(&json!(range[0]), &value)
                || !within_format(&json!(range[1]), &value)
                || (value["type"] == "integer" && range[2].fract() != 0.0)
            {
                return Err(invalid());
            }
            value["minimum"] = json!(range[0]);
            value["maximum"] = json!(range[1]);
            if range[0] % range[2] != 0.0 {
                return Ok(None);
            }
            value["multipleOf"] = json!(range[2]);
        } else if self.format != "bool" {
            return Ok(None);
        }
        Ok(Some(value))
    }
}

fn within_format(input: &Value, definition: &Value) -> bool {
    if definition["type"] == "boolean" {
        return input.is_boolean();
    }
    let Some(number) = input.as_f64() else {
        return false;
    };
    number.is_finite()
        && (definition["type"] != "integer" || number.fract() == 0.0)
        && definition
            .get("minimum")
            .and_then(Value::as_f64)
            .is_none_or(|min| number >= min)
        && definition
            .get("maximum")
            .and_then(Value::as_f64)
            .is_none_or(|max| number <= max)
}

#[cfg(test)]
mod tests;
