//! Exact action derivation is independent of scalar write domains.
use super::*;
use crate::ActionQuery;
use homeverb_domain::{Action, Idempotency, Risk, VerificationMethod};

impl ModelMapping {
    pub(crate) fn no_argument_actions(&self) -> Vec<(ActionQuery, ControlOption)> {
        let Ok(profiles) = crate::action::profiles() else {
            return vec![];
        };
        profiles
            .into_iter()
            .filter_map(|profile| self.action_option(profile))
            .collect()
    }

    fn action_option(
        &self,
        profile: crate::action::Profile,
    ) -> Option<(ActionQuery, ControlOption)> {
        let query = ActionQuery {
            siid: profile.siid,
            aiid: profile.aiid,
        };
        if profile.model != self.model
            || profile.spec_type != self.spec_type
            || crate::action::admit(&self.model, &self.spec_type, &query).is_err()
            || self.source["type"] != self.spec_type
            || !self.source["services"]
                .as_array()?
                .iter()
                .any(|s| s["iid"] == profile.siid && s["type"] == profile.service_type)
        {
            return None;
        }
        // Select semantics by the closed exact tuple, never arbitrary source names.
        let (component, capability, name) =
            match (profile.model.as_str(), profile.siid, profile.aiid) {
                ("abhome.light.wy0a01", 2, aiid @ 1..=4) => {
                    let action = match aiid {
                        1 => "toggle",
                        2 => "brightness_up",
                        3 => "brightness_down",
                        _ => "toggle_light_mode",
                    };
                    ("light-2", "vendor.xiaomi.mijia.light", action)
                }
                ("jdjz.fan.2072fc", 2, 1) => ("fan-2", "vendor.xiaomi.mijia.fan", "toggle"),
                ("xiaomi.hood.cyp10", 6, 1) => ("light-6", "vendor.xiaomi.mijia.light", "toggle"),
                ("cuco.plug.v3" | "yutai.plug.jdls1", 2, 1) => {
                    ("switch-2", "vendor.xiaomi.mijia.switch", "toggle")
                }
                ("opple.switch.0010d5", 3, 1) => {
                    ("switch-3", "vendor.xiaomi.mijia.switch", "toggle")
                }
                ("opple.switch.0010d6", 11, 1) => {
                    ("switch-11", "vendor.xiaomi.mijia.switch", "toggle")
                }
                ("opple.switch.0010d6", 12, 1) => {
                    ("switch-12", "vendor.xiaomi.mijia.switch", "toggle")
                }
                _ => return None,
            };
        if !self
            .properties
            .iter()
            .any(|p| p.component == component && p.capability == capability)
        {
            return None;
        }
        Some((
            query,
            ControlOption {
                component_id: component.into(),
                capability: capability.into(),
                action: name.into(),
                definition: Action {
                    parameters: json!({"type":"object","properties":{},"additionalProperties":false}),
                    transports: vec![homeverb_domain::Transport::Cloud],
                    idempotency: Idempotency::NonIdempotent,
                    risk: if capability == "vendor.xiaomi.mijia.switch" {
                        Risk::Elevated
                    } else {
                        Risk::Low
                    },
                    atomic: false,
                    independent_fields: false,
                    verification: VerificationMethod::Unavailable,
                },
            },
        ))
    }
}

/// Closed native startup vocabulary; elevated actions additionally require trusted approval.
pub fn no_argument_action_option(
    component: &str,
    capability: &str,
    action: &str,
) -> Option<&'static ControlOption> {
    static OPTIONS: std::sync::LazyLock<Vec<ControlOption>> = std::sync::LazyLock::new(|| {
        [
            "abhome.light.wy0a01",
            "jdjz.fan.2072fc",
            "xiaomi.hood.cyp10",
            "cuco.plug.v3",
            "yutai.plug.jdls1",
            "opple.switch.0010d5",
            "opple.switch.0010d6",
        ]
        .into_iter()
        .filter_map(|model| ModelMapping::pinned(model, None).ok().flatten())
        .flat_map(|mapping| {
            mapping
                .no_argument_actions()
                .into_iter()
                .map(|(_, option)| option)
        })
        .collect()
    });
    OPTIONS
        .iter()
        .find(|o| o.component_id == component && o.capability == capability && o.action == action)
}
