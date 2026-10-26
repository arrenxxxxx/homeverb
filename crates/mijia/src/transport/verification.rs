//! One-send protected account verification, with explicit caller-owned continuation.
use super::*;
use homeverb_core::SecretBytes;

pub(crate) const MAX_VERIFICATION_BYTES: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationEndpoint {
    Password,
    Options,
    Phone,
    Email,
    Completion,
    SigningKey,
    Exchange,
}

pub struct VerificationRequest {
    pub(crate) endpoint: VerificationEndpoint,
    pub(crate) location: PrivateString,
    pub(crate) cookie: PrivateString,
    pub(crate) body: SecretBytes,
}
impl fmt::Debug for VerificationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VerificationRequest([REDACTED])")
    }
}
impl VerificationRequest {
    pub fn endpoint(&self) -> VerificationEndpoint {
        self.endpoint
    }
    /// Protected protocol material for a trusted transport, never a log/export field.
    pub fn location(&self) -> &str {
        self.location.as_str()
    }
    pub fn cookie(&self) -> &str {
        self.cookie.as_str()
    }
    pub fn body(&self) -> &[u8] {
        self.body.expose()
    }
    pub(crate) fn url(&self) -> Result<Url> {
        let url = match self.endpoint {
            VerificationEndpoint::Exchange => exchange_url(self.location.as_str())?,
            _ => account_url(self.location.as_str())?,
        };
        let expected = match self.endpoint {
            VerificationEndpoint::Password => "/pass/serviceLoginAuth2",
            VerificationEndpoint::Options => "/identity/list",
            VerificationEndpoint::Phone => "/identity/auth/verifyPhone",
            VerificationEndpoint::Email => "/identity/auth/verifyEmail",
            VerificationEndpoint::Completion => "/identity/result/check",
            VerificationEndpoint::SigningKey => "/pass/serviceLogin",
            VerificationEndpoint::Exchange => "/sts",
        };
        if url.path() != expected
            || self.body.expose().len() > 8192
            || self.cookie.as_str().len() > 16_384
        {
            return Err(invalid());
        }
        if self.endpoint == VerificationEndpoint::SigningKey
            && url.query() != Some("sid=xiaomiio&_json=true")
        {
            return Err(invalid());
        }
        if self.endpoint == VerificationEndpoint::Exchange && !self.cookie.as_str().is_empty() {
            return Err(invalid());
        }
        Ok(url)
    }
}

pub struct VerificationResponse {
    pub(crate) status: u16,
    pub(crate) body: SecretBytes,
    pub(crate) location: Option<SecretBytes>,
    pub(crate) cookies: Vec<SecretBytes>,
}
impl fmt::Debug for VerificationResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VerificationResponse([REDACTED])")
    }
}
impl VerificationResponse {
    /// Construct a bounded one-hop result; interpretation belongs to the login flow.
    pub fn new(
        status: u16,
        body: SecretBytes,
        location: Option<SecretBytes>,
        cookies: Vec<SecretBytes>,
    ) -> Result<Self> {
        if !matches!(status, 200 | 301 | 302 | 303 | 307 | 308)
            || body.expose().len() > MAX_VERIFICATION_BYTES
            || location.as_ref().is_some_and(|v| v.expose().len() > 8192)
            || cookies.len() > 16
            || cookies.iter().any(|v| v.expose().len() > 8192)
            || cookies.iter().map(|v| v.expose().len()).sum::<usize>() > 32_768
        {
            return Err(invalid());
        }
        Ok(Self {
            status,
            body,
            location,
            cookies,
        })
    }
}

pub(crate) fn account_url(value: &str) -> Result<Url> {
    if !crate::bounded_text(value, 8192)
        || !value.is_ascii()
        || value.contains('\\')
        || value.starts_with("//")
    {
        return Err(invalid());
    }
    let full = if value.starts_with('/') {
        format!("https://account.xiaomi.com{value}")
    } else {
        value.to_owned()
    };
    let url = Url::parse(&full).map_err(|_| invalid())?;
    if url.scheme() != "https"
        || url.host_str() != Some("account.xiaomi.com")
        || url.port().is_some_and(|p| p != 443)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid());
    }
    // Reject encoded path separators, dot-segment normalization and URL-parser repair.
    if url.as_str() != full
        && url.as_str().replace(
            "https://account.xiaomi.com/",
            "https://account.xiaomi.com:443/",
        ) != full
    {
        return Err(invalid());
    }
    Ok(url)
}

pub(super) async fn transmit_verification(
    client: Client,
    url: Url,
    request: &VerificationRequest,
    operation: &Operation,
) -> Result<VerificationResponse> {
    operation.run(async {
        let mut cookie = HeaderValue::from_str(request.cookie()).map_err(|_| invalid())?;
        cookie.set_sensitive(true);
        let method = if matches!(request.endpoint, VerificationEndpoint::Password | VerificationEndpoint::Phone | VerificationEndpoint::Email) { Method::POST } else { Method::GET };
        let mut outgoing = client.request(method, url)
            .header("accept-encoding", "identity")
            .header("content-type", "application/x-www-form-urlencoded")
            .header("user-agent", "Android-7.1.1-1.0.0-ONEPLUS A3010-136-ABCDEFABCDEF0 APP/xiaomi.smarthome APPV/62830")
            .body(request.body.expose().to_vec());
        if !request.cookie().is_empty() { outgoing = outgoing.header("cookie", cookie); }
        let mut response = outgoing.send().await.map_err(|_| failed())?;
        if matches!(response.status().as_u16(), 401 | 403) { return Err(auth_required()); }
        let status = response.status().as_u16();
        if !matches!(status, 200 | 301 | 302 | 303 | 307 | 308) { return Err(failed()); }
        if response.content_length().is_some_and(|v| v > MAX_VERIFICATION_BYTES as u64) { return Err(invalid()); }
        let mut locations = response.headers().get_all("location").iter();
        let location = locations.next().map(|v| SecretBytes::new(v.as_bytes().to_vec()));
        if locations.next().is_some() { return Err(invalid()); }
        let mut cookies = Vec::new(); let mut size = 0;
        for cookie in response.headers().get_all("set-cookie") {
            size += cookie.as_bytes().len();
            if cookies.len() == 16 || cookie.as_bytes().len() > 8192 || size > 32_768 { return Err(invalid()); }
            cookies.push(SecretBytes::new(cookie.as_bytes().to_vec()));
        }
        let mut body = zeroize::Zeroizing::new(Vec::new());
        while let Some(chunk) = response.chunk().await.map_err(|_| failed())? {
            if body.len() + chunk.len() > MAX_VERIFICATION_BYTES { return Err(invalid()); }
            body.extend_from_slice(&chunk);
        }
        VerificationResponse::new(status, SecretBytes::new(std::mem::take(&mut *body)), location, cookies)
    }).await
}
