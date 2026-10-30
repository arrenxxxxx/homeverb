//! Closed protected provenance; never selected by public device metadata.
use super::*;
use crate::ModernDeviceGroup;
use serde::ser::SerializeSeq;

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum BindingReadProtocol {
    #[default]
    #[serde(rename = "legacy")]
    Legacy,
    #[serde(rename = "modern_reference_v1")]
    Modern,
}
impl BindingReadProtocol {
    pub fn is_legacy(&self) -> bool {
        *self == Self::Legacy
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ModernBinding {
    pub kind: u32,
    #[serde(serialize_with = "serialize_parent")]
    pub parent: Option<PrivateString>,
    #[serde(serialize_with = "serialize_children")]
    pub logical_children: Vec<PrivateString>,
}
fn serialize_parent<S: serde::Serializer>(
    value: &Option<PrivateString>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    value.as_ref().map(|v| v.as_str()).serialize(serializer)
}
fn serialize_children<S: serde::Serializer>(
    value: &[PrivateString],
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    let mut sequence = serializer.serialize_seq(Some(value.len()))?;
    for child in value {
        sequence.serialize_element(child.as_str())?;
    }
    sequence.end()
}
impl ModernBinding {
    pub fn from_group(group: &ModernDeviceGroup) -> Result<Self> {
        let entry = group.entry();
        let kind = entry.kind.ok_or_else(unconfirmed)?.0;
        if !matches!(kind, 0 | 6 | 16 | 18) {
            return Err(unconfirmed());
        }
        let mut logical_children: Vec<_> = group
            .logical_children()
            .iter()
            .map(|e| e.did.clone())
            .collect();
        logical_children.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        Ok(Self {
            kind,
            parent: entry.parent.clone(),
            logical_children,
        })
    }
    pub fn extend(&mut self, next: Self) -> Result<()> {
        if self.kind != next.kind || self.parent != next.parent {
            return Err(conflict());
        }
        for child in next.logical_children {
            if !self.logical_children.contains(&child) {
                self.logical_children.push(child);
            }
        }
        self.logical_children
            .sort_by(|a, b| a.as_str().cmp(b.as_str()));
        Ok(())
    }
}
impl Binding {
    pub fn is_modern(&self) -> bool {
        self.read_protocol == BindingReadProtocol::Modern
    }
    pub fn provider_ids(&self) -> impl Iterator<Item = &PrivateString> {
        std::iter::once(&self.did).chain(self.modern.iter().flat_map(|m| m.logical_children.iter()))
    }
}
impl Payload {
    pub fn enable_modern_bindings(&mut self) {
        self.version = 3;
    }
    pub(super) fn validate_modern_bindings(&self) -> Result<()> {
        let mut identities = HashSet::new();
        let mut parents = std::collections::HashMap::new();
        for binding in &self.bindings {
            if binding.is_modern() {
                if self.version != 3
                    || binding.modern.is_none()
                    || self.modern.is_none()
                    || binding.mapping
                        != ModelMapping::pinned(&binding.model, binding.reported_spec.as_deref())
                            .map_err(|_| unconfirmed())?
                {
                    return Err(unconfirmed());
                }
            } else if self.session.is_none() {
                return Err(unconfirmed());
            }
            for identity in binding.provider_ids() {
                if !crate::bounded_text(identity.as_str(), 128)
                    || !identities.insert(identity.as_str())
                    || identities.len() > 4096
                {
                    return Err(unconfirmed());
                }
            }
            if let Some(modern) = &binding.modern {
                if self.version != 3
                    || !matches!(modern.kind, 0 | 6 | 16 | 18)
                    || modern
                        .parent
                        .as_ref()
                        .is_some_and(|p| !crate::bounded_text(p.as_str(), 128) || p == &binding.did)
                    || modern.logical_children.len() > 4096
                {
                    return Err(unconfirmed());
                }
                if let Some(parent) = &modern.parent {
                    parents.insert(binding.did.as_str(), parent.as_str());
                }
                for child in &modern.logical_children {
                    parents.insert(child.as_str(), binding.did.as_str());
                }
            }
        }
        if self.version == 3 && !self.bindings.iter().any(|b| b.modern.is_some()) {
            return Err(unconfirmed());
        }
        let mut visited = HashSet::new();
        for start in parents.keys() {
            let mut path = HashSet::new();
            let mut current = *start;
            while !visited.contains(current) {
                if !path.insert(current) {
                    return Err(unconfirmed());
                }
                let Some(parent) = parents.get(current) else {
                    break;
                };
                current = parent;
            }
            visited.extend(path);
        }
        Ok(())
    }
}
