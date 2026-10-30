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
        let mut response = response;
        if let Some(index) = response
            .windows(10)
            .position(|value| value == b"SERVER_URL")
        {
            response.splice(index..index + 10, url.bytes());
        }
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

#[test]
fn captcha_origin_and_image_allocation_limits_are_closed() {
    assert!(captcha::captcha_url("/pass/getCode?token=fictional").is_ok());
    assert!(captcha::captcha_url("https://account.xiaomi.com/captcha.png").is_ok());
    for value in [
        "//foreign.invalid/img",
        "http://account.xiaomi.com/img",
        "https://account.xiaomi.com.evil.invalid/img",
        "https://user@account.xiaomi.com/img",
        "https://account.xiaomi.com:8443/img",
        "https://account.xiaomi.com/img#private",
        "https://127.0.0.1/img",
        " /pass/getCode",
        "/pass/\\evil",
    ] {
        assert!(captcha::captcha_url(value).is_err(), "{value}");
    }
    let image = captcha_png();
    assert_eq!(
        CaptchaResponse::new(
            SecretBytes::new(image.clone()),
            SecretBytes::new(b"fictional-ick".to_vec())
        )
        .unwrap()
        .media_type(),
        "image/png"
    );
    for body in [
        b"<html>private</html>".to_vec(),
        vec![0; captcha::MAX_IMAGE_BYTES + 1],
        {
            let mut image = image.clone();
            image[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
            image
        },
        image[..20].to_vec(),
    ] {
        assert!(
            CaptchaResponse::new(
                SecretBytes::new(body),
                SecretBytes::new(b"fictional-ick".to_vec())
            )
            .is_err()
        );
    }
    assert!(
        CaptchaResponse::new(
            SecretBytes::new(image),
            SecretBytes::new(b"injected;cookie".to_vec())
        )
        .is_err()
    );
    let jpeg = vec![0xff, 0xd8, 0xff, 0xc0, 0, 11, 8, 0, 1, 0, 1, 1, 1, 0x11, 0];
    assert_eq!(
        CaptchaResponse::new(
            SecretBytes::new(jpeg),
            SecretBytes::new(b"fictional-ick".to_vec())
        )
        .unwrap()
        .media_type(),
        "image/jpeg"
    );
}

fn captcha_png() -> Vec<u8> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jP1sAAAAASUVORK5CYII=").unwrap()
}
fn captcha_request() -> CaptchaRequest {
    CaptchaRequest::new(
        PrivateString::new("/pass/getCode?token=fictional".into()),
        PrivateString::new("sdkVersion=3.8.6; deviceId=fictional".into()),
    )
    .unwrap()
}

#[tokio::test]
async fn captcha_tls_checks_cookie_type_size_and_redirect_without_cloud_credentials() {
    for mode in 0..6 {
        let body = captcha_png();
        let headers = match mode {
            1 => "Content-Type: image/png\r\n",
            2 => "Content-Type: image/png\r\nSet-Cookie: ick=one\r\nSet-Cookie: ick=two\r\n",
            3 => "Content-Type: text/html\r\nSet-Cookie: ick=fictional\r\n",
            _ => "Content-Type: image/png\r\nSet-Cookie: ick=fictional\r\n",
        };
        let status = if mode == 4 {
            "302 Found\r\nLocation: SERVER_URL"
        } else {
            "200 OK"
        };
        let length = if mode == 5 {
            captcha::MAX_IMAGE_BYTES + 1
        } else {
            body.len()
        };
        let mut reply = format!(
            "HTTP/1.1 {status}\r\n{headers}Content-Length: {length}\r\nConnection: close\r\n\r\n"
        )
        .into_bytes();
        reply.extend_from_slice(&body);
        let server = Server::start(reply, false).await;
        let result = captcha::transmit_captcha(
            server.client.clone(),
            Url::parse(&server.url).unwrap(),
            &captcha_request(),
            &operation(),
        )
        .await;
        assert_eq!(result.is_ok(), mode == 0);
        assert_eq!(server.connections.load(Ordering::SeqCst), 1);
        let requests = server.requests.lock().unwrap();
        let wire = String::from_utf8_lossy(&requests[0]);
        assert!(wire.starts_with("GET "));
        assert!(!wire.contains("serviceToken"));
        assert!(!wire.contains("miot-encrypt-algorithm"));
    }
}

