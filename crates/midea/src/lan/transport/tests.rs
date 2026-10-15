use super::*;
use serde_json::Value;
use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use tokio_util::sync::CancellationToken;

fn fixture() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/lan-v3.json"
    )))
    .unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn credentials() -> V3Handshake {
    let f = fixture();
    V3Handshake::new(
        SecretBytes::new(bytes(&f["token"])),
        SecretBytes::new(bytes(&f["key"])),
    )
    .unwrap()
}

fn encrypted_frame(kind: u8) -> Vec<u8> {
    let f = fixture();
    let case = f["frames"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["size"] == 14 && v["type"] == kind)
        .unwrap();
    bytes(&case["frame"])
}

fn payload() -> SecretBytes {
    SecretBytes::new((0..14).collect())
}

fn operation() -> Operation {
    Operation::new(30_000, CancellationToken::new()).unwrap()
}

async fn closed(stream: &mut TcpStream) {
    // Peer::finish owns the watchdog after any deliberate clock advance.
    let read = stream.read(&mut [0; 1]).await;
    assert!(
        matches!(read, Ok(0))
            || matches!(read, Err(ref error) if error.kind() == std::io::ErrorKind::ConnectionReset)
    );
}

enum Reply {
    Success,
    HandshakeFailure(Vec<u8>),
    CommandFailure(Vec<u8>),
    Stall { after_command: bool },
}

struct Peer {
    endpoint: SocketAddrV4,
    ready: oneshot::Receiver<()>,
    task: JoinHandle<()>,
}

async fn send_fragments(stream: &mut TcpStream, bytes: &[u8]) {
    for part in bytes.chunks(3) {
        stream.write_all(part).await.unwrap();
        tokio::task::yield_now().await;
    }
}

impl Peer {
    async fn start(reply: Reply) -> Self {
        let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 2), 0))
            .await
            .unwrap();
        let endpoint = match listener.local_addr().unwrap() {
            std::net::SocketAddr::V4(address) => address,
            _ => unreachable!(),
        };
        let (ready, received) = oneshot::channel();
        let task = tokio::spawn(async move {
            let (mut stream, peer) = listener.accept().await.unwrap();
            assert_eq!(peer.ip(), Ipv4Addr::LOCALHOST);
            let mut request = [0; 72];
            stream.read_exact(&mut request).await.unwrap();
            assert_eq!(request.as_slice(), bytes(&fixture()["request"]));
            if let Reply::HandshakeFailure(response) = reply {
                stream.write_all(&response).await.unwrap();
                stream.shutdown().await.unwrap();
                closed(&mut stream).await;
                return;
            }
            if matches!(
                reply,
                Reply::Stall {
                    after_command: false
                }
            ) {
                let _ = ready.send(());
                closed(&mut stream).await;
                return;
            }
            send_fragments(&mut stream, &bytes(&fixture()["response"])).await;
            let mut request = [0; 54];
            stream.read_exact(&mut request).await.unwrap();
            assert_eq!(request.as_slice(), encrypted_frame(6));
            let _ = ready.send(());
            match reply {
                Reply::Success => send_fragments(&mut stream, &encrypted_frame(3)).await,
                Reply::CommandFailure(response) => {
                    stream.write_all(&response).await.unwrap();
                    stream.shutdown().await.unwrap();
                }
                Reply::Stall {
                    after_command: true,
                } => {}
                _ => unreachable!(),
            }
            closed(&mut stream).await;
        });
        Self {
            endpoint,
            ready: received,
            task,
        }
    }

    async fn finish(self) {
        tokio::time::timeout(Duration::from_secs(30), self.task)
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn preflight_rejects_targets_outside_the_selected_network() {
    let network = DiscoveryNetwork::loopback();
    let operation = Operation::new(30_000, CancellationToken::new()).unwrap();
    let progress = WriteProgress::default();
    let result = exchange_v3(
        &network,
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 1, 2), 6444),
        V3Handshake::new(SecretBytes::new(vec![0; 64]), SecretBytes::new(vec![0; 32])).unwrap(),
        &SecretBytes::new(vec![0; 14]),
        &progress,
        &operation,
    )
    .await;
    assert_eq!(result.unwrap_err().code, "INVALID_ARGUMENT");
    assert_eq!(progress.phase(), homeverb_domain::Phase::NotSent);
}

