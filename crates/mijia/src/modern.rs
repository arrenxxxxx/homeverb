//! Independent SID=mijia authentication. Never converts into a legacy cloud session.
use crate::{PrivateString, auth_required, codec, cookie_value, decode, invalid};
use homeverb_core::{Operation, SecretBytes};
use homeverb_domain::Result;
use reqwest::Url;
use serde::Deserialize;
use std::{collections::BTreeMap, fmt, time::Duration};

pub(crate) mod control;
pub(crate) mod reads;
pub use control::{
    ModernControlEndpoint, ModernControlRequest, ModernControlResponse, ModernControlTransport,
};
mod stored;
pub(crate) use stored::StoredModernSession;
pub(crate) mod transport;
pub use reads::{
    ModernContentEncoding, ModernDeviceGroup, ModernDeviceInventory, ModernDirectory,
    ModernDirectoryEntry, ModernHome, ModernHomes, ModernReadEndpoint, ModernReadRequest,
    ModernReadResponse, ModernReadTransport, ModernUnresolvedEntry, ModernUnresolvedReason,
};
pub use transport::{
    ModernAuthEndpoint, ModernAuthRequest, ModernAuthResponse, ModernAuthTransport,
};

const ACCOUNT: &str = "https://account.xiaomi.com";
const CALLBACK: &str = "https://sts.api.mijia.tech/mijia/sts";
const MAX_AUTH_BYTES: usize = 65_536;

struct ClientIdentity {
    device: PrivateString,
    pass_o: PrivateString,
    agent: PrivateString,
}
impl ClientIdentity {
    fn new() -> Result<Self> {
        fn random_hex(bytes: usize) -> Result<PrivateString> {
            let mut random = zeroize::Zeroizing::new(vec![0u8; bytes]);
            getrandom::fill(&mut random).map_err(|_| crate::failed())?;
            Ok(PrivateString::new(
                random.iter().map(|b| format!("{b:02X}")).collect(),
            ))
        }
        let device = random_hex(8)?;
        let pass_o = PrivateString::new(random_hex(8)?.as_str().to_ascii_lowercase());
        let id1 = random_hex(20)?;
        let id2 = random_hex(16)?;
        let id3 = random_hex(16)?;
        let id4 = random_hex(20)?;
        let agent = PrivateString::new(format!(
            "Android-15-11.0.701-Xiaomi-23046RP50C-OS2.0.212.0.VMYCNXM-{}-CN-{}-{}-SmartHome-MI_APP_STORE-{}|{}|{}-64",
            id1.as_str(),
            id3.as_str(),
            id2.as_str(),
            id1.as_str(),
            id4.as_str(),
            pass_o.as_str()
        ));
        Ok(Self {
            device,
            pass_o,
            agent,
        })
    }
    fn bootstrap(&self, session: Option<&ModernMijiaSession>) -> Result<ModernAuthRequest> {
        let mut cookie = format!(
            "deviceId={}; pass_o={}; uLocale=zh_CN",
            self.device.as_str(),
            self.pass_o.as_str()
        );
        if let Some(session) = session {
            session.validate()?;
            use std::fmt::Write;
            write!(
                cookie,
                "; passToken={}; userId={}; cUserId={}",
                session.pass_token.as_str(),
                session.subject.as_str(),
                session.encrypted_subject.as_str()
            )
            .map_err(|_| invalid())?;
        }
        ModernAuthRequest::new(
            ModernAuthEndpoint::Bootstrap,
            format!("{ACCOUNT}/pass/serviceLogin?_json=true&sid=mijia&_locale=zh_CN"),
            PrivateString::new(cookie),
            self.agent.clone(),
        )
    }
    fn request(&self, endpoint: ModernAuthEndpoint, url: String) -> Result<ModernAuthRequest> {
        ModernAuthRequest::new(
            endpoint,
            url,
            PrivateString::new(String::new()),
            self.agent.clone(),
        )
    }
}

