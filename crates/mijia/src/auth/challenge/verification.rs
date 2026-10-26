//! Explicit provider-ticket continuation; no SMS sending or account-setting confirmation.
use super::*;
use crate::transport::verification::account_url;
use crate::{VerificationEndpoint as Endpoint, VerificationRequest, VerificationResponse};
use serde::Deserialize;
mod cookies;
pub(super) use cookies::Cookies;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMethod {
    Phone,
    Email,
}
impl VerificationMethod {
    fn endpoint(self) -> Endpoint {
        match self {
            Self::Phone => Endpoint::Phone,
            Self::Email => Endpoint::Email,
        }
    }
    fn path(self) -> &'static str {
        match self {
            Self::Phone => "/identity/auth/verifyPhone",
            Self::Email => "/identity/auth/verifyEmail",
        }
    }
    fn flag(self) -> &'static str {
        match self {
            Self::Phone => "4",
            Self::Email => "8",
        }
    }
}

/// Single-use, non-serializable provider interaction context.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::VerificationChallenge) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct VerificationChallenge {
    page: PrivateString,
    methods: Vec<VerificationMethod>,
    cookies: Cookies,
    region: Region,
    lifetime: Operation,
}
impl fmt::Debug for VerificationChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VerificationChallenge([REDACTED])")
    }
}
impl VerificationChallenge {
    /// Sensitive provider page for a future trusted foreground browser, never logs/general JSON.
    pub fn provider_page(&self) -> &str {
        self.page.as_str()
    }
    pub fn methods(&self) -> &[VerificationMethod] {
        &self.methods
    }
    pub fn expires_at(&self) -> tokio::time::Instant {
        self.lifetime.deadline
    }
    pub(super) async fn begin(
        transport: &dyn CloudTransport,
        mut context: Context,
        page: PrivateString,
        operation: &Operation,
    ) -> Result<Self> {
        let mut url = account_url(page.as_str())?;
        if url.path() != "/fe/service/identity/authStart" {
            return Err(required());
        }
        let mut cookies = match context.verification_cookies.take() {
            Some(cookies) => *cookies,
            None => Cookies::new(context.cookie.as_str())?,
        };
        let region = context.region;
        let lifetime = context.lifetime.clone();
        drop(context);
        url.set_path("/identity/list");
        lifetime
            .run(operation.run(async {
                let response = send(
                    transport,
                    Endpoint::Options,
                    url.as_str(),
                    &cookies,
                    SecretBytes::new(vec![]),
                    operation,
                )
                .await?;
                if response.status != 200 || response.location.is_some() {
                    return Err(invalid());
                }
                let options: Options = decode(response.body.expose())?;
                if options.code != 0 {
                    return Err(required());
                }
                cookies.absorb(&url, response.cookies)?;
                let mut methods = Vec::new();
                let options = options
                    .options
                    .or_else(|| options.flag.map(|v| vec![v]))
                    .ok_or_else(required)?;
                if options.len() > 16 {
                    return Err(invalid());
                }
                for flag in options {
                    let method = match flag {
                        4 => VerificationMethod::Phone,
                        8 => VerificationMethod::Email,
                        _ => continue,
                    };
                    if methods.contains(&method) {
                        return Err(invalid());
                    }
                    let target = account_url(method.path())?;
                    if cookies.has_identity_session(&target) {
                        methods.push(method);
                    }
                }
                if methods.is_empty() {
                    return Err(required());
                }
                Ok(Self {
                    page,
                    methods,
                    cookies,
                    region,
                    lifetime: lifetime.clone(),
                })
            }))
            .await
    }
    pub async fn answer(
        self,
        transport: &dyn CloudTransport,
        method: VerificationMethod,
        ticket: SecretBytes,
        operation: &Operation,
    ) -> Result<MijiaSession> {
        let lifetime = self.lifetime.clone();
        lifetime
            .run(operation.run(async {
                let value = std::str::from_utf8(ticket.expose()).map_err(|_| Error::invalid())?;
                if !crate::bounded_text(value, 2048) || !self.methods.contains(&method) {
                    return Err(Error::invalid());
                }
                let mut cookies = self.cookies;
                let url = account_url(method.path())?;
                if !cookies.has_identity_session(&url) {
                    return Err(required());
                }
                let body = codec::form(&[
                    ("_flag", method.flag()),
                    ("ticket", value),
                    ("trust", "false"),
                    ("_json", "true"),
                ]);
                drop(ticket);
                let response = send(
                    transport,
                    method.endpoint(),
                    url.as_str(),
                    &cookies,
                    body,
                    operation,
                )
                .await?;
                if response.status != 200 || response.location.is_some() {
                    return Err(invalid());
                }
                let answer: Answer = decode(response.body.expose())?;
                if answer.code != 0 {
                    return Err(required());
                }
                cookies.absorb(&url, response.cookies)?;
                let location = answer.location.ok_or_else(invalid)?;
                complete(transport, location.as_str(), &mut cookies, operation).await?;
                let sign_url =
                    "https://account.xiaomi.com/pass/serviceLogin?sid=xiaomiio&_json=true";
                let response = send(
                    transport,
                    Endpoint::SigningKey,
                    sign_url,
                    &cookies,
                    SecretBytes::new(vec![]),
                    operation,
                )
                .await?;
                if response.status != 200 || response.location.is_some() {
                    return Err(invalid());
                }
                let status: LoginStatus = decode(response.body.expose())?;
                if status.code != 0 || status.notification.is_some() || status.captcha.is_some() {
                    return Err(required());
                }
                let signed: Signed = decode(response.body.expose())?;
                if codec::unbase64(signed.security.as_str())?.len() != 16 {
                    return Err(invalid());
                }
                cookies.absorb(&account_url(sign_url)?, response.cookies)?;
                let supplied = signed.subject.map(|s| match s {
                    Subject::Text(s) => s,
                    Subject::Number(n) => PrivateString::new(n.to_string()),
                });
                let subject = supplied
                    .as_ref()
                    .map(PrivateString::as_str)
                    .or_else(|| cookies.subject())
                    .ok_or_else(invalid)?;
                numeric_subject(subject)?;
                if cookies.subject().is_some_and(|v| v != subject) {
                    return Err(invalid());
                }
                let subject = PrivateString::new(subject.to_owned());
                if let Some(location) = signed.location.filter(|v| !v.as_str().is_empty()) {
                    crate::transport::exchange_url(location.as_str())?;
                    complete(transport, location.as_str(), &mut cookies, operation).await?;
                }
                if cookies.subject().is_some_and(|v| v != subject.as_str()) {
                    return Err(invalid());
                }
                let token = cookies.token().ok_or_else(auth_required)?;
                Ok(MijiaSession {
                    region: self.region,
                    user_id: subject,
                    security: signed.security,
                    token: PrivateString::new(token.to_owned()),
                })
            }))
            .await
    }
}