#[tokio::test]
async fn preflight_refuses_bad_payload_scope_progress_and_expired_operations() {
    let network = DiscoveryNetwork::loopback();
    for endpoint in [
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 0), 6444),
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 255), 6444),
        SocketAddrV4::new(Ipv4Addr::LOCALHOST, 6444),
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 2), 0),
    ] {
        let progress = WriteProgress::default();
        assert_eq!(
            exchange_v3(
                &network,
                endpoint,
                credentials(),
                &payload(),
                &progress,
                &operation()
            )
            .await
            .unwrap_err()
            .code,
            "INVALID_ARGUMENT"
        );
        assert_eq!(progress.phase(), Phase::NotSent);
    }
    let endpoint = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 2), 6444);
    for size in [0, 4047] {
        let progress = WriteProgress::default();
        assert_eq!(
            exchange_v3(
                &network,
                endpoint,
                credentials(),
                &SecretBytes::new(vec![0; size]),
                &progress,
                &operation()
            )
            .await
            .unwrap_err()
            .code,
            "INVALID_ARGUMENT"
        );
        assert_eq!(progress.phase(), Phase::NotSent);
    }
    let progress = WriteProgress::default();
    progress.mark_sent();
    assert_eq!(
        exchange_v3(
            &network,
            endpoint,
            credentials(),
            &payload(),
            &progress,
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "INVALID_ARGUMENT"
    );
    assert_eq!(progress.phase(), Phase::PossiblySent);
    for cancel in [false, true] {
        let mut operation = operation();
        if cancel {
            operation.cancellation.cancel();
        } else {
            operation.deadline = tokio::time::Instant::now();
        }
        let progress = WriteProgress::default();
        assert_eq!(
            exchange_v3(
                &network,
                endpoint,
                credentials(),
                &payload(),
                &progress,
                &operation
            )
            .await
            .unwrap_err()
            .code,
            if cancel {
                "CANCELLED"
            } else {
                "DEADLINE_EXCEEDED"
            }
        );
        assert_eq!(progress.phase(), Phase::NotSent);
    }
}

