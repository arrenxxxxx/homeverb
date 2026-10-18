use super::*;

fn request(endpoint: CloudEndpoint) -> CloudRequest {
    CloudRequest {
        endpoint,
        region: Region::Cn,
        location: None,
        cookie: PrivateString::new("serviceToken=FICTIONAL-TOKEN".into()),
        body: SecretBytes::new(b"data=fictional".to_vec()),
    }
}
fn operation() -> Operation {
    Operation::new(5000, CancellationToken::new()).unwrap()
}

#[tokio::test]
async fn production_read_and_write_transports_refuse_wrong_endpoints_before_send() {
    let progress = WriteProgress::default();
    assert_eq!(
        MijiaHttps
            .send(&request(CloudEndpoint::SetProperty), &operation())
            .await
            .unwrap_err()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(
        MijiaHttps
            .send_control(&request(CloudEndpoint::Properties), &progress, &operation())
            .await
            .unwrap_err()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(progress.phase(), homeverb_domain::Phase::NotSent);
    assert_eq!(
        request(CloudEndpoint::SetProperty).url().unwrap().as_str(),
        "https://api.io.mi.com/app/miotspec/prop/set"
    );
}

#[tokio::test]
async fn write_transport_marks_send_without_interpreting_http_acceptance() {
    for reply in [
        "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
        "HTTP/1.1 302 Found\r\nLocation: SERVER_URL\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        "HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        "HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nshort",
    ] {
        let server = Server::start(reply.as_bytes().to_vec(), false).await;
        let progress = WriteProgress::default();
        let result = transmit(
            server.client.clone(),
            Url::parse(&server.url).unwrap(),
            &request(CloudEndpoint::SetProperty),
            Some(&progress),
            &operation(),
        )
        .await;
        assert_eq!(result.is_ok(), reply.ends_with("{}"));
        assert_eq!(progress.phase(), homeverb_domain::Phase::PossiblySent);
        assert_eq!(server.connections.load(Ordering::SeqCst), 1);
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        let wire = String::from_utf8_lossy(&requests[0]);
        assert!(wire.starts_with("POST "));
        assert!(wire.contains("miot-encrypt-algorithm: ENCRYPT-RC4"));
        assert!(wire.ends_with("data=fictional"));
    }
}

#[tokio::test]
async fn write_preflight_failure_is_not_sent_but_cancel_deadline_and_drop_after_send_are_uncertain()
{
    for mode in 0..4 {
        let server = Server::start(vec![], true).await;
        let op = operation();
        let request = request(CloudEndpoint::SetProperty);
        let progress = WriteProgress::default();
        if mode == 0 {
            op.cancellation.cancel();
        }
        let mut op = op;
        if mode == 2 {
            op.deadline = tokio::time::Instant::now() + Duration::from_millis(200);
        }
        let mut send = Box::pin(transmit(
            server.client.clone(),
            Url::parse(&server.url).unwrap(),
            &request,
            Some(&progress),
            &op,
        ));
        if mode == 0 {
            assert_eq!(send.await.unwrap_err().code, "CANCELLED");
            assert_eq!(progress.phase(), homeverb_domain::Phase::NotSent);
            assert_eq!(server.connections.load(Ordering::SeqCst), 0);
            continue;
        }
        loop {
            tokio::select! {
                result = &mut send => panic!("Unexpected result: {result:?}"),
                _ = tokio::time::sleep(Duration::from_millis(1)) => { if !server.requests.lock().unwrap().is_empty() { break; } }
            }
        }
        if mode == 1 {
            op.cancellation.cancel();
            assert_eq!(send.await.unwrap_err().code, "CANCELLED");
        } else if mode == 2 {
            assert_eq!(send.await.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            drop(send);
        }
        assert_eq!(progress.phase(), homeverb_domain::Phase::PossiblySent);
        tokio::time::timeout(Duration::from_secs(2), async {
            while server.closed.load(Ordering::SeqCst) == 0 {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(server.connections.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn sts_allowlist_rejects_private_or_foreign_destinations() {
    assert!(exchange_url("https://sts.api.io.mi.com/sts?ticket=fictional").is_ok());
    for url in [
        "http://sts.api.io.mi.com/sts",
        "https://sts.api.io.mi.com.evil.invalid/sts",
        "https://user@sts.api.io.mi.com/sts",
        "https://sts.api.io.mi.com:8443/sts",
        "https://127.0.0.1/sts",
        "https://sts.api.io.mi.com/other",
        "https://sts.api.io.mi.com/sts#private",
    ] {
        assert!(exchange_url(url).is_err(), "{url}");
    }
    assert_eq!(
        request(CloudEndpoint::Devices).url().unwrap().as_str(),
        "https://api.io.mi.com/app/home/device_list"
    );
}

#[tokio::test]
async fn public_spec_requests_never_forward_account_cookies_or_encryption_headers() {
    let server = Server::start(
        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".to_vec(),
        false,
    )
    .await;
    let response = transmit(
        server.client.clone(),
        Url::parse(&server.url).unwrap(),
        &request(CloudEndpoint::SpecIndex),
        None,
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(response.body.expose(), b"{}");
    let requests = server.requests.lock().unwrap();
    let wire = String::from_utf8_lossy(&requests[0]);
    assert!(wire.starts_with("GET "));
    assert!(!wire.contains("cookie:"));
    assert!(!wire.contains("FICTIONAL-TOKEN"));
    assert!(!wire.contains("miot-encrypt-algorithm"));
}

#[tokio::test]
async fn tls_sends_exact_body_and_extracts_single_cookie() {
    let server = Server::start(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nSet-Cookie: serviceToken=fictional; Path=/; Secure\r\nConnection: close\r\n\r\n{}".to_vec(), false).await;
    let response = transmit(
        server.client.clone(),
        Url::parse(&server.url).unwrap(),
        &request(CloudEndpoint::LoginExchange),
        None,
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(response.service_token.unwrap().expose(), b"fictional");
    assert_eq!(response.body.expose(), b"{}");
    let requests = server.requests.lock().unwrap();
    let sent = std::str::from_utf8(&requests[0]).unwrap();
    assert!(sent.ends_with("data=fictional"));
    assert!(sent.contains("cookie: serviceToken=FICTIONAL-TOKEN"));
}

#[tokio::test]
async fn redirect_auth_large_body_and_duplicate_cookie_fail_once() {
    for (wire, expected) in [
        ("HTTP/1.1 302 Found\r\nLocation: SERVER_URL\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(), "MIJIA_REQUEST_FAILED"),
        ("HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(), "AUTH_REQUIRED"),
        ("HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into(), "AUTH_REQUIRED"),
        (format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", MAX_RESPONSE_BYTES+1), "MIJIA_INVALID_RESPONSE"),
        ("HTTP/1.1 200 OK\r\nContent-Length: 0\r\nSet-Cookie: serviceToken=one\r\nSet-Cookie: serviceToken=two\r\nConnection: close\r\n\r\n".into(), "MIJIA_INVALID_RESPONSE"),
        ("HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nshort".into(), "MIJIA_REQUEST_FAILED"),
    ] {
        let server = Server::start(wire.into_bytes(), false).await;
        let error = transmit(server.client.clone(), Url::parse(&server.url).unwrap(), &request(CloudEndpoint::LoginExchange), None, &operation()).await.unwrap_err();
        assert_eq!(error.code, expected);
        assert_eq!(server.connections.load(Ordering::SeqCst), 1);
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn untrusted_tls_and_cancelled_preflight_refuse() {
    let server = Server::start(vec![], false).await;
    let request = request(CloudEndpoint::Devices);
    let op = operation();
    op.cancellation.cancel();
    assert_eq!(
        transmit(
            server.client.clone(),
            Url::parse(&server.url).unwrap(),
            &request,
            None,
            &op
        )
        .await
        .unwrap_err()
        .code,
        "CANCELLED"
    );
    assert_eq!(server.connections.load(Ordering::SeqCst), 0);
    let client = client_builder().build().unwrap();
    assert_eq!(
        transmit(
            client,
            Url::parse(&server.url).unwrap(),
            &request,
            None,
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "MIJIA_REQUEST_FAILED"
    );
    assert!(server.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn cancellation_deadline_and_drop_close_pending_socket() {
    for mode in [0, 1, 2] {
        let server = Server::start(vec![], true).await;
        let op = operation();
        let req = request(CloudEndpoint::Devices);
        let mut operation_for_send = op.clone();
        if mode == 1 {
            operation_for_send.deadline = tokio::time::Instant::now() + Duration::from_millis(200);
        }
        let send = transmit(
            server.client.clone(),
            Url::parse(&server.url).unwrap(),
            &req,
            None,
            &operation_for_send,
        );
        let mut send = Box::pin(send);
        loop {
            tokio::select! {
                result = &mut send => panic!("Unexpected early result: {result:?}"),
                _ = tokio::time::sleep(Duration::from_millis(1)) => { if !server.requests.lock().unwrap().is_empty() { break; } }
            }
        }
        if mode == 0 {
            op.cancellation.cancel();
            assert_eq!(send.await.unwrap_err().code, "CANCELLED");
        } else if mode == 1 {
            assert_eq!(send.await.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            drop(send);
        }
        tokio::time::timeout(Duration::from_secs(2), async {
            while server.closed.load(Ordering::SeqCst) == 0 {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(server.connections.load(Ordering::SeqCst), 1);
    }
}
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
                        let headers = String::from_utf8_lossy(&bytes[..end]).to_ascii_lowercase();
                        let size: usize = headers
                            .lines()
                            .find_map(|s| s.strip_prefix("content-length: "))
                            .unwrap_or("0")
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
                    // Flush the final TLS record before dropping the fixture socket.
                    // Otherwise a short write can look like a truncated response.
                    let _ = stream.flush().await;
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
