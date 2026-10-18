//! Fixed-origin, one-send HTTPS requests owned by the current operation.

use crate::{MAX_RESPONSE_BYTES, PrivateString, Region, auth_required, failed, invalid};
use async_trait::async_trait;
use homeverb_core::{Operation, SecretBytes};
use homeverb_domain::Result;
use reqwest::{
    Client, Method, Url,
    header::{HeaderMap, HeaderValue},
};
use std::{fmt, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudEndpoint {
    LoginSign,
    LoginPassword,
    LoginExchange,
    Devices,
    Properties,
    Homes,
    HomeDevices,
    SpecIndex,
    SpecInstance,
}
impl CloudEndpoint {
    pub fn path(self) -> &'static str {
        match self {
            Self::LoginSign => "/pass/serviceLogin",
            Self::LoginPassword => "/pass/serviceLoginAuth2",
            Self::LoginExchange => "/sts",
            Self::Devices => "/home/device_list",
            Self::Properties => "/miotspec/prop/get",
            Self::Homes => "/v2/homeroom/gethome_merged",
            Self::HomeDevices => "/v2/home/home_device_list",
            Self::SpecIndex => "/miot-spec-v2/instances",
            Self::SpecInstance => "/miot-spec-v2/instance",
        }
    }
}

/// Private native transport material, never a serializable host response.
pub struct CloudRequest {
    pub(crate) endpoint: CloudEndpoint,
    pub(crate) region: Region,
    pub(crate) location: Option<PrivateString>,
    pub(crate) cookie: PrivateString,
    pub(crate) body: SecretBytes,
}
impl fmt::Debug for CloudRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CloudRequest([REDACTED])")
    }
}
impl CloudRequest {
    pub fn endpoint(&self) -> CloudEndpoint {
        self.endpoint
    }
    pub fn body(&self) -> &[u8] {
        self.body.expose()
    }
    pub fn cookie(&self) -> &str {
        self.cookie.as_str()
    }
    pub fn region(&self) -> Region {
        self.region
    }
    fn url(&self) -> Result<Url> {
        let value = match self.endpoint {
            CloudEndpoint::SpecIndex => {
                "https://miot-spec.org/miot-spec-v2/instances?status=all".to_owned()
            }
            CloudEndpoint::SpecInstance => {
                let kind = self.location.as_ref().ok_or_else(invalid)?;
                crate::spec::validate_device_type(kind.as_str())?;
                return Url::parse_with_params(
                    "https://miot-spec.org/miot-spec-v2/instance",
                    &[("type", kind.as_str())],
                )
                .map_err(|_| invalid());
            }
            CloudEndpoint::LoginSign => {
                "https://account.xiaomi.com/pass/serviceLogin?sid=xiaomiio&_json=true".to_owned()
            }
            CloudEndpoint::LoginPassword => {
                "https://account.xiaomi.com/pass/serviceLoginAuth2".to_owned()
            }
            CloudEndpoint::LoginExchange => {
                return exchange_url(self.location.as_ref().ok_or_else(invalid)?.as_str());
            }
            _ => format!("{}/app{}", self.region.origin(), self.endpoint.path()),
        };
        Url::parse(&value).map_err(|_| invalid())
    }
}

pub(crate) fn exchange_url(value: &str) -> Result<Url> {
    let url = Url::parse(value).map_err(|_| invalid())?;
    if value.len() > 8192
        || url.scheme() != "https"
        || url.host_str() != Some("sts.api.io.mi.com")
        || url.port().is_some_and(|p| p != 443)
        || url.path() != "/sts"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid());
    }
    Ok(url)
}

pub struct CloudResponse {
    pub body: SecretBytes,
    pub service_token: Option<SecretBytes>,
}
impl fmt::Debug for CloudResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CloudResponse([REDACTED])")
    }
}

