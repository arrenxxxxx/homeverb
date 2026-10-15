//! Operation-scoped HTTPS with a separate, transmission-tracked control path.

use crate::{MAX_RESPONSE_BYTES, codec, invalid};
use async_trait::async_trait;
use homeverb_core::{Operation, SecretBytes, WriteProgress};
use homeverb_domain::{Error, Result};
use reqwest::{
    Client, StatusCode,
    header::{HeaderMap, HeaderValue},
};
use serde::Serialize;
use std::{fmt, time::Duration};
use zeroize::Zeroizing;

const ORIGIN: &str = "https://mp-prod.smartmidea.net/mas/v5/app/proxy?alias=";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudEndpoint {
    LoginId,
    Login,
    Homes,
    HomeAppliances,
    ApplianceState,
    ApplianceControl,
}

impl CloudEndpoint {
    pub fn path(self) -> &'static str {
        match self {
            Self::LoginId => "/v1/user/login/id/get",
            Self::Login => "/mj/user/login",
            Self::Homes => "/v1/homegroup/list/get",
            Self::HomeAppliances => "/v1/appliance/home/list/get",
            Self::ApplianceState => "/mjl/v1/device/status/lua/get",
            Self::ApplianceControl => "/mjl/v1/device/lua/control",
        }
    }
}

/// Native transport input, never a UI DTO. Fields are available to trusted transports only.
/// ```compile_fail
/// fn export(value: &homeverb_midea::SignedRequest) { let _ = serde_json::to_string(value); }
/// ```
pub struct SignedRequest {
    endpoint: CloudEndpoint,
    body: SecretBytes,
    random: String,
    signature: Zeroizing<String>,
    token: Option<SecretBytes>,
}

impl fmt::Debug for SignedRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SignedRequest([REDACTED])")
    }
}

impl SignedRequest {
    pub(crate) fn new(
        endpoint: CloudEndpoint,
        payload: &impl Serialize,
        token: Option<&SecretBytes>,
        random: String,
    ) -> Result<Self> {
        let body = SecretBytes::new(serde_json::to_vec(payload).map_err(|_| invalid())?);
        if body.expose().len() > MAX_RESPONSE_BYTES {
            return Err(invalid());
        }
        Ok(Self {
            endpoint,
            signature: codec::sign(body.expose(), &random),
            body,
            random,
            token: token.map(|value| SecretBytes::new(value.expose().to_vec())),
        })
    }

    pub fn endpoint(&self) -> CloudEndpoint {
        self.endpoint
    }
    pub fn body(&self) -> &[u8] {
        self.body.expose()
    }
    pub fn random(&self) -> &str {
        &self.random
    }
    pub fn signature(&self) -> &str {
        &self.signature
    }
    pub fn token(&self) -> Option<&[u8]> {
        self.token.as_ref().map(SecretBytes::expose)
    }
}

#[async_trait]
pub trait CloudTransport: Send + Sync {
    /// Send once within the supplied operation. Never log headers, bodies or provider errors.
    async fn send(&self, request: &SignedRequest, operation: &Operation) -> Result<SecretBytes>;

    /// Controls must never fall back to the read transport. Mark possible transmission
    /// immediately before sending, with no retries, redirects or implicit login.
    async fn send_control(
        &self,
        _: &SignedRequest,
        _: &WriteProgress,
        _: &Operation,
    ) -> Result<SecretBytes> {
        Err(control_unsupported())
    }
}

fn control_unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "The transport does not admit this control path.",
    )
}

/// No URL, proxy, certificate or redirect overrides are exposed in production.
#[derive(Default)]
pub struct MeijuHttps;

fn failed() -> Error {
    Error::new(
        "MIDEA_REQUEST_FAILED",
        "The cloud request could not be completed.",
    )
}

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

fn headers(request: &SignedRequest) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    headers.insert("secretversion", HeaderValue::from_static("1"));
    for (key, bytes) in [
        ("sign", request.signature().as_bytes()),
        ("random", request.random().as_bytes()),
    ] {
        let mut value = HeaderValue::from_bytes(bytes).map_err(|_| invalid())?;
        value.set_sensitive(true);
        headers.insert(key, value);
    }
    if let Some(token) = request.token() {
        let mut value = HeaderValue::from_bytes(token).map_err(|_| invalid())?;
        value.set_sensitive(true);
        headers.insert("accesstoken", value);
    }
    Ok(headers)
}

async fn send_once(
    client: Client,
    url: &str,
    request: &SignedRequest,
    operation: &Operation,
) -> Result<SecretBytes> {
    if request.endpoint == CloudEndpoint::ApplianceControl {
        return Err(control_unsupported());
    }
    transmit(client, url, request, None, operation).await
}