/// Protected, single-use QR material for a trusted native renderer only.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::ModernQrChallenge) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct ModernQrChallenge {
    identity: ClientIdentity,
    scan: PrivateString,
    poll: PrivateString,
    lifetime: Operation,
}
impl fmt::Debug for ModernQrChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernQrChallenge([REDACTED])")
    }
}
impl ModernQrChallenge {
    /// Render locally as QR data. Do not log, open as a browser URL or serialize.
    pub fn scan_payload(&self) -> &[u8] {
        self.scan.as_str().as_bytes()
    }
    pub fn expires_at(&self) -> tokio::time::Instant {
        self.lifetime.deadline
    }

    /// One foreground long poll and one STS exchange. Any outcome consumes the challenge.
    pub async fn finish(
        self,
        transport: &dyn ModernAuthTransport,
        operation: &Operation,
    ) -> Result<ModernMijiaSession> {
        let lifetime = self.lifetime.clone();
        lifetime
            .run(operation.run(async {
                let request = self
                    .identity
                    .request(ModernAuthEndpoint::Poll, self.poll.as_str().to_owned())?;
                let mut poll_operation = operation.clone();
                poll_operation.deadline = poll_operation
                    .deadline
                    .min(lifetime.deadline)
                    .min(tokio::time::Instant::now() + Duration::from_secs(120));
                let response = poll_operation
                    .run(transport.send_modern_auth(&request, &poll_operation))
                    .await?;
                let status: Status = auth_decode(response.body.expose())?;
                if status.code != 0 {
                    return Err(auth_required());
                }
                let result: LoginResult = auth_decode(response.body.expose())?;
                let subject = subject(result.user_id)?;
                let pass_token = result.pass_token.ok_or_else(invalid)?;
                let encrypted_subject = result.c_user_id.ok_or_else(invalid)?;
                validate_material(&subject, &encrypted_subject, &pass_token, &result.ssecurity)?;
                let token = exchange(
                    transport,
                    &self.identity,
                    result.location,
                    &subject,
                    &encrypted_subject,
                    operation,
                )
                .await?;
                Ok(ModernMijiaSession {
                    identity: self.identity,
                    subject,
                    encrypted_subject,
                    pass_token,
                    security: result.ssecurity,
                    token,
                })
            }))
            .await
    }
}

/// Native SID=mijia credentials. No persistence or conversion to legacy sessions.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::ModernMijiaSession) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct ModernMijiaSession {
    identity: ClientIdentity,
    subject: PrivateString,
    encrypted_subject: PrivateString,
    pass_token: PrivateString,
    security: PrivateString,
    token: PrivateString,
}
impl fmt::Debug for ModernMijiaSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernMijiaSession([REDACTED])")
    }
}
impl ModernMijiaSession {
    fn validate(&self) -> Result<()> {
        validate_material(
            &self.subject,
            &self.encrypted_subject,
            &self.pass_token,
            &self.security,
        )?;
        if !cookie_value(self.token.as_str()) {
            return Err(invalid());
        }
        Ok(())
    }
    /// Explicit same-subject renewal. Failure leaves the caller's old session intact.
    pub async fn renew(
        &self,
        transport: &dyn ModernAuthTransport,
        operation: &Operation,
    ) -> Result<Self> {
        operation
            .run(async {
                let request = self.identity.bootstrap(Some(self))?;
                let response = transport.send_modern_auth(&request, operation).await?;
                let status: Status = auth_decode(response.body.expose())?;
                if status.code != 0 {
                    return Err(auth_required());
                }
                let result: LoginResult = auth_decode(response.body.expose())?;
                let subject = subject(result.user_id)?;
                if subject != self.subject
                    || result
                        .c_user_id
                        .as_ref()
                        .is_some_and(|v| v != &self.encrypted_subject)
                {
                    return Err(invalid());
                }
                let pass_token = result.pass_token.unwrap_or_else(|| self.pass_token.clone());
                validate_material(
                    &subject,
                    &self.encrypted_subject,
                    &pass_token,
                    &result.ssecurity,
                )?;
                let token = exchange(
                    transport,
                    &self.identity,
                    result.location,
                    &subject,
                    &self.encrypted_subject,
                    operation,
                )
                .await?;
                Ok(Self {
                    identity: ClientIdentity {
                        device: self.identity.device.clone(),
                        pass_o: self.identity.pass_o.clone(),
                        agent: self.identity.agent.clone(),
                    },
                    subject,
                    encrypted_subject: self.encrypted_subject.clone(),
                    pass_token,
                    security: result.ssecurity,
                    token,
                })
            })
            .await
    }
}

