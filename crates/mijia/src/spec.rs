//! Exact-model public MIoT evidence. Only reviewed readable semantics are projected.

mod actions;

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
                if !matches!(urn(&kind, "device")?.0, "fan" | "doorbell" | "lock")
                    && reference(model, &kind).is_none()
                {
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

pub(crate) fn urn<'a>(value: &'a str, category: &str) -> Result<(&'a str, &'a str)> {
    let parts: Vec<_> = value.split(':').collect();
    if value.len() > 512
        || !matches!(parts.len(), 7 | 8)
        || (parts.len() == 8
            && (parts[7].len() != 8 || !parts[7].bytes().all(|b| b.is_ascii_hexdigit())))
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
    source: Value,
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
        Self::decode_inner(model, kind, bytes, true)
    }

    fn decode_inner(
        model: &str,
        kind: &str,
        bytes: &[u8],
        include_reference: bool,
    ) -> Result<Self> {
        let (device, code) = urn(kind, "device")?;
        if !(include_reference && reference(model, kind).is_some()
            || matches!(
                (device, code),
                ("fan", "0000A005") | ("lock", "0000A038") | ("doorbell", "0000A03B")
            ))
        {
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
        if include_reference {
            append_reference(model, kind, bytes, &mut properties)?;
        }
        if properties.len() > 512 {
            return Err(invalid());
        }
        Ok(Self {
            model: model.into(),
            spec_type: kind.into(),
            properties,
            source: crate::decode(bytes)?,
        })
    }

    /// Derive a temporary numeric-log domain without changing persisted read mappings.
    pub(crate) fn numeric_history_definition(&self, siid: u32, piid: u32) -> Result<Value> {
        let source = self.source["services"]
            .as_array()
            .and_then(|services| services.iter().find(|s| s["iid"] == siid))
            .and_then(|service| service["properties"].as_array())
            .and_then(|properties| properties.iter().find(|p| p["iid"] == piid))
            .ok_or_else(invalid)?;
        let property: Property = serde_json::from_value(source.clone()).map_err(|_| invalid())?;
        let mut definition = property.definition()?.ok_or_else(invalid)?;
        if !matches!(definition["type"].as_str(), Some("number" | "integer"))
            || !property.access.iter().any(|a| a == "read")
            || property.range.is_some() && property.values.is_some()
        {
            return Err(invalid());
        }
        if let Some(values) = property.values {
            if values.is_empty()
                || values.len() > 128
                || values.iter().any(|v| !within_format(&v.value, &definition))
            {
                return Err(invalid());
            }
            definition["enum"] = json!(values.into_iter().map(|v| v.value).collect::<Vec<_>>());
        } else if let Some(range) = property.range {
            if range.len() != 3
                || !range.iter().all(|v| v.is_finite())
                || range[0] > range[1]
                || range[2] <= 0.0
                || !within_format(&json!(range[0]), &definition)
                || !within_format(&json!(range[1]), &definition)
                || definition["type"] == "integer" && range[2].fract() != 0.0
            {
                return Err(invalid());
            }
            // The pinned ranges are zero-aligned. Refuse a future offset grid
            // instead of silently changing its allowed values to multipleOf.
            let ratio = range[0] / range[2];
            if !ratio.is_finite() || (ratio - ratio.round()).abs() > 1e-9 {
                return Err(invalid());
            }
            definition["minimum"] = json!(range[0]);
            definition["maximum"] = json!(range[1]);
            definition["multipleOf"] = json!(range[2]);
        }
        definition["readable"] = json!(true);
        definition["writable"] = json!(false);
        Ok(definition)
    }

    /// Re-derive saved mappings from the same protected public-spec evidence. Public
    /// descriptors or edited derived fields cannot select another wire property.
    pub(crate) fn validate_saved(&self) -> Result<()> {
        if !crate::bounded_text(&self.model, 256) {
            return Err(invalid());
        }
        let bytes = serde_json::to_vec(&self.source).map_err(|_| invalid())?;
        let expected = Self::decode(&self.model, &self.spec_type, &bytes)?;
        let mut previous_hood = expected.clone();
        for property in &mut previous_hood.properties {
            if hood_manual_setting(&self.model, property.siid, property.piid) {
                property.write_domain = None;
            }
        }
        let mut previous_reference = expected.clone();
        for property in &mut previous_reference.properties {
            if reference_setting(&self.model, property.siid, property.piid)
                || outlet_setting(&self.model, property.siid, property.piid)
            {
                property.write_domain = None;
            }
        }
        let previous_settings = previous_reference.clone();
        if self.model == REFERENCE_FAN_MODEL && self.spec_type == REFERENCE_FAN_TYPE {
            for property in &mut previous_reference.properties {
                if property.siid == 2 && matches!(property.piid, 6..=9) {
                    property.write_domain = None;
                }
            }
        }
        if expected != *self
            && previous_hood != *self
            && previous_reference != *self
            && previous_settings != *self
            && Self::decode_inner(&self.model, &self.spec_type, &bytes, false)
                .ok()
                .as_ref()
                != Some(self)
        {
            return Err(invalid());
        }
        Ok(())
    }

    pub(crate) fn upgraded(&self) -> Result<Self> {
        self.validate_saved()?;
        Self::decode(
            &self.model,
            &self.spec_type,
            &serde_json::to_vec(&self.source).map_err(|_| invalid())?,
        )
    }

    pub(crate) fn pinned(model: &str, reported: Option<&str>) -> Result<Option<Self>> {
        let profiles = references();
        let matches: Vec<_> = profiles
            .iter()
            .filter(|p| p.model == model && reported.is_none_or(|kind| p.spec_type == kind))
            .collect();
        if matches.len() != 1 {
            return Ok(None);
        }
        let profile = matches[0];
        Self::decode(
            model,
            &profile.spec_type,
            &serde_json::to_vec(&profile.source).map_err(|_| invalid())?,
        )
        .map(Some)
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

/// Saved spec-derived choices, independent of Core authorization.
#[derive(Serialize)]
pub struct ControlOption {
    pub component_id: String,
    pub capability: String,
    pub action: String,
    pub definition: homeverb_domain::Action,
}

impl MappedProperty {
    pub(crate) fn control_family_enabled(&self, fan: bool, settings: bool) -> bool {
        if self.component.starts_with("fan-") {
            fan
        } else {
            settings
        }
    }
    pub(crate) fn control_option(&self) -> Option<ControlOption> {
        use homeverb_domain::{Action, Idempotency, Risk, Transport, VerificationMethod};
        // Only independently reviewed derivations have a write domain. Protected
        // payload validation re-derives the complete mapping before this is used.
        let domain = self.write_domain.as_ref()?;
        Some(ControlOption {
            component_id: self.component.clone(),
            capability: self.capability.clone(),
            action: if self.capability == "power" {
                "set".into()
            } else {
                format!("set_{}", self.property)
            },
            definition: Action {
                parameters: json!({"type":"object","properties":{self.property.clone():domain},"required":[self.property.clone()],"additionalProperties":false}),
                transports: vec![Transport::Cloud],
                idempotency: Idempotency::SetValue,
                risk: Risk::Low,
                atomic: false,
                independent_fields: false,
                verification: VerificationMethod::Unavailable,
            },
        })
    }
}

impl ModelMapping {
    pub(crate) fn control_options(&self) -> Vec<ControlOption> {
        self.properties
            .iter()
            .filter_map(|property| self.control_option(property))
            .collect()
    }
    pub(crate) fn control_option(&self, property: &MappedProperty) -> Option<ControlOption> {
        let mut option = property.control_option()?;
        if outlet_setting(&self.model, property.siid, property.piid) && property.siid != 13 {
            option.definition.risk = homeverb_domain::Risk::Elevated;
        }
        Some(option)
    }
    pub(crate) fn is_outlet(&self) -> bool {
        self.model == OUTLET_MODEL
    }
    pub(crate) fn control_family_enabled(
        &self,
        property: &MappedProperty,
        fan: bool,
        settings: bool,
        outlet: bool,
    ) -> bool {
        if self.is_outlet() {
            outlet
        } else {
            property.control_family_enabled(fan, settings)
        }
    }
    pub(crate) fn controlled_capabilities(
        &self,
        fan: bool,
        settings: bool,
        outlet: bool,
    ) -> Vec<Capability> {
        let mut capabilities = self.capabilities();
        for property in &self.properties {
            if !self.control_family_enabled(property, fan, settings, outlet) {
                continue;
            }
            if let Some(option) = self.control_option(property) {
                let capability = capabilities
                    .iter_mut()
                    .find(|c| {
                        c.id == option.capability
                            && c.component_id.as_ref() == Some(&option.component_id)
                    })
                    .unwrap();
                capability.properties.get_mut(&property.property).unwrap()["writable"] =
                    json!(true);
                capability.actions.insert(option.action, option.definition);
            }
        }
        capabilities
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
            && [
                "percentage",
                "arcdegrees",
                "celsius",
                "kelvin",
                "watt",
                "kWh",
                "A",
                "V",
                "ppm",
                "seconds",
                "minutes",
                "hours",
            ]
            .contains(&unit.as_str())
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

#[derive(Deserialize)]
struct ReferenceProfile {
    model: String,
    spec_type: String,
    source: Value,
}
fn references() -> &'static [ReferenceProfile] {
    static PROFILES: std::sync::OnceLock<Vec<ReferenceProfile>> = std::sync::OnceLock::new();
    PROFILES.get_or_init(|| {
        serde_json::from_str(include_str!("reference_profiles.json"))
            .expect("compiled reviewed reference catalog")
    })
}
fn reference(model: &str, kind: &str) -> Option<&'static ReferenceProfile> {
    references()
        .iter()
        .find(|p| p.model == model && p.spec_type == kind)
}
fn append_reference(
    model: &str,
    kind: &str,
    bytes: &[u8],
    properties: &mut Vec<MappedProperty>,
) -> Result<()> {
    let Some(profile) = reference(model, kind) else {
        return Ok(());
    };
    let actual: Value = crate::decode(bytes)?;
    let spec: Spec = crate::decode(bytes)?;
    for service in spec.services {
        let Some(known_service) = profile.source["services"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["iid"] == service.iid && s["type"] == service.kind)
        else {
            continue;
        };
        let mut ids = HashSet::new();
        for property in service.properties {
            if property.iid == 0 || !ids.insert(property.iid) {
                return Err(invalid());
            }
            if properties
                .iter()
                .any(|p| p.siid == service.iid && p.piid == property.iid)
            {
                continue;
            }
            let Some(known) = known_service["properties"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| {
                    p["iid"] == property.iid
                        && p["type"] == property.kind
                        && p["format"] == property.format
                })
            else {
                continue;
            };
            let actual_property = actual["services"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["iid"] == service.iid)
                .unwrap()["properties"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["iid"] == property.iid)
                .unwrap();
            if !property.access.iter().any(|a| a == "read")
                || actual_property.get("unit") != known.get("unit")
            {
                continue;
            }
            let Some(mut definition) = property.definition()? else {
                continue;
            };
            definition["readable"] = json!(true);
            definition["writable"] = json!(false);
            let service_name = service
                .kind
                .split(':')
                .nth(3)
                .ok_or_else(invalid)?
                .replace('-', "_");
            let name = property
                .kind
                .split(':')
                .nth(3)
                .ok_or_else(invalid)?
                .replace(['-', '.'], "_");
            let common_power = matches!(service_name.as_str(), "air_purifier" | "switch" | "light")
                && property.kind.split(':').nth(1) == Some("miot-spec-v2")
                && property.kind.split(':').nth(4) == Some("00000006")
                && name == "on"
                && property.format == "bool";
            let write_domain = if reference_setting(model, service.iid, property.iid)
                || outlet_setting(model, service.iid, property.iid)
            {
                exact_reference_domain(&property, known)?
            } else {
                reference_fan_domain(model, kind, service.iid, &service.kind, &property, known)?
            };
            properties.push(MappedProperty {
                siid: service.iid,
                piid: property.iid,
                component: format!("{service_name}-{}", service.iid),
                capability: if common_power {
                    "power".into()
                } else {
                    format!("vendor.xiaomi.mijia.{service_name}")
                },
                property: if common_power {
                    "state".into()
                } else {
                    format!("{name}_{}", property.iid)
                },
                definition,
                write_domain,
            });
        }
    }
    Ok(())
}

const REFERENCE_FAN_MODEL: &str = "jdjz.fan.2072fc";
const REFERENCE_FAN_TYPE: &str = "urn:miot-spec-v2:device:fan:0000A005:jdjz-2072fc:1:0000D062";

fn reference_fan_domain(
    model: &str,
    kind: &str,
    siid: u32,
    service: &str,
    property: &Property,
    known: &Value,
) -> Result<Option<Value>> {
    if model != REFERENCE_FAN_MODEL
        || kind != REFERENCE_FAN_TYPE
        || siid != 2
        || service != "urn:miot-spec-v2:service:fan:00007808:jdjz-2072fc:1:0000D062"
        || !matches!(property.iid, 6..=9)
        || !property.access.iter().any(|a| a == "write")
    {
        return Ok(None);
    }
    exact_reference_domain(property, known)
}

fn exact_reference_domain(property: &Property, known: &Value) -> Result<Option<Value>> {
    // append_reference correlates model, device/service/property URNs, format,
    // unit and readable access. Require the exact reviewed write domain too.
    if !property.access.iter().any(|a| a == "write") {
        return Ok(None);
    }
    let reviewed: Property = serde_json::from_value(known.clone()).map_err(|_| invalid())?;
    let Ok(domain) = property.write_domain() else {
        return Ok(None);
    };
    if domain != reviewed.write_domain()? {
        return Ok(None);
    }
    Ok(domain)
}

fn hood_manual_setting(model: &str, siid: u32, piid: u32) -> bool {
    model == "xiaomi.hood.cyp10" && matches!((siid, piid), (2 | 3, 1) | (13, 1 | 2))
}

fn reference_setting(model: &str, siid: u32, piid: u32) -> bool {
    match model {
        "xiaomi.hood.cyp10" => {
            (siid == 6 && matches!(piid, 1 | 2)) || hood_manual_setting(model, siid, piid)
        }
        "jdjz.fan.2072fc" => siid == 3 && piid == 5,
        "abhome.light.wy0a01" => siid == 2 && matches!(piid, 1 | 2 | 3 | 7 | 9 | 12 | 13),
        "zhimi.airpurifier.v6" => matches!((siid, piid), (2, 1..=3) | (5..=8, 1)),
        _ => false,
    }
}

/// Public pinned startup shape, not saved-device evidence or execution authority.
/// No account, network access or model inference occurs here.
pub fn reference_setting_option(component: &str, property: &str) -> Option<&'static ControlOption> {
    static OPTIONS: std::sync::OnceLock<Vec<ControlOption>> = std::sync::OnceLock::new();
    OPTIONS
        .get_or_init(|| {
            [
                "abhome.light.wy0a01",
                "zhimi.airpurifier.v6",
                "xiaomi.hood.cyp10",
                "jdjz.fan.2072fc",
            ]
            .into_iter()
            .flat_map(|model| {
                let mapping = ModelMapping::pinned(model, None)
                    .expect("reviewed reference")
                    .expect("pinned model");
                mapping
                    .properties
                    .iter()
                    .filter(|property| reference_setting(model, property.siid, property.piid))
                    .filter_map(|property| mapping.control_option(property))
                    .collect::<Vec<_>>()
            })
            .collect()
        })
        .iter()
        .find(|o| {
            o.component_id == component
                && o.definition.parameters["properties"]
                    .get(property)
                    .is_some()
        })
}

pub(crate) const OUTLET_MODEL: &str = "cuco.plug.v3";
fn outlet_setting(model: &str, siid: u32, piid: u32) -> bool {
    model == OUTLET_MODEL && matches!((siid, piid), (2, 1 | 2) | (7 | 13, 1))
}

/// Pinned shape for separately reviewed outlet grants. Not runtime authorization.
pub fn outlet_control_option(component: &str, property: &str) -> Option<&'static ControlOption> {
    static OPTIONS: std::sync::OnceLock<Vec<ControlOption>> = std::sync::OnceLock::new();
    OPTIONS
        .get_or_init(|| {
            ModelMapping::pinned(OUTLET_MODEL, None)
                .expect("reviewed outlet")
                .expect("pinned outlet")
                .control_options()
        })
        .iter()
        .find(|option| {
            option.component_id == component
                && option.definition.parameters["properties"]
                    .get(property)
                    .is_some()
        })
}
