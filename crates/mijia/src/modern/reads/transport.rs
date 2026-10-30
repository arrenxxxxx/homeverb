//! Fixed source-specific read profiles, never a generic modern RPC transport.
use super::*;
use async_trait::async_trait;
use reqwest::{Client, header::HeaderValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModernReadEndpoint {
    Directory,
    Homes,
    Properties,
    /// Community statistics profile; distinct origin and per-field codec.
    Statistics,
}
impl ModernReadEndpoint {
    pub fn path(self) -> &'static str {
        match self {
            Self::Directory => "/v2/home/device_list_page",
            Self::Homes => "/v2/homeroom/gethome_merged",
            Self::Properties => "/miotspec/prop/get",
            Self::Statistics => "/v2/user/statistics",
        }
    }
    pub fn url(self) -> &'static str {
        match self {
            Self::Directory => "https://core.api.mijia.tech/app/v2/home/device_list_page",
            Self::Homes => "https://core.api.mijia.tech/app/v2/homeroom/gethome_merged",
            Self::Properties => "https://core.api.mijia.tech/app/miotspec/prop/get",
            Self::Statistics => "https://api.mijia.tech/app/v2/user/statistics",
        }
    }
}
pub struct ModernReadRequest {
    pub(super) endpoint: ModernReadEndpoint,
    pub(super) body: SecretBytes,
    pub(super) cookie: PrivateString,
    pub(super) agent: PrivateString,
}
impl fmt::Debug for ModernReadRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernReadRequest([REDACTED])")
    }
}
impl ModernReadRequest {
    pub fn endpoint(&self) -> ModernReadEndpoint {
        self.endpoint
    }
    /// Protected request buffers are exposed only to trusted transport implementations.
    pub fn body(&self) -> &[u8] {
        self.body.expose()
    }
    pub fn cookie(&self) -> &str {
        self.cookie.as_str()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModernContentEncoding {
    Plain,
    Gzip,
}
pub struct ModernReadResponse {
    pub body: SecretBytes,
    pub encoding: ModernContentEncoding,
}
impl fmt::Debug for ModernReadResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernReadResponse([REDACTED])")
    }
}
#[async_trait]
pub trait ModernReadTransport: Send + Sync {
    /// One fixed-origin POST. No retry, relogin, control or response logging.
    async fn send_modern_read(
        &self,
        request: &ModernReadRequest,
        operation: &Operation,
    ) -> Result<ModernReadResponse>;
}
#[async_trait]
impl ModernReadTransport for crate::MijiaHttps {
    async fn send_modern_read(
        &self,
        request: &ModernReadRequest,
        operation: &Operation,
    ) -> Result<ModernReadResponse> {
        operation
            .run(async {
                let url = Url::parse(request.endpoint.url()).map_err(|_| invalid())?;
                let client = crate::transport::client_builder()
                    .build()
                    .map_err(|_| crate::failed())?;
                transmit(client, url, request, operation).await
            })
            .await
    }
}
pub(crate) async fn transmit(
    client: Client,
    url: Url,
    request: &ModernReadRequest,
    operation: &Operation,
) -> Result<ModernReadResponse> {
    operation
        .run(async {
            let mut cookie =
                HeaderValue::from_str(request.cookie.as_str()).map_err(|_| invalid())?;
            cookie.set_sensitive(true);
            let limit = if request.endpoint == ModernReadEndpoint::Statistics {
                crate::MAX_RESPONSE_BYTES
            } else {
                MAX_READ_BYTES
            };
            let mut outgoing = client
                .post(url)
                .header("cookie", cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .header("accept-encoding", "identity")
                .header("miot-encrypt-algorithm", "ENCRYPT-RC4")
                .header("miot-accept-encoding", "GZIP")
                .header("user-agent", request.agent.as_str())
                .body(request.body.expose().to_vec());
            if request.endpoint == ModernReadEndpoint::Statistics {
                outgoing = outgoing.header("x-xiaomi-protocal-flag-cli", "PROTOCAL-HTTP2");
            }
            let mut response = outgoing.send().await.map_err(|_| crate::failed())?;
            if matches!(response.status().as_u16(), 401 | 403) {
                return Err(auth_required());
            }
            if response.status() != reqwest::StatusCode::OK {
                return Err(crate::failed());
            }
            if response.content_length().is_some_and(|v| v > limit as u64) {
                return Err(invalid());
            }
            let mut encoding = ModernContentEncoding::Plain;
            for name in ["content-encoding", "miot-content-encoding"] {
                let mut headers = response.headers().get_all(name).iter();
                if let Some(value) = headers.next() {
                    if headers.next().is_some() {
                        return Err(invalid());
                    }
                    let value = value.to_str().map_err(|_| invalid())?;
                    if name == "miot-content-encoding" && value.eq_ignore_ascii_case("GZIP") {
                        encoding = ModernContentEncoding::Gzip;
                    } else if !value.eq_ignore_ascii_case("identity") {
                        return Err(invalid());
                    }
                }
            }
            let mut body = zeroize::Zeroizing::new(Vec::new());
            while let Some(chunk) = response.chunk().await.map_err(|_| crate::failed())? {
                if body.len() + chunk.len() > limit {
                    return Err(invalid());
                }
                body.extend_from_slice(&chunk);
            }
            Ok(ModernReadResponse {
                body: SecretBytes::new(std::mem::take(&mut *body)),
                encoding,
            })
        })
        .await
}

#[cfg(test)]
pub(crate) fn fixture_request(endpoint: ModernReadEndpoint) -> ModernReadRequest {
    ModernReadRequest { endpoint, agent: PrivateString::new("Android-13-11.7.709-XiaomiHome".into()), body: SecretBytes::new(b"data=fictional".to_vec()), cookie: PrivateString::new("cUserId=fictional-c-user; serviceToken=fictional-service; yetAnotherServiceToken=fictional-service; countryCode=CN".into()) }
}