/// Starts a mainland, independent QR login; sends no account credentials or messages.
pub async fn begin_modern_qr_login(
    transport: &dyn ModernAuthTransport,
    operation: &Operation,
) -> Result<ModernQrChallenge> {
    let mut lifetime = operation.clone();
    lifetime.deadline = lifetime
        .deadline
        .min(tokio::time::Instant::now() + Duration::from_secs(180));
    lifetime
        .run(async {
            let identity = ClientIdentity::new()?;
            let response = transport
                .send_modern_auth(&identity.bootstrap(None)?, &lifetime)
                .await?;
            let status: Status = auth_decode(response.body.expose())?;
            if status.code != 70016 {
                return Err(auth_required());
            }
            let bootstrap: Bootstrap = auth_decode(response.body.expose())?;
            let url = exact_url(
                bootstrap.location.as_str(),
                "account.xiaomi.com",
                "/fe/service/login",
            )?;
            let mut fields = query(&url)?;
            if fields.get("sid").map(PrivateString::as_str) != Some("mijia")
                || fields.get("callback").map(PrivateString::as_str) != Some(CALLBACK)
                || fields.keys().any(|key| {
                    ![
                        "_json",
                        "_locale",
                        "_sign",
                        "callback",
                        "needTheme",
                        "qs",
                        "serviceParam",
                        "showActiveX",
                        "sid",
                    ]
                    .contains(&key.as_str())
                })
            {
                return Err(invalid());
            }
            for (key, value) in [
                ("theme", ""),
                ("bizDeviceType", ""),
                ("_hasLogo", "false"),
                ("_qrsize", "240"),
            ] {
                fields.insert(key.to_owned(), PrivateString::new(value.to_owned()));
            }
            fields.insert(
                "_dc".into(),
                PrivateString::new(chrono::Utc::now().timestamp_millis().to_string()),
            );
            let mut url =
                Url::parse(&format!("{ACCOUNT}/longPolling/loginUrl")).map_err(|_| invalid())?;
            {
                let mut pairs = url.query_pairs_mut();
                for (key, value) in &fields {
                    pairs.append_pair(key, value.as_str());
                }
            }
            let request = identity.request(ModernAuthEndpoint::Qr, url.to_string())?;
            let response = transport.send_modern_auth(&request, &lifetime).await?;
            let status: Status = auth_decode(response.body.expose())?;
            if status.code != 0 {
                return Err(auth_required());
            }
            let qr: QrResult = auth_decode(response.body.expose())?;
            let scan = exact_url(
                qr.login_url.as_str(),
                "c3.account.xiaomi.com",
                "/longPolling/login",
            )?;
            let scan_fields = query(&scan)?;
            if scan_fields.len() != 4
                || scan_fields.get("sid").map(PrivateString::as_str) != Some("mijia")
                || !["ticket", "dc", "ts"].iter().all(|key| {
                    scan_fields
                        .get(*key)
                        .is_some_and(|v| !v.as_str().is_empty())
                })
            {
                return Err(invalid());
            }
            transport::validate_url(ModernAuthEndpoint::Poll, qr.lp.as_str())?;
            Ok(ModernQrChallenge {
                identity,
                scan: qr.login_url,
                poll: qr.lp,
                lifetime: lifetime.clone(),
            })
        })
        .await
}

