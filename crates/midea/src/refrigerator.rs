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
