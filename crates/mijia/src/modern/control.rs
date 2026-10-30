//! Fixed modern property/action control transport with an independent send boundary.
use super::*;
use async_trait::async_trait;
use homeverb_core::WriteProgress;
use homeverb_domain::Phase;
use reqwest::{Client, header::HeaderValue};

/// Closed control endpoints; neither exposes an arbitrary RPC route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModernControlEndpoint {
    SetProperty,
    Action,
}
impl ModernControlEndpoint {
    pub fn path(self) -> &'static str {
        match self {
            Self::SetProperty => "/miotspec/prop/set",
            Self::Action => "/miotspec/action",
        }
    }
    pub fn url(self) -> &'static str {
        match self {
            Self::SetProperty => "https://core.api.mijia.tech/app/miotspec/prop/set",
            Self::Action => "https://core.api.mijia.tech/app/miotspec/action",
        }
    }
}
pub struct ModernControlRequest {
    pub(super) endpoint: ModernControlEndpoint,
    pub(super) body: SecretBytes,
    pub(super) cookie: PrivateString,
}
impl fmt::Debug for ModernControlRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernControlRequest([REDACTED])")
    }
}
impl ModernControlRequest {
    pub fn path(&self) -> &'static str {
        self.endpoint.path()
    }
    pub fn url(&self) -> &'static str {
        self.endpoint.url()
    }
    /// Protected buffers are for trusted native transports, never host JSON or logs.
    pub fn body(&self) -> &[u8] {
        self.body.expose()
    }
    pub fn cookie(&self) -> &str {
        self.cookie.as_str()
    }
}
pub struct ModernControlResponse {
    pub body: SecretBytes,
    pub encoding: ModernContentEncoding,
}
impl fmt::Debug for ModernControlResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernControlResponse([REDACTED])")
    }
}
#[async_trait]
pub trait ModernControlTransport: Send + Sync {
    /// One tracked send, with no read-path substitution, retry, renewal or fallback.
    async fn send_modern_control(
        &self,
        request: &ModernControlRequest,
        progress: &WriteProgress,
        operation: &Operation,
    ) -> Result<ModernControlResponse>;
}
#[async_trait]
impl ModernControlTransport for crate::MijiaHttps {
    async fn send_modern_control(
        &self,
        request: &ModernControlRequest,
        progress: &WriteProgress,
        operation: &Operation,
    ) -> Result<ModernControlResponse> {
        operation
            .run(async {
                let client = crate::transport::client_builder()
                    .build()
                    .map_err(|_| crate::failed())?;
                transmit(
                    client,
                    Url::parse(request.url()).map_err(|_| invalid())?,
                    request,
                    progress,
                    operation,
                )
                .await
            })
            .await
    }
}
pub(crate) async fn transmit(
    client: Client,
    url: Url,
    request: &ModernControlRequest,
    progress: &WriteProgress,
    operation: &Operation,
) -> Result<ModernControlResponse> {
    operation
        .run(async {
            if progress.phase() != Phase::NotSent {
                return Err(invalid());
            }
            let mut cookie = HeaderValue::from_str(request.cookie()).map_err(|_| invalid())?;
            cookie.set_sensitive(true);
            let outgoing = client
                .post(url)
                .header("cookie", cookie)
                .header("content-type", "application/x-www-form-urlencoded")
                .header("accept-encoding", "identity")
                .header("miot-encrypt-algorithm", "ENCRYPT-RC4")
                .header("miot-accept-encoding", "GZIP")
                .header("user-agent", "Android-13-11.7.709-XiaomiHome")
                .body(request.body().to_vec())
                .build()
                .map_err(|_| crate::failed())?;
            operation.run(async { Ok(()) }).await?;
            // DNS/TLS failures after this boundary cannot prove that nothing was sent.
            progress.mark_sent();
            let mut response = client
                .execute(outgoing)
                .await
                .map_err(|_| crate::failed())?;
            if matches!(response.status().as_u16(), 401 | 403) {
                return Err(auth_required());
            }
            if response.status() != reqwest::StatusCode::OK {
                return Err(crate::failed());
            }
            if response
                .content_length()
                .is_some_and(|n| n > crate::MAX_RESPONSE_BYTES as u64)
            {
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
                if body.len() + chunk.len() > crate::MAX_RESPONSE_BYTES {
                    return Err(invalid());
                }
                body.extend_from_slice(&chunk);
            }
            Ok(ModernControlResponse {
                body: SecretBytes::new(std::mem::take(&mut *body)),
                encoding,
            })
        })
        .await
}
#[cfg(test)]
pub(crate) fn fixture_request(endpoint: ModernControlEndpoint) -> ModernControlRequest {
    ModernControlRequest { endpoint, body:SecretBytes::new(b"data=fictional".to_vec()),cookie:PrivateString::new("cUserId=fictional-user; serviceToken=fictional-modern-token; yetAnotherServiceToken=fictional-modern-token; countryCode=CN".into()) }
}