#[tokio::test]
async fn fragmented_success_is_one_exchange_and_not_acceptance_evidence() {
    let peer = Peer::start(Reply::Success).await;
    let progress = WriteProgress::default();
    let result = exchange_v3(
        &DiscoveryNetwork::loopback(),
        peer.endpoint,
        credentials(),
        &payload(),
        &progress,
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(result.sequence, 0);
    assert_eq!(result.payload.expose(), payload().expose());
    assert_eq!(progress.phase(), Phase::PossiblySent);
    peer.finish().await;
}

#[tokio::test]
async fn refused_connection_is_fixed_and_proven_before_command_send() {
    // Reserve an unused loopback TCP port without opening a listener.
    let reservation = TcpSocket::new_v4().unwrap();
    reservation
        .bind((Ipv4Addr::new(127, 0, 0, 2), 0).into())
        .unwrap();
    let endpoint = match reservation.local_addr().unwrap() {
        std::net::SocketAddr::V4(address) => address,
        _ => unreachable!(),
    };
    let progress = WriteProgress::default();
    let error = exchange_v3(
        &DiscoveryNetwork::loopback(),
        endpoint,
        credentials(),
        &payload(),
        &progress,
        &operation(),
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, "MIDEA_REQUEST_FAILED");
    assert_eq!(error.message, "The V3 LAN exchange could not be completed.");
    assert_eq!(progress.phase(), Phase::NotSent);
}

#[tokio::test]
async fn handshake_failures_never_send_a_command() {
    let mut rejected = vec![0x83, 0x70, 0, 5, 0x20, 1, 0, 0];
    rejected.extend_from_slice(b"ERROR");
    let mut corrupted = bytes(&fixture()["response"]);
    corrupted[71] ^= 1;
    for (response, code) in [
        (rejected, "AUTH_REQUIRED"),
        (corrupted, "AUTH_REQUIRED"),
        (
            vec![0x83, 0x70, 0xff, 0xff, 0x20, 1],
            "MIDEA_RESPONSE_INVALID",
        ),
        (
            bytes(&fixture()["response"])[..10].to_vec(),
            "MIDEA_REQUEST_FAILED",
        ),
    ] {
        let peer = Peer::start(Reply::HandshakeFailure(response)).await;
        let progress = WriteProgress::default();
        let error = exchange_v3(
            &DiscoveryNetwork::loopback(),
            peer.endpoint,
            credentials(),
            &payload(),
            &progress,
            &operation(),
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(progress.phase(), Phase::NotSent);
        peer.finish().await;
    }
}

#[tokio::test]
async fn response_failures_preserve_possible_transmission_without_retries() {
    let mut corrupted = encrypted_frame(3);
    *corrupted.last_mut().unwrap() ^= 1;
    for (response, code) in [
        (vec![], "MIDEA_REQUEST_FAILED"),
        (encrypted_frame(3)[..10].to_vec(), "MIDEA_REQUEST_FAILED"),
        (corrupted, "MIDEA_RESPONSE_INVALID"),
        (
            vec![0x83, 0x70, 0xff, 0xff, 0x20, 3],
            "MIDEA_RESPONSE_INVALID",
        ),
    ] {
        let peer = Peer::start(Reply::CommandFailure(response)).await;
        let progress = WriteProgress::default();
        let error = exchange_v3(
            &DiscoveryNetwork::loopback(),
            peer.endpoint,
            credentials(),
            &payload(),
            &progress,
            &operation(),
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(progress.phase(), Phase::PossiblySent);
        peer.finish().await;
    }
}

#[tokio::test]
async fn cancellation_deadline_and_future_drop_close_before_and_after_command_send() {
    for after_command in [false, true] {
        for stop in ["cancel", "deadline", "drop"] {
            let mut peer = Peer::start(Reply::Stall { after_command }).await;
            let operation = operation();
            let cancellation = operation.cancellation.clone();
            let progress = WriteProgress::default();
            let observed = progress.clone();
            let endpoint = peer.endpoint;
            let task = tokio::spawn(async move {
                exchange_v3(
                    &DiscoveryNetwork::loopback(),
                    endpoint,
                    credentials(),
                    &payload(),
                    &progress,
                    &operation,
                )
                .await
            });
            tokio::time::timeout(Duration::from_secs(30), &mut peer.ready)
                .await
                .unwrap()
                .unwrap();
            match stop {
                "cancel" => cancellation.cancel(),
                "deadline" => {
                    // Pause only after real I/O so clock auto-advance cannot race setup.
                    tokio::time::pause();
                    tokio::time::advance(Duration::from_secs(31)).await;
                }
                _ => task.abort(),
            }
            let result = tokio::time::timeout(Duration::from_secs(30), task)
                .await
                .unwrap();
            if stop == "deadline" {
                tokio::time::resume();
            }
            match stop {
                "cancel" => assert_eq!(result.unwrap().unwrap_err().code, "CANCELLED"),
                "deadline" => assert_eq!(result.unwrap().unwrap_err().code, "DEADLINE_EXCEEDED"),
                _ => assert!(result.unwrap_err().is_cancelled()),
            }
            assert_eq!(
                observed.phase(),
                if after_command {
                    Phase::PossiblySent
                } else {
                    Phase::NotSent
                }
            );
            peer.finish().await;
        }
    }
}

#[tokio::test]
async fn exact_frame_reads_preserve_coalesced_frame_boundaries() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        stream
            .write_all(&[encrypted_frame(3), encrypted_frame(3)].concat())
            .await
            .unwrap();
        closed(&mut stream).await;
    });
    let mut stream = TcpStream::connect(address).await.unwrap();
    for _ in 0..2 {
        assert_eq!(
            read_frame(&mut stream, 3, &operation())
                .await
                .unwrap()
                .expose(),
            encrypted_frame(3)
        );
    }
    drop(stream);
    tokio::time::timeout(Duration::from_secs(30), server)
        .await
        .unwrap()
        .unwrap();
}
