//! Product-scoped E1 start mappings; never infer controls from a display model alone.

use crate::control::CloudControl;
use homeverb_domain::{Action, Idempotency, Risk, Transport, VerificationMethod};
use serde_json::{Value, json};

pub const DISHWASHER_PROGRAMS: &[&str] = &[
    "auto_wash",
    "strong_wash",
    "standard_wash",
    "eco_wash",
    "glass_wash",
    "90min_wash",
    "fast_wash",
    "soak_wash",
    "hour_wash",
    "quietnight_wash",
    "germ",
    "self_clean",
    "fruit_wash",
    "soft_wash",
    "single_disinfect",
    "seafood_wash",
    "hotpot_wash",
];
pub const DISHWASHER_ADDITIONAL: [u8; 5] = [0, 9, 10, 13, 14];

const STANDARD: &[&str] = &[
    "auto_wash",
    "strong_wash",
    "standard_wash",
    "eco_wash",
    "glass_wash",
    "90min_wash",
    "fast_wash",
    "soak_wash",
    "hour_wash",
    "quietnight_wash",
    "germ",
    "self_clean",
    "fruit_wash",
];
const SHORT: &[&str] = &[
    "auto",
    "intensive",
    "normal",
    "eco",
    "glass",
    "90min",
    "rapid",
    "soak",
    "1hour",
    "quiet",
    "hygiene",
    "self_clean",
    "fruit",
];
const SOFT: &[&str] = &[
    "auto_wash",
    "strong_wash",
    "standard_wash",
    "eco_wash",
    "soft_wash",
    "fast_wash",
    "soak_wash",
    "self_clean",
    "fruit_wash",
];
const ADDITIONAL: &[&str] = &[
    "strong_wash",
    "standard_wash",
    "single_disinfect",
    "eco_wash",
    "glass_wash",
    "fast_wash",
    "soak_wash",
    "self_clean",
    "fruit_wash",
    "germ",
    "seafood_wash",
    "hotpot_wash",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DishwasherProfile {
    Short,
    Standard,
    Soft,
    Additional,
}

impl DishwasherProfile {
    pub(crate) fn action(self) -> Action {
        let programs = match self {
            Self::Short | Self::Standard => STANDARD,
            Self::Soft => SOFT,
            Self::Additional => ADDITIONAL,
        };
        let mut parameters = json!({"type":"object",
            "description":"Start the selected dishwasher program. This can consume water and heat.",
            "properties":{"program":{"type":"string","enum":programs}},
            "required":["program"],"additionalProperties":false});
        if self == Self::Additional {
            parameters["properties"]["additional"] =
                json!({"type":"integer","enum":DISHWASHER_ADDITIONAL});
            parameters["required"] = json!(["program", "additional"]);
        }
        Action {
            parameters,
            transports: vec![Transport::Cloud],
            idempotency: Idempotency::NonIdempotent,
            risk: Risk::Elevated,
            atomic: false,
            independent_fields: false,
            verification: VerificationMethod::Unavailable,
        }
    }

    pub(crate) fn control(self, arguments: &Value) -> homeverb_domain::Result<CloudControl> {
        self.action().validate_arguments(arguments)?;
        let program = arguments["program"].as_str().unwrap();
        let mode = match self {
            Self::Short => SHORT[STANDARD.iter().position(|item| *item == program).unwrap()],
            Self::Soft if program == "soft_wash" => "glass_wash",
            _ => program,
        };
        Ok(CloudControl::DishwasherStart {
            work_status: "work",
            mode: mode.into(),
            additional: arguments
                .get("additional")
                .map(|value| value.as_f64().unwrap() as u8),
        })
    }
}

pub(crate) fn select(subtype: Option<&str>, sn8: Option<&str>) -> Option<DishwasherProfile> {
    subtype?;
    match sn8? {
        "7600020L" => Some(DishwasherProfile::Short),
        "76006481" | "7600649Q" => Some(DishwasherProfile::Standard),
        "7600V1E7" => Some(DishwasherProfile::Soft),
        "760Y0026" => Some(DishwasherProfile::Additional),
        _ => None,
    }
}
