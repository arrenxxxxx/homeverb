//! Reviewed AC setpoint mappings from the pinned product profiles.

use crate::control::CloudControl;
use homeverb_domain::{Error, Result};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TemperatureProfile {
    pub minimum: u8,
    pub step: f64,
    pub fractional_field: bool,
}

impl TemperatureProfile {
    pub(crate) fn parameter(self) -> Value {
        json!({"type":"number","minimum":self.minimum,"maximum":30,"multipleOf":self.step})
    }

    pub(crate) fn control(self, value: &Value) -> Result<CloudControl> {
        let value = value.as_f64().ok_or_else(Error::invalid)?;
        if !value.is_finite()
            || value < f64::from(self.minimum)
            || value > 30.0
            || value % self.step != 0.0
        {
            return Err(Error::invalid());
        }
        Ok(CloudControl::Temperature {
            temperature: value as u8,
            small_temperature: self.fractional_field.then_some(value.fract()),
        })
    }
}

/// Never substitute a family default for missing higher-priority evidence.
pub(crate) fn select(
    subtype: Option<&str>,
    sn8: Option<&str>,
    category: Option<&str>,
) -> Option<TemperatureProfile> {
    let subtype = subtype?;
    let profile = |minimum, step, fractional_field| {
        Some(TemperatureProfile {
            minimum,
            step,
            fractional_field,
        })
    };
    if subtype == "524" {
        return profile(16, 1.0, true);
    }
    match sn8? {
        "23096245" => profile(17, 1.0, false),
        "22396961" | "22396963" | "22396965" | "22396969" | "22396973" | "22397057"
        | "22397059" | "22397061" | "22397063" | "22397065" | "22397067" | "22397091" => {
            profile(16, 0.5, true)
        }
        "22040055" | "22040023" | "22040053" | "22396505" | "22396517" | "22396521"
        | "22396525" | "22396533" | "22259015" | "22019031" | "22019035" | "22019045"
        | "23096653" | "23096633" | "22012369" | "22270043" | "22251077" => profile(17, 0.5, true),
        // These pinned overrides are not independent room-AC temperature setters.
        "22012895" | "21293193" | "17497071" | "10693145" | "106J6363" | "26093137"
        | "26093139" | "26096947" | "22012227" | "22013167" | "22013285" | "22251749" => None,
        _ if category.is_some_and(|value| value.replace('-', "_") == "wall_air_conditioner") => {
            profile(17, 0.5, true)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_selectors_preserve_priority_and_never_guess_missing_evidence() {
        for code in [
            "22040055", "22040023", "22040053", "22396505", "22396517", "22396521", "22396525",
            "22396533", "22259015", "22019031", "22019035", "22019045", "23096653", "23096633",
            "22012369", "22270043", "22251077",
        ] {
            assert_eq!(
                select(Some("1"), Some(code), None),
                Some(TemperatureProfile {
                    minimum: 17,
                    step: 0.5,
                    fractional_field: true
                })
            );
        }
        for code in [
            "22396961", "22396963", "22396965", "22396969", "22396973", "22397057", "22397059",
            "22397061", "22397063", "22397065", "22397067", "22397091",
        ] {
            assert_eq!(select(Some("1"), Some(code), None).unwrap().minimum, 16);
        }
        for code in [
            "22012895", "21293193", "17497071", "10693145", "106J6363", "26093137", "26093139",
            "26096947", "22012227", "22013167", "22013285", "22251749",
        ] {
            assert!(select(Some("1"), Some(code), Some("wall-air-conditioner")).is_none());
        }
        assert!(select(None, Some("22397091"), Some("wall-air-conditioner")).is_none());
        assert!(select(Some("1"), None, Some("wall-air-conditioner")).is_none());
        assert!(select(Some("1"), Some("TEST1234"), None).is_none());
        assert!(select(Some("1"), Some("TEST1234"), Some("central-air-conditioner")).is_none());
        assert_eq!(
            select(Some("524"), Some("21293193"), None).unwrap().step,
            1.0
        );
        assert_eq!(select(Some("524"), None, None).unwrap().minimum, 16);
        assert_eq!(
            select(Some("1"), Some("23096245"), Some("wall-air-conditioner"))
                .unwrap()
                .step,
            1.0
        );
        assert_eq!(
            select(Some("1"), Some("TEST1234"), Some("wall-air-conditioner"))
                .unwrap()
                .minimum,
            17
        );
        for category in ["wall_air_conditioner", "wall-air_conditioner"] {
            assert_eq!(
                select(Some("1"), Some("TEST1234"), Some(category)),
                select(Some("1"), Some("TEST1234"), Some("wall-air-conditioner"))
            );
            assert!(select(Some("1"), Some("26096947"), Some(category)).is_none());
        }
    }

    #[test]
    fn setpoints_are_exact_and_encodings_do_not_add_unrelated_fields() {
        for profile in [
            TemperatureProfile {
                minimum: 17,
                step: 0.5,
                fractional_field: true,
            },
            TemperatureProfile {
                minimum: 16,
                step: 1.0,
                fractional_field: true,
            },
            TemperatureProfile {
                minimum: 17,
                step: 1.0,
                fractional_field: false,
            },
            TemperatureProfile {
                minimum: 16,
                step: 0.5,
                fractional_field: true,
            },
        ] {
            for half in 30..=62 {
                let value = f64::from(half) / 2.0;
                let result = profile.control(&json!(value));
                let valid = value >= f64::from(profile.minimum)
                    && value <= 30.0
                    && value % profile.step == 0.0;
                assert_eq!(result.is_ok(), valid);
                if let Ok(control) = result {
                    let mut expected = json!({"temperature":value as u8});
                    if profile.fractional_field {
                        expected["small_temperature"] = json!(value.fract());
                    }
                    assert_eq!(json!(control), expected);
                }
            }
            for bad in [
                json!(null),
                json!(true),
                json!("26"),
                json!(26.25),
                json!(1e20),
            ] {
                assert!(profile.control(&bad).is_err());
            }
        }
    }
}