async fn transmit(
    client: Client,
    url: &str,
    request: &SignedRequest,
    progress: Option<&WriteProgress>,
    operation: &Operation,
) -> Result<SecretBytes> {
    operation
        .run(async {
            // TLS/HTTP libraries may make transient copies outside SecretBytes ownership.
            let outgoing = client
                .post(url)
                .headers(headers(request)?)
                .body(request.body().to_vec())
                .build()
                .map_err(|_| failed())?;
            operation.run(async { Ok(()) }).await?;
            // All synchronous preflight precedes this marker; connection errors after it
            // do not establish that no bytes reached the vendor.
            if let Some(progress) = progress {
                progress.mark_sent();
            }
            let mut response = client.execute(outgoing).await.map_err(|_| failed())?;
            match response.status() {
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    return Err(Error::new(
                        "AUTH_REQUIRED",
                        "The platform account requires user-side authorization.",
                    ));
                }
                status if !status.is_success() => return Err(failed()),
                _ => {}
            }
            if response
                .content_length()
                .is_some_and(|n| n > MAX_RESPONSE_BYTES as u64)
            {
                return Err(invalid());
            }
            let mut bytes = Zeroizing::new(Vec::new());
            while let Some(chunk) = response.chunk().await.map_err(|_| failed())? {
                if chunk.len() > MAX_RESPONSE_BYTES - bytes.len() {
                    return Err(invalid());
                }
                bytes.extend_from_slice(&chunk);
            }
            Ok(SecretBytes::new(std::mem::take(&mut *bytes)))
        })
        .await
}

#[async_trait]
impl CloudTransport for MeijuHttps {
    async fn send(&self, request: &SignedRequest, operation: &Operation) -> Result<SecretBytes> {
        operation
            .run(async {
                let client = client_builder().build().map_err(|_| failed())?;
                let url = format!("{ORIGIN}{}", request.endpoint.path());
                send_once(client, &url, request, operation).await
            })
            .await
    }