#[tokio::test]
async fn captcha_cancellation_and_drop_close_waiting_socket() {
    for mode in 0..3 {
        let server = Server::start(vec![], true).await;
        let op = operation();
        if mode == 0 {
            op.cancellation.cancel();
        }
        let request = captcha_request();
        let mut pending = Box::pin(captcha::transmit_captcha(
            server.client.clone(),
            Url::parse(&server.url).unwrap(),
            &request,
            &op,
        ));
        if mode == 0 {
            assert_eq!(pending.await.unwrap_err().code, "CANCELLED");
            assert_eq!(server.connections.load(Ordering::SeqCst), 0);
            continue;
        }
        loop {
            tokio::select! {
                result = &mut pending => panic!("Unexpected result: {result:?}"),
                _ = tokio::time::sleep(Duration::from_millis(1)) => { if !server.requests.lock().unwrap().is_empty() { break; } }
            }
        }
        if mode == 1 {
            op.cancellation.cancel();
            assert_eq!(pending.await.unwrap_err().code, "CANCELLED");
        } else {
            drop(pending);
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

fn verification_request(endpoint: VerificationEndpoint) -> VerificationRequest {
    let location = match endpoint {
        VerificationEndpoint::Password => "https://account.xiaomi.com/pass/serviceLoginAuth2",
        VerificationEndpoint::PhoneInfo => {
            "https://account.xiaomi.com/identity/auth/verifyPhone?_flag=4&_json=true"
        }
        VerificationEndpoint::EmailInfo => {
            "https://account.xiaomi.com/identity/auth/verifyEmail?_flag=8&_json=true"
        }
        VerificationEndpoint::Quota => "https://account.xiaomi.com/identity/pass/sms/userQuota",
        VerificationEndpoint::SendPhone => {
            "https://account.xiaomi.com/identity/auth/sendPhoneTicket"
        }
        VerificationEndpoint::SendEmail => {
            "https://account.xiaomi.com/identity/auth/sendEmailTicket?_dc=1234567890000"
        }
        VerificationEndpoint::Options => {
            "https://account.xiaomi.com/identity/list?context=fictional"
        }
        VerificationEndpoint::Phone => "https://account.xiaomi.com/identity/auth/verifyPhone",
        VerificationEndpoint::Email => "https://account.xiaomi.com/identity/auth/verifyEmail",
        VerificationEndpoint::Completion => {
            "https://account.xiaomi.com/identity/result/check?id=fictional"
        }
        VerificationEndpoint::SigningKey => {
            "https://account.xiaomi.com/pass/serviceLogin?sid=xiaomiio&_json=true"
        }
        VerificationEndpoint::Exchange => "https://sts.api.io.mi.com/sts?ticket=fictional",
    };
    VerificationRequest {
        endpoint,
        location: PrivateString::new(location.into()),
        cookie: PrivateString::new(
            if endpoint == VerificationEndpoint::Exchange {
                ""
            } else {
                "identity_session=fictional"
            }
            .into(),
        ),
        body: SecretBytes::new(
            if matches!(
                endpoint,
                VerificationEndpoint::Phone | VerificationEndpoint::Email
            ) {
                b"_flag=4&ticket=fictional&trust=false&_json=true".to_vec()
            } else {
                vec![]
            },
        ),
    }
}

#[test]
fn verification_requests_and_responses_keep_closed_origins_paths_and_bounds() {
    for endpoint in [
        VerificationEndpoint::Password,
        VerificationEndpoint::Options,
        VerificationEndpoint::PhoneInfo,
        VerificationEndpoint::EmailInfo,
        VerificationEndpoint::Quota,
        VerificationEndpoint::SendPhone,
        VerificationEndpoint::SendEmail,
        VerificationEndpoint::Phone,
        VerificationEndpoint::Email,
        VerificationEndpoint::Completion,
        VerificationEndpoint::SigningKey,
        VerificationEndpoint::Exchange,
    ] {
        assert!(verification_request(endpoint).url().is_ok());
        let mut r = verification_request(endpoint);
        r.location = PrivateString::new("https://foreign.invalid/identity/list".into());
        assert!(r.url().is_err());
    }
    for location in [
        "http://account.xiaomi.com/identity/list",
        "https://account.xiaomi.com:444/identity/list",
        "https://account.xiaomi.com/identity/list#fragment",
        "https://user@account.xiaomi.com/identity/list",
        "https://account.xiaomi.com/identity/../identity/list",
        "https://account.xiaomi.com/identity%2flist",
        "https://account.xiaomi.com/other",
    ] {
        let mut r = verification_request(VerificationEndpoint::Options);
        r.location = PrivateString::new(location.into());
        assert!(r.url().is_err());
    }
    let mut r = verification_request(VerificationEndpoint::SigningKey);
    r.location = PrivateString::new(
        "https://account.xiaomi.com/pass/serviceLogin?sid=mijia&_json=true".into(),
    );
    assert!(r.url().is_err());
    let mut r = verification_request(VerificationEndpoint::Exchange);
    r.cookie = PrivateString::new("passToken=fictional".into());
    assert!(r.url().is_err());
    assert!(
        VerificationResponse::new(200, SecretBytes::new(vec![0; 65_537]), None, vec![]).is_err()
    );
    assert!(
        VerificationResponse::new(
            200,
            SecretBytes::new(vec![]),
            None,
            (0..17)
                .map(|_| SecretBytes::new(b"fictional=x".to_vec()))
                .collect()
        )
        .is_err()
    );
    assert!(
        VerificationResponse::new(
            200,
            SecretBytes::new(vec![]),
            Some(SecretBytes::new(vec![b'x'; 8193])),
            vec![]
        )
        .is_err()
    );
}

#[tokio::test]
async fn verification_tls_is_one_send_without_automatic_redirect_or_cookie_jar() {
    for endpoint in [
        VerificationEndpoint::Options,
        VerificationEndpoint::PhoneInfo,
        VerificationEndpoint::EmailInfo,
        VerificationEndpoint::Quota,
        VerificationEndpoint::SendPhone,
        VerificationEndpoint::SendEmail,
        VerificationEndpoint::Phone,
        VerificationEndpoint::Exchange,
    ] {
        for redirected in [false, true] {
            let reply = if redirected {
                "HTTP/1.1 302 Found\r\nLocation: SERVER_URL\r\nSet-Cookie: identity_session=fictional; Path=/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            } else {
                "HTTP/1.1 200 OK\r\nSet-Cookie: identity_session=fictional; Path=/\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}"
            };
            let server = Server::start(reply.as_bytes().to_vec(), false).await;
            let r = verification_request(endpoint);
            let response = verification::transmit_verification(
                server.client.clone(),
                Url::parse(&server.url).unwrap(),
                &r,
                &operation(),
            )
            .await
            .unwrap();
            assert_eq!(response.status, if redirected { 302 } else { 200 });
            assert_eq!(response.location.is_some(), redirected);
            assert_eq!(response.cookies.len(), 1);
            assert_eq!(server.connections.load(Ordering::SeqCst), 1);
            let requests = server.requests.lock().unwrap();
            assert_eq!(requests.len(), 1);
            let wire = String::from_utf8_lossy(&requests[0]);
            assert_eq!(
                wire.starts_with("POST "),
                matches!(
                    endpoint,
                    VerificationEndpoint::Phone
                        | VerificationEndpoint::Quota
                        | VerificationEndpoint::SendPhone
                        | VerificationEndpoint::SendEmail
                )
            );
            assert_eq!(
                wire.contains("cookie: identity_session=fictional"),
                endpoint != VerificationEndpoint::Exchange
            );
            assert!(!wire.contains("miot-encrypt-algorithm"));
        }
    }
}

#[tokio::test]
async fn verification_tls_refuses_status_duplicate_locations_and_oversize_responses() {
    for reply in [
        "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        "HTTP/1.1 500 Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        "HTTP/1.1 200 OK\r\nContent-Length: 65537\r\nConnection: close\r\n\r\n",
        "HTTP/1.1 302 Found\r\nLocation: /one\r\nLocation: /two\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    ] {
        let server = Server::start(reply.as_bytes().to_vec(), false).await;
        let r = verification_request(VerificationEndpoint::Options);
        assert!(
            verification::transmit_verification(
                server.client.clone(),
                Url::parse(&server.url).unwrap(),
                &r,
                &operation()
            )
            .await
            .is_err()
        );
        assert_eq!(server.connections.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn verification_tls_cancel_timeout_and_drop_close_without_replay() {
    for mode in 0..4 {
        let server = Server::start(vec![], true).await;
        let op =
            Operation::new(if mode == 2 { 150 } else { 5000 }, CancellationToken::new()).unwrap();
        if mode == 0 {
            op.cancellation.cancel();
        }
        let r = verification_request(VerificationEndpoint::Phone);
        let mut work = Box::pin(verification::transmit_verification(
            server.client.clone(),
            Url::parse(&server.url).unwrap(),
            &r,
            &op,
        ));
        if mode == 0 {
            assert_eq!(work.await.unwrap_err().code, "CANCELLED");
            assert_eq!(server.connections.load(Ordering::SeqCst), 0);
            continue;
        }
        loop {
            tokio::select! {result=&mut work=>panic!("Unexpected completion {result:?}"),_=tokio::time::sleep(Duration::from_millis(2))=>()}
            if !server.requests.lock().unwrap().is_empty() {
                break;
            }
        }
        if mode == 1 {
            op.cancellation.cancel();
            assert_eq!(work.await.unwrap_err().code, "CANCELLED");
        } else if mode == 2 {
            assert_eq!(work.await.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            drop(work);
        }
        tokio::time::timeout(Duration::from_secs(1), async {
            while server.closed.load(Ordering::SeqCst) == 0 {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(server.connections.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn statistics_tls_uses_fixed_legacy_route_and_encryption_headers_once() {
    assert_eq!(
        request(CloudEndpoint::Statistics).url().unwrap().as_str(),
        "https://api.io.mi.com/app/v2/user/statistics"
    );
    let server = Server::start(
        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".to_vec(),
        false,
    )
    .await;
    transmit(
        server.client.clone(),
        Url::parse(&server.url).unwrap(),
        &request(CloudEndpoint::Statistics),
        None,
        &operation(),
    )
    .await
    .unwrap();
    {
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        let wire = String::from_utf8_lossy(&requests[0]);
        assert!(wire.starts_with("POST "));
        assert!(wire.contains("miot-encrypt-algorithm: ENCRYPT-RC4"));
        assert!(wire.contains("x-xiaomi-protocal-flag-cli: PROTOCAL-HTTP2"));
        assert!(wire.ends_with("data=fictional"));
    }
    assert_eq!(
        MijiaHttps
            .send_control(
                &request(CloudEndpoint::Statistics),
                &WriteProgress::default(),
                &operation()
            )
            .await
            .unwrap_err()
            .code,
        "CAPABILITY_UNSUPPORTED"
    );
}

fn modern_request(endpoint: crate::ModernAuthEndpoint) -> crate::ModernAuthRequest {
    use crate::ModernAuthEndpoint as E;
    let url = match endpoint {
        E::Bootstrap => {
            "https://account.xiaomi.com/pass/serviceLogin?_json=true&sid=mijia&_locale=zh_CN"
        }
        E::Qr => {
            "https://account.xiaomi.com/longPolling/loginUrl?sid=mijia&callback=https%3A%2F%2Fsts.api.mijia.tech%2Fmijia%2Fsts"
        }
        E::Poll => "https://c3.lp.account.xiaomi.com/lp/s?k=fictional-poll",
        E::Exchange => "https://sts.api.mijia.tech/mijia/sts?ticket=fictional-sts",
    };
    crate::ModernAuthRequest::new(
        endpoint,
        url.into(),
        PrivateString::new(if endpoint == E::Bootstrap {
            "passToken=fictional-pass-token".into()
        } else {
            String::new()
        }),
        PrivateString::new("fictional-modern-agent".into()),
    )
    .unwrap()
}

#[tokio::test]
async fn modern_auth_tls_isolated_cookies_bounded_responses_and_no_redirect_retry() {
    use crate::ModernAuthEndpoint as E;
    for endpoint in [E::Bootstrap, E::Qr, E::Poll, E::Exchange] {
        for mode in 0..8 {
            let reply = match mode {
                0 => "HTTP/1.1 200 OK\r\nSet-Cookie: serviceToken=fictional-service\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".to_owned(),
                1 => "HTTP/1.1 302 Found\r\nLocation: SERVER_URL\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned(),
                2 => "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned(),
                3 => "HTTP/1.1 200 OK\r\nContent-Length: 65537\r\nConnection: close\r\n\r\n".to_owned(),
                4 => "HTTP/1.1 200 OK\r\nSet-Cookie: serviceToken=one\r\nSet-Cookie: serviceToken=two\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".to_owned(),
                5 => format!("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{}", "x".repeat(65537)),
                7 => "HTTP/1.1 200 OK\r\nSet-Cookie: userId=12345\r\nSet-Cookie: userId=54321\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".to_owned(),
                _ => "HTTP/1.1 200 OK\r\nContent-Length: 20\r\nConnection: close\r\n\r\nshort".to_owned(),
            };
            let server = Server::start(reply.into_bytes(), false).await;
            let result = crate::modern::transport::transmit(
                server.client.clone(),
                Url::parse(&server.url).unwrap(),
                &modern_request(endpoint),
                &operation(),
            )
            .await;
            assert_eq!(
                result.is_ok(),
                mode == 0 || ((mode == 4 || mode == 7) && endpoint != E::Exchange)
            );
            if let Ok(result) = result {
                assert_eq!(result.service_token.is_some(), endpoint == E::Exchange);
                assert_eq!(format!("{result:?}"), "ModernAuthResponse([REDACTED])");
            }
            assert_eq!(server.connections.load(Ordering::SeqCst), 1);
            let requests = server.requests.lock().unwrap();
            assert_eq!(requests.len(), 1);
            let wire = String::from_utf8_lossy(&requests[0]);
            assert!(wire.starts_with("GET "));
            assert_eq!(wire.contains("cookie:"), endpoint == E::Bootstrap);
            assert_eq!(
                wire.contains("passToken=fictional"),
                endpoint == E::Bootstrap
            );
            assert!(!wire.contains("serviceToken="));
            assert!(!wire.contains("ENCRYPT-RC4"));
        }
    }
}

#[tokio::test]
async fn modern_auth_tls_cancellation_deadline_and_drop_close_socket() {
    for mode in 0..4 {
        let server = Server::start(vec![], true).await;
        let mut op = operation();
        if mode == 0 {
            op.cancellation.cancel();
        }
        if mode == 2 {
            op.deadline = tokio::time::Instant::now() + Duration::from_millis(200);
        }
        let request = modern_request(crate::ModernAuthEndpoint::Poll);
        let mut pending = Box::pin(crate::modern::transport::transmit(
            server.client.clone(),
            Url::parse(&server.url).unwrap(),
            &request,
            &op,
        ));
        if mode == 0 {
            assert_eq!(pending.await.unwrap_err().code, "CANCELLED");
            assert_eq!(server.connections.load(Ordering::SeqCst), 0);
            continue;
        }
        loop {
            tokio::select! {
                result = &mut pending => panic!("Unexpected {result:?}"),
                _ = tokio::time::sleep(Duration::from_millis(1)) => { if !server.requests.lock().unwrap().is_empty() { break; } }
            }
        }
        if mode == 1 {
            op.cancellation.cancel();
            assert_eq!(pending.await.unwrap_err().code, "CANCELLED");
        } else if mode == 2 {
            assert_eq!(pending.await.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            drop(pending);
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