async fn exchange(
    transport: &dyn ModernAuthTransport,
    identity: &ClientIdentity,
    location: PrivateString,
    subject: &PrivateString,
    encrypted_subject: &PrivateString,
    operation: &Operation,
) -> Result<PrivateString> {
    let request = identity.request(ModernAuthEndpoint::Exchange, location.as_str().to_owned())?;
    let response = transport.send_modern_auth(&request, operation).await?;
    for (returned, expected) in [
        (&response.user_id, subject),
        (&response.c_user_id, encrypted_subject),
    ] {
        if returned
            .as_ref()
            .is_some_and(|v| v.expose() != expected.as_str().as_bytes())
        {
            return Err(invalid());
        }
    }
    let token = response.service_token.ok_or_else(auth_required)?;
    let token = std::str::from_utf8(token.expose()).map_err(|_| invalid())?;
    if !cookie_value(token) {
        return Err(invalid());
    }
    Ok(PrivateString::new(token.to_owned()))
}
fn auth_decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > MAX_AUTH_BYTES {
        return Err(invalid());
    }
    decode(bytes)
}
fn validate_material(
    subject: &PrivateString,
    encrypted_subject: &PrivateString,
    pass_token: &PrivateString,
    security: &PrivateString,
) -> Result<()> {
    if !cookie_value(encrypted_subject.as_str())
        || !cookie_value(pass_token.as_str())
        || subject.as_str().is_empty()
        || subject.as_str().len() > 32
        || subject.as_str().starts_with('0')
        || !subject.as_str().bytes().all(|v| v.is_ascii_digit())
        || codec::unbase64(security.as_str())?.len() != 16
    {
        return Err(invalid());
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Subject {
    Text(PrivateString),
    Number(u64),
}
fn subject(value: Subject) -> Result<PrivateString> {
    let text = match value {
        Subject::Text(v) => v,
        Subject::Number(v) => PrivateString::new(v.to_string()),
    };
    if text.as_str().is_empty()
        || text.as_str().len() > 32
        || text.as_str().starts_with('0')
        || !text.as_str().bytes().all(|v| v.is_ascii_digit())
    {
        return Err(invalid());
    }
    Ok(text)
}
#[derive(Deserialize)]
struct Status {
    code: i64,
}
#[derive(Deserialize)]
struct Bootstrap {
    location: PrivateString,
}
#[derive(Deserialize)]
struct QrResult {
    #[serde(rename = "loginUrl")]
    login_url: PrivateString,
    lp: PrivateString,
}
#[derive(Deserialize)]
struct LoginResult {
    #[serde(rename = "userId")]
    user_id: Subject,
    #[serde(rename = "cUserId")]
    c_user_id: Option<PrivateString>,
    #[serde(rename = "passToken")]
    pass_token: Option<PrivateString>,
    ssecurity: PrivateString,
    location: PrivateString,
}

fn exact_url(value: &str, host: &str, path: &str) -> Result<Url> {
    if !crate::bounded_text(value, 8192)
        || !value.is_ascii()
        || value.bytes().any(|b| b.is_ascii_whitespace())
        || value.contains('\\')
    {
        return Err(invalid());
    }
    let url = Url::parse(value).map_err(|_| invalid())?;
    if url.scheme() != "https"
        || url.host_str() != Some(host)
        || url.path() != path
        || url.port().is_some_and(|p| p != 443)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid());
    }
    Ok(url)
}
fn query(url: &Url) -> Result<BTreeMap<String, PrivateString>> {
    let mut result = BTreeMap::new();
    for (key, value) in url.query_pairs() {
        if result.len() >= 32
            || key.is_empty()
            || key.len() > 64
            || value.len() > 4096
            || key.chars().any(char::is_control)
            || value.chars().any(char::is_control)
            || result
                .insert(key.into_owned(), PrivateString::new(value.into_owned()))
                .is_some()
        {
            return Err(invalid());
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
