//! Exact reference-origin reads. Logical directory entries are not physical Devices.
use super::*;
use crate::{ModelMapping, PropertyQuery, PropertyReading};
use homeverb_domain::Error;
use serde_json::json;
use std::collections::HashSet;

mod community;
mod property_history;
mod reconciliation;
mod statistics;
mod writes;
pub use reconciliation::{
    ModernDeviceGroup, ModernDeviceInventory, ModernUnresolvedEntry, ModernUnresolvedReason,
};

pub(crate) mod transport;
pub use transport::{
    ModernContentEncoding, ModernReadEndpoint, ModernReadRequest, ModernReadResponse,
    ModernReadTransport,
};
const MAX_READ_BYTES: usize = 8 * 1024 * 1024;
const MAX_PAGES: usize = 16;
const MAX_ENTRIES: usize = 4096;

/// Protected cloud membership, possibly a logical child. No physical identity is inferred.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::ModernDirectoryEntry) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct ModernDirectoryEntry {
    subject: PrivateString,
    encrypted_subject: PrivateString,
    pub(crate) did: PrivateString,
    name: String,
    model: String,
    spec_type: Option<String>,
    online_hint: Option<bool>,
    pub(crate) kind: Option<DirectoryKind>,
    pub(crate) parent: Option<PrivateString>,
    mapping: Option<ModelMapping>,
}
impl fmt::Debug for ModernDirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernDirectoryEntry([REDACTED])")
    }
}
impl ModernDirectoryEntry {
    pub(crate) fn cloud_device(&self) -> crate::CloudDevice {
        crate::CloudDevice {
            did: self.did.clone(),
            name: self.name.clone(),
            model: self.model.clone(),
            spec_type: self.spec_type.clone(),
            online_hint: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn model(&self) -> &str {
        &self.model
    }
    pub fn reported_spec(&self) -> Option<&str> {
        self.spec_type.as_deref()
    }
    /// Provider hint only, not a live observation or physical reachability proof.
    pub fn online_hint(&self) -> Option<bool> {
        self.online_hint
    }
    pub fn readable_properties(&self) -> Vec<PropertyQuery> {
        self.mapping
            .as_ref()
            .map(|m| {
                m.properties
                    .iter()
                    .map(|p| PropertyQuery {
                        siid: p.siid,
                        piid: p.piid,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
pub struct ModernDirectory {
    pub entries: Vec<ModernDirectoryEntry>,
    /// Always false until physical/shared membership and split children are reconciled.
    pub complete: bool,
    pub issues: Vec<Error>,
}
impl fmt::Debug for ModernDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernDirectory([REDACTED])")
    }
}

pub struct ModernHome {
    id: u64,
    owner: u64,
    name: Option<String>,
}
impl fmt::Debug for ModernHome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernHome([REDACTED])")
    }
}
impl ModernHome {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
pub struct ModernHomes {
    pub homes: Vec<ModernHome>,
    pub complete: bool,
    pub issues: Vec<Error>,
}
impl fmt::Debug for ModernHomes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernHomes([REDACTED])")
    }
}

impl ModernMijiaSession {
    /// Traverse the documented logical directory, with one shared foreground budget.
    pub async fn directory(
        &self,
        transport: &dyn ModernReadTransport,
        operation: &Operation,
    ) -> Result<ModernDirectory> {
        operation.run(async {
            let mut result = ModernDirectory { entries: Vec::new(), complete: false, issues: Vec::new() };
            let mut cursor: Option<PrivateString> = None;
            let mut cursors = HashSet::new();
            let mut identities = HashSet::new();
            for page_number in 0..MAX_PAGES {
                let mut data = json!({"getVirtualModel":true,"getHuamiDevices":1,"get_split_device":true,"support_smart_home":true,"get_cariot_device":true,"get_third_device":true,"get_phone_device":true,"get_miwear_device":true});
                if let Some(cursor) = &cursor { data["start_did"] = json!(cursor.as_str()); }
                let body = SecretBytes::new(serde_json::to_vec(&data).map_err(|_| invalid())?);
                let response = self.reference_request(transport, ModernReadEndpoint::Directory, &body, operation).await;
                let bytes = match response {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        if result.entries.is_empty() || matches!(error.code.as_str(), "AUTH_REQUIRED" | "CANCELLED" | "DEADLINE_EXCEEDED" | "MIJIA_INVALID_RESPONSE") { return Err(error); }
                        result.issues.push(error); return Ok(result);
                    }
                };
                let page: Envelope<DirectoryPage> = read_decode(bytes.expose())?;
                let page = page.result;
                if page.list.len() > MAX_ENTRIES || result.entries.len() + page.list.len() > MAX_ENTRIES || (page.has_more && page.list.is_empty()) { return Err(invalid()); }
                for wire in page.list {
                    operation.run(async { Ok(()) }).await?;
                    if !identities.insert(wire.did.clone()) {
                        let old = result.entries.iter().find(|v| v.did == wire.did).ok_or_else(invalid)?;
                        if old.model != wire.model || old.spec_type != wire.spec_type || old.kind != wire.pid || old.parent.as_ref().map(|p| p.as_str()) != wire.parent_id.as_ref().filter(|p| !p.as_str().is_empty()).map(|p| p.as_str()) { return Err(crate::protected::conflict()); }
                        return Err(invalid());
                    }
                    result.entries.push(self.directory_entry(wire)?);
                }
                if !page.has_more { return Ok(result); }
                let next = page.next_start_did.ok_or_else(invalid)?;
                if !crate::bounded_text(next.as_str(), 128) || !cursors.insert(next.clone()) { return Err(invalid()); }
                cursor = Some(next);
                if page_number + 1 == MAX_PAGES { result.issues.push(partial()); }
            }
            Ok(result)
        }).await
    }

    /// One documented home-list read; unknown paging remains explicitly incomplete.
    pub async fn homes(
        &self,
        transport: &dyn ModernReadTransport,
        operation: &Operation,
    ) -> Result<ModernHomes> {
        operation.run(async {
            let body = SecretBytes::new(br#"{"fg":true,"fetch_share":true,"fetch_share_dev":true,"limit":300,"app_ver":12,"fetch_cariot":true,"plat_form":0}"#.to_vec());
            let bytes = self.reference_request(transport, ModernReadEndpoint::Homes, &body, operation).await?;
            let response: Envelope<HomeList> = read_decode(bytes.expose())?;
            if response.result.homelist.len() > 300 { return Err(invalid()); }
            let mut homes = Vec::new();
            let mut ids = HashSet::new();
            for wire in response.result.homelist {
                let home = ModernHome { id: wire.id.number()?, owner: wire.uid.number()?, name: wire.name };
                if !ids.insert((home.owner, home.id)) || home.name.as_ref().is_some_and(|name| !crate::bounded_text(name, 256)) { return Err(invalid()); }
                homes.push(home);
            }
            Ok(ModernHomes { homes, complete: false, issues: if response.result.has_more == Some(false) { Vec::new() } else { vec![partial()] } })
        }).await
    }

    /// Read only exact pinned scalar properties of an entry returned for this subject.
    pub async fn properties(
        &self,
        transport: &dyn ModernReadTransport,
        entry: &ModernDirectoryEntry,
        queries: &[PropertyQuery],
        operation: &Operation,
    ) -> Result<Vec<PropertyReading>> {
        operation
            .run(async {
                if entry.subject != self.subject
                    || entry.encrypted_subject != self.encrypted_subject
                {
                    return Err(crate::protected::conflict());
                }
                let allowed = entry.readable_properties();
                if queries.is_empty()
                    || queries.len() > 90
                    || queries.iter().collect::<HashSet<_>>().len() != queries.len()
                {
                    return Err(Error::invalid());
                }
                if queries.iter().any(|q| !allowed.contains(q)) {
                    return Err(Error::new(
                        "CAPABILITY_UNSUPPORTED",
                        "The modern entry has no reviewed mapping for this property.",
                    ));
                }
                let params: Vec<_> = queries
                    .iter()
                    .map(|q| json!({"did":entry.did.as_str(),"siid":q.siid,"piid":q.piid}))
                    .collect();
                let body = SecretBytes::new(
                    serde_json::to_vec(&json!({"params":params})).map_err(|_| invalid())?,
                );
                let response = self
                    .reference_request(transport, ModernReadEndpoint::Properties, &body, operation)
                    .await?;
                crate::inventory::decode_properties(response.expose(), entry.did.as_str(), queries)
            })
            .await
    }

    pub(crate) fn protected_entry(
        &self,
        device: &crate::CloudDevice,
    ) -> Result<ModernDirectoryEntry> {
        self.directory_entry(EntryWire {
            did: device.did.clone(),
            name: device.name.clone(),
            model: device.model.clone(),
            spec_type: device.spec_type.clone(),
            online: None,
            pid: None,
            parent_id: None,
        })
    }

    fn directory_entry(&self, wire: EntryWire) -> Result<ModernDirectoryEntry> {
        let parent = wire.parent_id.filter(|p| !p.as_str().is_empty());
        if parent
            .as_ref()
            .is_some_and(|p| !crate::bounded_text(p.as_str(), 128))
        {
            return Err(invalid());
        }
        if !crate::bounded_text(wire.did.as_str(), 128)
            || !crate::bounded_text(&wire.name, 256)
            || !crate::bounded_text(&wire.model, 256)
            || wire
                .spec_type
                .as_ref()
                .is_some_and(|v| !crate::bounded_text(v, 512))
        {
            return Err(invalid());
        }
        let mapping = ModelMapping::pinned(&wire.model, wire.spec_type.as_deref())?;
        Ok(ModernDirectoryEntry {
            subject: self.subject.clone(),
            encrypted_subject: self.encrypted_subject.clone(),
            did: wire.did,
            name: wire.name,
            model: wire.model,
            spec_type: wire.spec_type,
            online_hint: wire.online,
            kind: wire.pid,
            parent,
            mapping,
        })
    }

    async fn reference_request(
        &self,
        transport: &dyn ModernReadTransport,
        endpoint: ModernReadEndpoint,
        data: &SecretBytes,
        operation: &Operation,
    ) -> Result<SecretBytes> {
        operation
            .run(async {
                self.validate()?;
                if data.expose().len() > MAX_READ_BYTES {
                    return Err(Error::invalid());
                }
                let text = std::str::from_utf8(data.expose()).map_err(|_| Error::invalid())?;
                let (body, key) = codec::encode_reference(
                    endpoint.path(),
                    text,
                    self.security.as_str(),
                    &codec::nonce()?,
                )?;
                let request = ModernReadRequest {
                    endpoint,
                    body,
                    agent: PrivateString::new("Android-13-11.7.709-XiaomiHome".into()),
                    cookie: PrivateString::new(format!(
                        "cUserId={}; serviceToken={}; yetAnotherServiceToken={}; countryCode=CN",
                        self.encrypted_subject.as_str(),
                        self.token.as_str(),
                        self.token.as_str()
                    )),
                };
                let response = transport.send_modern_read(&request, operation).await?;
                let decoded = decode_response(response, key.expose())?;
                operation.run(async { Ok(()) }).await?;
                Ok(decoded)
            })
            .await
    }
}
fn partial() -> Error {
    Error::new(
        "MIJIA_INVENTORY_PARTIAL",
        "The modern cloud response does not prove complete membership.",
    )
}
fn read_decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > MAX_READ_BYTES {
        return Err(invalid());
    }
    serde_json::from_slice(bytes).map_err(|_| invalid())
}
fn check_status(code: i64) -> Result<()> {
    match code {
        0 => Ok(()),
        2 | 3 | -3 | -6 | -10001 | -10002 | -10003 | -704010000 => Err(auth_required()),
        _ => Err(Error::new(
            "MIJIA_REQUEST_REJECTED",
            "Mijia rejected the modern request.",
        )),
    }
}
fn decode_response(response: ModernReadResponse, key: &[u8]) -> Result<SecretBytes> {
    decode_reference_response(response, key, MAX_READ_BYTES)
}
fn decode_reference_response(
    response: ModernReadResponse,
    key: &[u8],
    limit: usize,
) -> Result<SecretBytes> {
    if response.body.expose().len() > limit {
        return Err(invalid());
    }
    if response.body.expose().trim_ascii_start().first() == Some(&b'{') {
        let status: Status = read_decode(response.body.expose())?;
        check_status(status.code)?;
        return Err(invalid());
    }
    let decrypted = codec::decrypt(key, response.body.expose())?;
    let bytes = match response.encoding {
        ModernContentEncoding::Plain => decrypted,
        ModernContentEncoding::Gzip => {
            use std::io::Read;
            let mut decoder = flate2::bufread::GzDecoder::new(decrypted.expose());
            let mut output = zeroize::Zeroizing::new(Vec::new());
            decoder
                .by_ref()
                .take((limit + 1) as u64)
                .read_to_end(&mut output)
                .map_err(|_| invalid())?;
            if output.len() > limit || !decoder.into_inner().is_empty() {
                return Err(invalid());
            }
            SecretBytes::new(std::mem::take(&mut *output))
        }
    };
    let status: Status = read_decode(bytes.expose())?;
    check_status(status.code)?;
    Ok(bytes)
}

#[derive(Deserialize)]
struct Envelope<T> {
    result: T,
}
#[derive(Deserialize)]
struct DirectoryPage {
    list: Vec<EntryWire>,
    has_more: bool,
    next_start_did: Option<PrivateString>,
}
#[derive(Deserialize)]
struct EntryWire {
    pid: Option<DirectoryKind>,
    parent_id: Option<PrivateString>,
    did: PrivateString,
    name: String,
    model: String,
    spec_type: Option<String>,
    #[serde(rename = "isOnline")]
    online: Option<bool>,
}
/// Closed canonical representation; numeric and textual wire kinds are equivalent.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct DirectoryKind(pub(crate) u32);
impl<'de> Deserialize<'de> for DirectoryKind {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Number(u32),
            Text(String),
        }
        let number = match Wire::deserialize(deserializer)? {
            Wire::Number(n) => n,
            Wire::Text(text) => {
                if text.is_empty()
                    || text.len() > 10
                    || (text.len() > 1 && text.starts_with('0'))
                    || !text.bytes().all(|b| b.is_ascii_digit())
                {
                    return Err(serde::de::Error::custom("Invalid directory kind"));
                }
                text.parse()
                    .map_err(|_| serde::de::Error::custom("Invalid directory kind"))?
            }
        };
        Ok(Self(number))
    }
}
#[derive(Deserialize)]
struct HomeList {
    homelist: Vec<HomeWire>,
    has_more: Option<bool>,
}
#[derive(Deserialize)]
struct HomeWire {
    id: Id,
    uid: Id,
    name: Option<String>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Id {
    Number(u64),
    Text(PrivateString),
}
impl Id {
    fn number(self) -> Result<u64> {
        let n = match self {
            Self::Number(n) => n,
            Self::Text(v) => {
                if v.as_str().starts_with('0') || !v.as_str().bytes().all(|b| b.is_ascii_digit()) {
                    return Err(invalid());
                }
                v.as_str().parse().map_err(|_| invalid())?
            }
        };
        if n == 0 || n > i64::MAX as u64 {
            return Err(invalid());
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests;