#[async_trait]
pub trait CloudTransport: Send + Sync {
    /// Send once, with no logging, retry, redirects or deadline extension.
    async fn send(&self, request: &CloudRequest, operation: &Operation) -> Result<CloudResponse>;
}

#[derive(Default)]
pub struct MijiaHttps;

fn client_builder() -> reqwest::ClientBuilder {
    Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .http1_only()
        .pool_max_idle_per_host(0)
        .connect_timeout(Duration::from_secs(10))
}

#[async_trait]
impl CloudTransport for MijiaHttps {
    async fn send(&self, request: &CloudRequest, operation: &Operation) -> Result<CloudResponse> {
        operation
            .run(async {
                let url = request.url()?;
                let client = client_builder().build().map_err(|_| failed())?;
                transmit(client, url, request, operation).await
            })
            .await
    }
}

async fn transmit(
    client: Client,
    url: Url,
    request: &CloudRequest,
    operation: &Operation,
) -> Result<CloudResponse> {
    operation.run(async {
        let mut headers = HeaderMap::new();
        let mut cookie = HeaderValue::from_str(request.cookie.as_str()).map_err(|_| invalid())?;
        cookie.set_sensitive(true);
        if !matches!(request.endpoint, CloudEndpoint::SpecIndex | CloudEndpoint::SpecInstance) { headers.insert("cookie", cookie); }
        headers.insert("content-type", HeaderValue::from_static("application/x-www-form-urlencoded"));
        headers.insert("accept-encoding", HeaderValue::from_static("identity"));
        headers.insert("user-agent", HeaderValue::from_static("Android-7.1.1-1.0.0-ONEPLUS A3010-136-ABCDEFABCDEF0 APP/xiaomi.smarthome APPV/62830"));
        if matches!(request.endpoint, CloudEndpoint::Devices | CloudEndpoint::Properties | CloudEndpoint::Homes | CloudEndpoint::HomeDevices) {
            headers.insert("miot-encrypt-algorithm", HeaderValue::from_static("ENCRYPT-RC4"));
            headers.insert("x-xiaomi-protocal-flag-cli", HeaderValue::from_static("PROTOCAL-HTTP2"));
        }
        let method = if matches!(request.endpoint, CloudEndpoint::LoginSign | CloudEndpoint::LoginExchange | CloudEndpoint::SpecIndex | CloudEndpoint::SpecInstance) { Method::GET } else { Method::POST };
        let outgoing = client.request(method, url).headers(headers).body(request.body.expose().to_vec()).build().map_err(|_| failed())?;
        let mut response = client.execute(outgoing).await.map_err(|_| failed())?;
        if matches!(response.status().as_u16(), 401 | 403) { return Err(auth_required()); }
        if response.status() != reqwest::StatusCode::OK { return Err(failed()); }
        let limit = if request.endpoint == CloudEndpoint::SpecIndex { crate::spec::MAX_INDEX_BYTES } else { MAX_RESPONSE_BYTES };
        if response.content_length().is_some_and(|v| v > limit as u64) { return Err(invalid()); }
        let mut token = None;
        if request.endpoint == CloudEndpoint::LoginExchange {
            for cookie in response.headers().get_all("set-cookie") {
                let cookie = cookie.to_str().map_err(|_| invalid())?;
                if let Some(value) = cookie.split(';').next().and_then(|c| c.strip_prefix("serviceToken=")) {
                    if token.is_some() || !crate::cookie_value(value) { return Err(invalid()); }
                    token = Some(SecretBytes::new(value.as_bytes().to_vec()));
                }
            }
        }
        let mut body = zeroize::Zeroizing::new(Vec::new());
        while let Some(chunk) = response.chunk().await.map_err(|_| failed())? {
            if body.len() + chunk.len() > limit { return Err(invalid()); }
            body.extend_from_slice(&chunk);
        }
        Ok(CloudResponse { body: SecretBytes::new(std::mem::take(&mut *body)), service_token: token })
    }).await
}

#[cfg(test)]
mod tests;
