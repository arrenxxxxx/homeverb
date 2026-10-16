//! Private product mapping evidence from the existing cloud inventory.

use homeverb_domain::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use zeroize::Zeroize;

/// Provider mapping evidence, not a public capability or device identifier.
/// ```compile_fail
/// fn export(profile: &homeverb_midea::ProductProfile) {
///     let _ = serde_json::to_string(profile);
/// }
/// ```
#[derive(Clone, Default, PartialEq, Eq)]
pub struct ProductProfile(StoredProfile);

impl fmt::Debug for ProductProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProductProfile([REDACTED])")
    }
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredProfile {
    sn8: Option<String>,
    subtype: Option<String>,
    category: Option<String>,
}

impl Drop for StoredProfile {
    fn drop(&mut self) {
        self.sn8.zeroize();
        self.subtype.zeroize();
        self.category.zeroize();
    }
}

impl StoredProfile {
    pub(crate) fn validate(&self) -> Result<()> {
        let valid = self
            .sn8
            .as_ref()
            .is_none_or(|value| valid_code(value) && value != "00000000")
            && self.subtype.as_ref().is_none_or(|value| {
                value
                    .parse::<u64>()
                    .is_ok_and(|number| number != 0 && number.to_string() == *value)
            })
            && self
                .category
                .as_ref()
                .is_none_or(|value| valid_category(value));
        if valid { Ok(()) } else { Err(crate::invalid()) }
    }
}

fn valid_code(value: &str) -> bool {
    value.len() == 8 && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn valid_category(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control)
}

fn text(value: Option<&Value>) -> Result<Option<&str>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if value.is_empty() => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        _ => Err(crate::invalid()),
    }
}

impl ProductProfile {
    pub(crate) fn temperature(&self) -> Option<crate::temperature::TemperatureProfile> {
        crate::temperature::select(
            self.0.subtype.as_deref(),
            self.0.sn8.as_deref(),
            self.0.category.as_deref(),
        )
    }

    pub(crate) fn dishwasher(&self) -> Option<crate::dishwasher::DishwasherProfile> {
        crate::dishwasher::select(self.0.subtype.as_deref(), self.0.sn8.as_deref())
    }

    pub(crate) fn parse(entry: &Value) -> Result<Self> {
        let sn8 = text(entry.get("sn8"))?.filter(|value| *value != "00000000");
        if sn8.is_some_and(|value| !valid_code(value)) {
            return Err(crate::invalid());
        }
        let category = text(entry.get("category"))?;
        if category.is_some_and(|value| !valid_category(value)) {
            return Err(crate::invalid());
        }
        let subtype = match entry.get("modelNumber") {
            None | Some(Value::Null) => None,
            Some(Value::String(value)) if value.is_empty() => None,
            Some(Value::String(value))
                if value.len() <= 20 && value.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                Some(value.parse::<u64>().map_err(|_| crate::invalid())?)
            }
            Some(Value::Number(value)) if value.is_u64() => value.as_u64(),
            _ => return Err(crate::invalid()),
        }
        .filter(|value| *value != 0)
        .map(|value| value.to_string());
        Ok(Self(StoredProfile {
            sn8: sn8.map(str::to_owned),
            subtype,
            category: category.map(str::to_owned),
        }))
    }

    pub(crate) fn from_stored(profile: &Option<StoredProfile>) -> Self {
        Self(profile.clone().unwrap_or_default())
    }

    pub(crate) fn stored(&self) -> Option<StoredProfile> {
        (self != &Self::default()).then(|| self.0.clone())
    }

    /// Check all known evidence before filling missing fields; never erase or replace it.
    pub(crate) fn merge(&mut self, incoming: &Self) -> Result<bool> {
        let fields = [
            (&self.0.sn8, &incoming.0.sn8),
            (&self.0.subtype, &incoming.0.subtype),
            (&self.0.category, &incoming.0.category),
        ];
        if fields
            .iter()
            .any(|(old, new)| matches!((old, new), (Some(a), Some(b)) if a != b))
        {
            return Err(Error::new(
                "DEVICE_IDENTITY_CONFLICT",
                "The account inventory contains conflicting product mapping evidence.",
            ));
        }
        let changed = fields
            .iter()
            .any(|(old, new)| old.is_none() && new.is_some());
        if self.0.sn8.is_none() {
            self.0.sn8.clone_from(&incoming.0.sn8);
        }
        if self.0.subtype.is_none() {
            self.0.subtype.clone_from(&incoming.0.subtype);
        }
        if self.0.category.is_none() {
            self.0.category.clone_from(&incoming.0.category);
        }
        Ok(changed)
    }
}
