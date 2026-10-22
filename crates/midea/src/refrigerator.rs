//! Optional CA observations from the reviewed public mapping and official UI trace.
//! Field presence is not write admission or proof of physical feature support.

pub(crate) const FLAGS: &[&str] = &[
    "storage_mode",
    "freezing_mode",
    "intelligent_mode",
    "energy_saving_mode",
    "holiday_mode",
    "moisturize_mode",
    "preservation_mode",
    "acme_freezing_mode",
    "icea_mode_power",
    "freezing_ice_machine_power",
];

pub(crate) const TEMPERATURES: &[&str] = &[
    "left_flexzone_temperature",
    "right_flexzone_temperature",
    "left_variable_real_temperature",
    "right_variable_real_temperature",
];

/// Exact 310A2111 observations, kept in a separate descriptor scope so old
/// descriptors do not receive undeclared fields before an inventory refresh.
pub(crate) const DETAILS: &str = "vendor.midea.refrigerator.details";
pub(crate) const DETAIL_FLAGS: &[&str] = &[
    "deep_cold_mode",
    "plasma_aseptic_mode_power",
    "electronic_smell",
    "rapid_ice_making",
    "open_door_tips_switch",
    "telstar_saving_mode",
];
pub(crate) const DOORS: &[&str] = &[
    "storage_door_state",
    "freezer_door_state",
    "flexzone_door_state",
];
