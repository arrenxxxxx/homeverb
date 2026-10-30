//! Fixed modern authentication origins. No credential-bearing cross-origin redirects.
use super::*;
use async_trait::async_trait;
use reqwest::{Client, header::HeaderValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModernAuthEndpoint {
    Bootstrap,
    Qr,
    Poll,
    Exchange,
}

pub struct ModernAuthRequest {
    endpoint: ModernAuthEndpoint,
    location: PrivateString,
    cookie: PrivateString,
    agent: PrivateString,
}
impl fmt::Debug for ModernAuthRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernAuthRequest([REDACTED])")
    }
}
impl ModernAuthRequest {
    pub(crate) fn new(
        endpoint: ModernAuthEndpoint,
        location: String,
        cookie: PrivateString,
        agent: PrivateString,
    ) -> Result<Self> {
        validate_url(endpoint, &location)?;
        if endpoint != ModernAuthEndpoint::Bootstrap && !cookie.as_str().is_empty() {
            return Err(invalid());
        }
        Ok(Self {
            endpoint,
            location: PrivateString::new(location),
            cookie,
            agent,
        })
    }
    pub fn endpoint(&self) -> ModernAuthEndpoint {
        self.endpoint
    }
    /// Protected request material for a trusted transport implementation only.
    pub fn url(&self) -> Result<Url> {
        validate_url(self.endpoint, self.location.as_str())
    }
    pub fn cookie(&self) -> &str {
        self.cookie.as_str()
    }
    pub fn user_agent(&self) -> &str {
        self.agent.as_str()
    }
}
pub struct ModernAuthResponse {
    pub body: SecretBytes,
    pub service_token: Option<SecretBytes>,
    pub user_id: Option<SecretBytes>,
    pub c_user_id: Option<SecretBytes>,
}
impl fmt::Debug for ModernAuthResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernAuthResponse([REDACTED])")
    }
}
#[async_trait]
pub trait ModernAuthTransport: Send + Sync {
    /// One validated GET, no redirects, retries, logs or cookies carried between origins.
    async fn send_modern_auth(
        &self,
        request: &ModernAuthRequest,
        operation: &Operation,
    ) -> Result<ModernAuthResponse>;
}
#[async_trait]
impl ModernAuthTransport for crate::MijiaHttps {
    async fn send_modern_auth(
        &self,
        request: &ModernAuthRequest,
        operation: &Operation,
    ) -> Result<ModernAuthResponse> {
        operation
            .run(async {
                let url = request.url()?;
                let client = crate::transport::client_builder()
                    .build()
                    .map_err(|_| crate::failed())?;
                transmit(client, url, request, operation).await
            })
            .await
    }
}

pub(super) fn validate_url(endpoint: ModernAuthEndpoint, value: &str) -> Result<Url> {
    let (host, path) = match endpoint {
        ModernAuthEndpoint::Bootstrap => ("account.xiaomi.com", "/pass/serviceLogin"),
        ModernAuthEndpoint::Qr => ("account.xiaomi.com", "/longPolling/loginUrl"),
        ModernAuthEndpoint::Poll => ("c3.lp.account.xiaomi.com", "/lp/s"),
        ModernAuthEndpoint::Exchange => ("sts.api.mijia.tech", "/mijia/sts"),
    };
    let url = exact_url(value, host, path)?;
    let fields = query(&url)?;
    match endpoint {
        ModernAuthEndpoint::Bootstrap => {
            if fields.len() != 3
                || fields.get("sid").map(PrivateString::as_str) != Some("mijia")
                || fields.get("_json").map(PrivateString::as_str) != Some("true")
                || fields.get("_locale").map(PrivateString::as_str) != Some("zh_CN")
            {
                return Err(invalid());
            }
        }
        ModernAuthEndpoint::Qr => {
            if fields.get("sid").map(PrivateString::as_str) != Some("mijia")
                || fields.get("callback").map(PrivateString::as_str) != Some(CALLBACK)
            {
                return Err(invalid());
            }
        }
        ModernAuthEndpoint::Poll => {
            if fields.len() != 1 || fields.get("k").is_none_or(|v| v.as_str().is_empty()) {
                return Err(invalid());
            }
        }
        ModernAuthEndpoint::Exchange => {
            if fields.is_empty() {
                return Err(invalid());
            }
        }
    }
    Ok(url)
}

pub(crate) async fn transmit(
    client: Client,
    url: Url,
    request: &ModernAuthRequest,
    operation: &Operation,
) -> Result<ModernAuthResponse> {
    operation
        .run(async {
            let mut outgoing = client
                .get(url)
                .header("accept-encoding", "identity")
                .header("user-agent", request.agent.as_str())
                .header("content-type", "application/x-www-form-urlencoded");
            if request.endpoint == ModernAuthEndpoint::Bootstrap {
                let mut cookie =
                    HeaderValue::from_str(request.cookie.as_str()).map_err(|_| invalid())?;
                cookie.set_sensitive(true);
                outgoing = outgoing.header("cookie", cookie);
            }
            let mut response = outgoing.send().await.map_err(|_| crate::failed())?;
            if matches!(response.status().as_u16(), 401 | 403) {
                return Err(auth_required());
            }
            if response.status() != reqwest::StatusCode::OK {
                return Err(crate::failed());
            }
            if response
                .content_length()
                .is_some_and(|v| v > MAX_AUTH_BYTES as u64)
            {
                return Err(invalid());
            }
            let mut token = None;
            let mut user_id = None;
            let mut c_user_id = None;
            if request.endpoint == ModernAuthEndpoint::Exchange {
                for cookie in response.headers().get_all("set-cookie") {
                    let cookie = cookie.to_str().map_err(|_| invalid())?;
                    let Some((name, value)) =
                        cookie.split(';').next().and_then(|v| v.split_once('='))
                    else {
                        continue;
                    };
                    let destination = match name {
                        "serviceToken" => &mut token,
                        "userId" => &mut user_id,
                        "cUserId" => &mut c_user_id,
                        _ => continue,
                    };
                    if destination.is_some() || !cookie_value(value) {
                        return Err(invalid());
                    }
                    *destination = Some(SecretBytes::new(value.as_bytes().to_vec()));
                }
            }
            let mut body = zeroize::Zeroizing::new(Vec::new());
            while let Some(chunk) = response.chunk().await.map_err(|_| crate::failed())? {
                if body.len() + chunk.len() > MAX_AUTH_BYTES {
                    return Err(invalid());
                }
                body.extend_from_slice(&chunk);
            }
            Ok(ModernAuthResponse {
                body: SecretBytes::new(std::mem::take(&mut *body)),
                service_token: token,
                user_id,
                c_user_id,
            })
        })
        .await
}
