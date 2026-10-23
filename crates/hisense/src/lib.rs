//! Offline Aijia washer profiles. No authentication, cloud transport or controls.

use chrono::{DateTime, Utc};
use homeverb_domain::{Capability, Error, Result};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::OnceLock};

pub const PROVIDER: &str = "hisense.aijia";
const MAX_CACHE_BYTES: usize = 65_536;

#[derive(Debug, Deserialize)]
struct Profiles {
    profiles: Vec<WasherProfile>,
}

/// Pinned product configuration, not a retail-model or physical-support claim.
#[derive(Debug, Deserialize)]
pub struct WasherProfile {
    product_code: String,
    category_code: String,
    partner_id: String,
    components: Vec<String>,
    fields: Vec<Field>,
    programs: Vec<Program>,
    missing_program_function_codes: Vec<String>,
}

/// Static program metadata. References do not establish complete parameter ranges.
#[derive(Debug, Deserialize)]
pub struct Program {
    pub component: String,
    pub code: String,
    pub value: String,
    pub function_codes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Field {
    wire: String,
    component: String,
    property: String,
    kind: String,
    #[serde(default)]
    values: BTreeMap<String, String>,
    minimum: Option<u64>,
    maximum: Option<u64>,
    unit: Option<String>,
}

/// Historical ordinary-property cache projection, deliberately not a Core Snapshot.
/// Capture time is not provider reception time or physical sampling evidence.
#[derive(Debug)]
pub struct CachedState {
    captured_at: DateTime<Utc>,
    components: BTreeMap<String, BTreeMap<String, Value>>,
}

impl CachedState {
    pub fn captured_at(&self) -> DateTime<Utc> {
        self.captured_at
    }

    pub fn components(&self) -> &BTreeMap<String, BTreeMap<String, Value>> {
        &self.components
    }

    pub fn is_stale(&self) -> bool {
        true
    }
}

impl WasherProfile {
    /// Exact saved identity metadata must match; names and virtual groups do not select a profile.
    /// This local selector does not establish account membership or identity authority.
    pub fn select(product: &str, category: &str, partner: &str) -> Result<&'static Self> {
        static PROFILES: OnceLock<Profiles> = OnceLock::new();
        PROFILES
            .get_or_init(|| {
                serde_json::from_str(include_str!("profiles.json"))
                    .expect("validated pinned Hisense profiles")
            })
            .profiles
            .iter()
            .find(|p| {
                p.product_code == product && p.category_code == category && p.partner_id == partner
            })
            .ok_or_else(|| {
                Error::new(
                    "CAPABILITY_UNSUPPORTED",
                    "The Hisense product identity has no reviewed washer profile.",
                )
            })
    }

    pub fn components(&self) -> &[String] {
        &self.components
    }

    pub fn programs<'a>(&'a self, component: &'a str) -> Result<impl Iterator<Item = &'a Program>> {
        if !self.components.iter().any(|c| c == component) {
            return Err(Error::invalid());
        }
        Ok(self
            .programs
            .iter()
            .filter(move |p| p.component == component))
    }

    pub fn missing_program_function_codes(&self) -> &[String] {
        &self.missing_program_function_codes
    }

    /// Read-only component definitions for future account-bound adapter admission.
    /// No Device, Connection, startup grant or callable host adapter is created.
    pub fn capabilities(&self) -> Vec<Capability> {
        let mut capabilities = BTreeMap::new();
        for field in &self.fields {
            let (id, property) = field
                .property
                .rsplit_once('.')
                .expect("pinned property key");
            let capability = capabilities
                .entry((field.component.clone(), id.to_owned()))
                .or_insert_with(|| Capability {
                    schema_version: "1.0".into(),
                    id: id.into(),
                    component_id: Some(field.component.clone()),
                    revision: 1,
                    properties: BTreeMap::new(),
                    actions: BTreeMap::new(),
                    events: BTreeMap::new(),
                });
            let mut definition = json!({
                "type": field.kind, "readable": true, "writable": false
            });
            match field.kind.as_str() {
                "string" => definition["enum"] = json!(field.values.values().collect::<Vec<_>>()),
                "integer" => {
                    definition["minimum"] = json!(field.minimum);
                    definition["maximum"] = json!(field.maximum);
                    definition["unit"] = json!(field.unit);
                }
                _ => {}
            }
            capability.properties.insert(property.into(), definition);
        }
        capabilities.into_values().collect()
    }

    /// Decode a flat map of ordinary App-cache state codes, never a gateway response.
    /// Only reviewed fields leave this function. Missing/null values stay absent.
    pub fn decode_cached_state(
        &self,
        bytes: &[u8],
        captured_at: DateTime<Utc>,
    ) -> Result<CachedState> {
        if bytes.len() > MAX_CACHE_BYTES {
            return Err(invalid_cache());
        }
        let input: BTreeMap<String, Value> =
            serde_json::from_slice(bytes).map_err(|_| invalid_cache())?;
        let mut components: BTreeMap<String, BTreeMap<String, Value>> = self
            .components
            .iter()
            .map(|c| (c.clone(), BTreeMap::new()))
            .collect();
        for field in &self.fields {
            if let Some(value) = input.get(&field.wire).filter(|v| !v.is_null()) {
                let raw = value.as_str().ok_or_else(invalid_cache)?;
                let value = match field.kind.as_str() {
                    "boolean" if field.values.contains_key(raw) => json!(raw == "1"),
                    "string" => json!(field.values.get(raw).ok_or_else(invalid_cache)?),
                    "integer" => {
                        if raw.is_empty() || !raw.bytes().all(|b| b.is_ascii_digit()) {
                            return Err(invalid_cache());
                        }
                        let number: u64 = raw.parse().map_err(|_| invalid_cache())?;
                        if field.minimum.is_some_and(|min| number < min)
                            || field.maximum.is_some_and(|max| number > max)
                        {
                            return Err(invalid_cache());
                        }
                        json!(number)
                    }
                    _ => return Err(invalid_cache()),
                };
                components
                    .get_mut(&field.component)
                    .expect("pinned component")
                    .insert(field.property.clone(), value);
            }
        }
        Ok(CachedState {
            captured_at,
            components,
        })
    }
}

fn invalid_cache() -> Error {
    Error::new(
        "HISENSE_INVALID_CACHE",
        "The Hisense cache does not match the reviewed read-only state mapping.",
    )
}