    async fn send_control(
        &self,
        request: &SignedRequest,
        progress: &WriteProgress,
        operation: &Operation,
    ) -> Result<SecretBytes> {
        if request.endpoint != CloudEndpoint::ApplianceControl {
            return Err(control_unsupported());
        }
        operation
            .run(async {
                let client = client_builder().build().map_err(|_| failed())?;
                let url = format!("{ORIGIN}{}", request.endpoint.path());
                transmit(client, &url, request, Some(progress), operation).await
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        task::JoinHandle,
    };
    use tokio_rustls::{
        TlsAcceptor,
        rustls::{ServerConfig, pki_types::PrivatePkcs8KeyDer},
    };
    use tokio_util::sync::CancellationToken;

    struct Server {
        url: String,
        client: Client,
        requests: Arc<Mutex<Vec<Vec<u8>>>>,
        connections: Arc<AtomicUsize>,
        closed: Arc<AtomicUsize>,
        task: JoinHandle<()>,
    }
    impl Drop for Server {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    impl Server {
        async fn start(response: Vec<u8>, stall: bool) -> Self {
            let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
            let cert = certificate.cert.der().clone();
            let key = PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der());
            let config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(vec![cert.clone()], key.into())
                .unwrap();
            let acceptor = TlsAcceptor::from(Arc::new(config));
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!(
                "https://localhost:{}/test",
                listener.local_addr().unwrap().port()
            );
            let response = String::from_utf8(response)
                .unwrap()
                .replace("SERVER_URL", &url)
                .into_bytes();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let connections = Arc::new(AtomicUsize::new(0));
            let closed = Arc::new(AtomicUsize::new(0));
            let (seen, connected, disconnected) =
                (requests.clone(), connections.clone(), closed.clone());
            let task = tokio::spawn(async move {
                loop {
                    let (socket, _) = listener.accept().await.unwrap();
                    connected.fetch_add(1, Ordering::SeqCst);
                    let Ok(mut stream) = acceptor.accept(socket).await else {
                        continue;
                    };
                    let mut bytes = Vec::new();
                    loop {
                        let mut buf = [0; 4096];
                        let n = stream.read(&mut buf).await.unwrap();
                        if n == 0 {
                            break;
                        }
                        bytes.extend_from_slice(&buf[..n]);
                        if let Some(end) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
                            let headers =
                                String::from_utf8_lossy(&bytes[..end]).to_ascii_lowercase();
                            let size: usize = headers
                                .lines()
                                .find_map(|s| s.strip_prefix("content-length: "))
                                .unwrap()
                                .parse()
                                .unwrap();
                            if bytes.len() >= end + 4 + size {
                                break;
                            }
                        }
                    }
                    seen.lock().unwrap().push(bytes);
                    if !response.is_empty() {
                        let _ = stream.write_all(&response).await;
                    }
                    if stall {
                        let mut byte = [0];
                        let _ = stream.read(&mut byte).await;
                        disconnected.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
            let client = client_builder()
                .add_root_certificate(reqwest::Certificate::from_der(cert.as_ref()).unwrap())
                .build()
                .unwrap();
            Self {
                url,
                client,
                requests,
                connections,
                closed,
                task,
            }
        }
    }

    fn request() -> SignedRequest {
        SignedRequest::new(
            CloudEndpoint::Homes,
            &serde_json::json!({"reqId":"synthetic"}),
            Some(&SecretBytes::new(b"FICTIONAL-TOKEN".to_vec())),
            "1792030210".into(),
        )
        .unwrap()
    }
    fn operation(ms: u64) -> Operation {
        Operation::new(ms, CancellationToken::new()).unwrap()
    }

    fn control_request() -> SignedRequest {
        SignedRequest::new(
            CloudEndpoint::ApplianceControl,
            &serde_json::json!({"applianceCode":"1","command":{"control":{"power":"off"}}}),
            Some(&SecretBytes::new(b"FICTIONAL-TOKEN".to_vec())),
            "1792030210".into(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn control_transport_requires_explicit_path_and_keeps_preflight_unsent() {
        let server = Server::start(vec![], false).await;
        let mut request = control_request();
        let progress = WriteProgress::default();
        assert_eq!(
            send_once(
                server.client.clone(),
                &server.url,
                &request,
                &operation(1000)
            )
            .await
            .unwrap_err()
            .code,
            "CAPABILITY_UNSUPPORTED"
        );
        assert_eq!(
            MeijuHttps
                .send_control(&super::tests::request(), &progress, &operation(1000))
                .await
                .unwrap_err()
                .code,
            "CAPABILITY_UNSUPPORTED"
        );
        let cancelled = operation(1000);
        cancelled.cancellation.cancel();
        assert_eq!(
            transmit(
                server.client.clone(),
                &server.url,
                &request,
                Some(&progress),
                &cancelled
            )
            .await
            .unwrap_err()
            .code,
            "CANCELLED"
        );
        request.token = Some(SecretBytes::new(b"invalid\nheader".to_vec()));
        assert_eq!(
            transmit(
                server.client.clone(),
                &server.url,
                &request,
                Some(&progress),
                &operation(1000)
            )
            .await
            .unwrap_err()
            .code,
            "MIDEA_RESPONSE_INVALID"
        );
        assert_eq!(progress.phase(), homeverb_domain::Phase::NotSent);
        assert_eq!(server.connections.load(Ordering::SeqCst), 0);

        struct ReadOnly;
        #[async_trait]
        impl CloudTransport for ReadOnly {
            async fn send(&self, _: &SignedRequest, _: &Operation) -> Result<SecretBytes> {
                panic!("Controls must not fall through to read transports.")
            }
        }
        assert_eq!(
            ReadOnly
                .send_control(&control_request(), &progress, &operation(1000))
                .await
                .unwrap_err()
                .code,
            "CAPABILITY_UNSUPPORTED"
        );
    }

    #[tokio::test]
    async fn control_http_replies_errors_and_disconnects_remain_possibly_sent() {
        for (wire, error) in [
            ("HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".into(), None),
            ("HTTP/1.1 302 Found\r\nLocation: SERVER_URL\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(), Some("MIDEA_REQUEST_FAILED")),
            ("HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(), Some("MIDEA_REQUEST_FAILED")),
            ("HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(), Some("AUTH_REQUIRED")),
            ("HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(), Some("AUTH_REQUIRED")),
            (format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", MAX_RESPONSE_BYTES + 1), Some("MIDEA_RESPONSE_INVALID")),
            ("HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\nshort".into(), Some("MIDEA_REQUEST_FAILED")),
        ] {
            let server = Server::start(wire.into_bytes(), false).await;
            let progress = WriteProgress::default();
            let result = transmit(server.client.clone(), &server.url, &control_request(), Some(&progress), &operation(2000)).await;
            assert_eq!(result.err().map(|e| e.code), error.map(str::to_owned));
            assert_eq!(progress.phase(), homeverb_domain::Phase::PossiblySent);
            assert_eq!(server.connections.load(Ordering::SeqCst), 1);
            assert_eq!(server.requests.lock().unwrap().len(), 1);
        }
    }

    #[tokio::test]
    async fn control_cancel_and_deadline_close_sockets_without_erasing_send_progress() {
        for cancel in [false, true] {
            let server = Server::start(
                b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\n".to_vec(),
                true,
            )
            .await;
            let progress = WriteProgress::default();
            let request = control_request();
            let op = operation(if cancel { 2000 } else { 200 });
            let work = transmit(
                server.client.clone(),
                &server.url,
                &request,
                Some(&progress),
                &op,
            );
            let stop = async {
                if cancel {
                    while server.requests.lock().unwrap().is_empty() {
                        tokio::task::yield_now().await;
                    }
                    op.cancellation.cancel();
                }
            };
            let (result, _) = tokio::join!(work, stop);
            assert_eq!(
                result.unwrap_err().code,
                if cancel {
                    "CANCELLED"
                } else {
                    "DEADLINE_EXCEEDED"
                }
            );
            assert_eq!(progress.phase(), homeverb_domain::Phase::PossiblySent);
            tokio::time::timeout(Duration::from_secs(1), async {
                while server.closed.load(Ordering::SeqCst) == 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            assert_eq!(server.connections.load(Ordering::SeqCst), 1);
        }
    }

    #[tokio::test]
    async fn sends_exact_signed_body_and_sensitive_headers_over_verified_tls() {
        let response =
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".to_vec();
        let server = Server::start(response, false).await;
        let request = request();
        let body = send_once(
            server.client.clone(),
            &server.url,
            &request,
            &operation(2000),
        )
        .await
        .unwrap();
        assert_eq!(body.expose(), b"{}");
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].ends_with(request.body()));
        let headers = headers(&request).unwrap();
        assert!(headers["accesstoken"].is_sensitive());
        assert!(headers["sign"].is_sensitive());
        assert_eq!(server.connections.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn rejects_redirect_http_auth_failure_and_oversized_bodies_without_retry() {
        let oversized = "x".repeat(MAX_RESPONSE_BYTES + 1);
        for (wire, code) in [
            ("HTTP/1.1 302 Found\r\nLocation: SERVER_URL\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned(), "MIDEA_REQUEST_FAILED"),
            ("HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned(), "MIDEA_REQUEST_FAILED"),
            ("HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned(), "AUTH_REQUIRED"),
            (format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", MAX_RESPONSE_BYTES + 1), "MIDEA_RESPONSE_INVALID"),
            (format!("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n{oversized}\r\n0\r\n\r\n", oversized.len()), "MIDEA_RESPONSE_INVALID"),
            ("HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\nshort".to_owned(), "MIDEA_REQUEST_FAILED"),
        ] {
            let server = Server::start(wire.into_bytes(), false).await;
            let error = send_once(server.client.clone(), &server.url, &request(), &operation(2000)).await.unwrap_err();
            assert_eq!(error.code, code);
            assert!(!format!("{error:?}").contains("localhost"));
            assert_eq!(server.connections.load(Ordering::SeqCst), 1);
            assert_eq!(server.requests.lock().unwrap().len(), 1);
        }
    }

    #[tokio::test]
    async fn rejects_untrusted_certificates_and_plaintext_and_honors_preflight() {
        let server = Server::start(vec![], false).await;
        let client = client_builder().build().unwrap();
        assert_eq!(
            send_once(client, &server.url, &request(), &operation(2000))
                .await
                .unwrap_err()
                .code,
            "MIDEA_REQUEST_FAILED"
        );
        assert!(server.requests.lock().unwrap().is_empty());
        let before = server.connections.load(Ordering::SeqCst);
        assert!(
            send_once(
                server.client.clone(),
                &server.url.replace("https:", "http:"),
                &request(),
                &operation(1000)
            )
            .await
            .is_err()
        );
        let op = operation(1000);
        op.cancellation.cancel();
        assert_eq!(
            send_once(server.client.clone(), &server.url, &request(), &op)
                .await
                .unwrap_err()
                .code,
            "CANCELLED"
        );
        assert_eq!(server.connections.load(Ordering::SeqCst), before);
    }

    #[tokio::test]
    async fn cancellation_and_deadline_close_incomplete_response_sockets() {
        for cancel in [false, true] {
            let server = Server::start(
                b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\n\r\n".to_vec(),
                true,
            )
            .await;
            let op = operation(if cancel { 2000 } else { 200 });
            let request = request();
            let work = send_once(server.client.clone(), &server.url, &request, &op);
            let stop = async {
                if cancel {
                    while server.requests.lock().unwrap().is_empty() {
                        tokio::task::yield_now().await;
                    }
                    op.cancellation.cancel();
                }
            };
            let (result, _) = tokio::join!(work, stop);
            assert_eq!(
                result.unwrap_err().code,
                if cancel {
                    "CANCELLED"
                } else {
                    "DEADLINE_EXCEEDED"
                }
            );
            tokio::time::timeout(Duration::from_secs(1), async {
                while server.closed.load(Ordering::SeqCst) == 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            assert_eq!(server.connections.load(Ordering::SeqCst), 1);
        }
    }
}