#[derive(Deserialize)]
struct Options {
    code: i64,
    options: Option<Vec<u32>>,
    flag: Option<u32>,
}
#[derive(Deserialize)]
struct Answer {
    code: i64,
    location: Option<PrivateString>,
}
#[derive(Deserialize)]
struct Signed {
    #[serde(rename = "userId")]
    subject: Option<Subject>,
    #[serde(rename = "ssecurity")]
    security: PrivateString,
    location: Option<PrivateString>,
}
fn numeric_subject(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 32
        || !value.bytes().all(|b| b.is_ascii_digit())
        || value.bytes().all(|b| b == b'0')
    {
        return Err(invalid());
    }
    Ok(())
}
pub(super) async fn send_login(
    transport: &dyn CloudTransport,
    request: &CloudRequest,
    cookies: &mut Option<Box<Cookies>>,
    operation: &Operation,
) -> Result<crate::CloudResponse> {
    let Some(jar) = cookies else {
        return transport.send(request, operation).await;
    };
    let (endpoint, location) = match request.endpoint() {
        CloudEndpoint::LoginSign => (
            Endpoint::SigningKey,
            "https://account.xiaomi.com/pass/serviceLogin?sid=xiaomiio&_json=true",
        ),
        CloudEndpoint::LoginPassword => (
            Endpoint::Password,
            "https://account.xiaomi.com/pass/serviceLoginAuth2",
        ),
        _ => return Err(invalid()),
    };
    let url = account_url(location)?;
    let mut cookie = jar.header(&url);
    if let Some(ick) = request.cookie().split("; ").find(|v| v.starts_with("ick=")) {
        cookie = PrivateString::new(format!("{}; {ick}", cookie.as_str()));
    }
    let outgoing = VerificationRequest {
        endpoint,
        location: PrivateString::new(location.into()),
        cookie,
        body: SecretBytes::new(request.body().to_vec()),
    };
    let response = transport.send_verification(&outgoing, operation).await?;
    if response.status != 200 || response.location.is_some() {
        return Err(invalid());
    }
    jar.absorb(&url, response.cookies)?;
    Ok(crate::CloudResponse {
        body: response.body,
        service_token: None,
    })
}

async fn send(
    transport: &dyn CloudTransport,
    endpoint: Endpoint,
    location: &str,
    cookies: &Cookies,
    body: SecretBytes,
    operation: &Operation,
) -> Result<VerificationResponse> {
    let mut request = VerificationRequest {
        endpoint,
        location: PrivateString::new(location.to_owned()),
        cookie: PrivateString::new(String::new()),
        body,
    };
    let url = request.url()?;
    request.cookie = cookies.header(&url);
    transport.send_verification(&request, operation).await
}
async fn complete(
    transport: &dyn CloudTransport,
    location: &str,
    cookies: &mut Cookies,
    operation: &Operation,
) -> Result<()> {
    let mut location = PrivateString::new(location.to_owned());
    let mut visited = Vec::new();
    for _ in 0..4 {
        if !crate::bounded_text(location.as_str(), 8192)
            || !location.as_str().is_ascii()
            || location.as_str().contains('\\')
        {
            return Err(invalid());
        }
        let (endpoint, url) = if let Ok(url) = crate::transport::exchange_url(location.as_str()) {
            (Endpoint::Exchange, url)
        } else {
            let url = account_url(location.as_str())?;
            if url.path() != "/identity/result/check" {
                return Err(required());
            }
            (Endpoint::Completion, url)
        };
        if visited
            .iter()
            .any(|v: &PrivateString| v.as_str() == url.as_str())
        {
            return Err(invalid());
        }
        visited.push(PrivateString::new(url.to_string()));
        let response = send(
            transport,
            endpoint,
            url.as_str(),
            cookies,
            SecretBytes::new(vec![]),
            operation,
        )
        .await?;
        cookies.absorb(&url, response.cookies)?;
        if response.status == 200 {
            if response.location.is_some() {
                return Err(invalid());
            }
            return Ok(());
        }
        let next = response.location.ok_or_else(invalid)?;
        let next = std::str::from_utf8(next.expose()).map_err(|_| invalid())?;
        // A relative continuation is permitted only within the original account origin.
        if next.starts_with('/') && endpoint == Endpoint::Exchange {
            return Err(invalid());
        }
        location = PrivateString::new(next.to_owned());
    }
    Err(invalid())
}

#[cfg(test)]
mod tests;
