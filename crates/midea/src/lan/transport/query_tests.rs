use super::*;
use aes::{
    Aes256,
    cipher::{BlockModeDecrypt, KeyIvInit, block_padding::NoPadding},
};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    net::Ipv4Addr,
    sync::atomic::{AtomicI64, Ordering},
    time::Duration,
};
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use tokio_util::sync::CancellationToken;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/lan-ac-read.json")).unwrap()
}

fn v3() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/lan-v3.json")).unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn credentials() -> V3Handshake {
    V3Handshake::new(
        SecretBytes::new(bytes(&v3()["token"])),
        SecretBytes::new(bytes(&v3()["key"])),
    )
    .unwrap()
}

fn operation() -> Operation {
    Operation::new(30_000, CancellationToken::new()).unwrap()
}

struct TestClock {
    start: DateTime<Utc>,
    calls: AtomicI64,
}

impl TestClock {
    fn new(start: DateTime<Utc>) -> Self {
        Self {
            start,
            calls: AtomicI64::new(0),
        }
    }
}

impl Clock for TestClock {
    fn now(&self) -> DateTime<Utc> {
        self.start + chrono::Duration::seconds(self.calls.fetch_add(1, Ordering::Relaxed))
    }
}

async fn query(endpoint: SocketAddrV4, operation: &Operation) -> Result<AcC0Status> {
    let fixture = fixture();
    query_ac_v3(
        &DiscoveryNetwork::loopback(),
        endpoint,
        credentials(),
        &CloudId(fixture["appliance_id"].as_str().unwrap().into()),
        &TestClock::new(fixture["timestamp"].as_str().unwrap().parse().unwrap()),
        fixture["sequence"].as_u64().unwrap() as u8,
        operation,
    )
    .await
}

async fn read_request(stream: &mut TcpStream, expected: &str, sequence: u16) {
    let mut header = [0; 6];
    stream.read_exact(&mut header).await.unwrap();
    assert_eq!(&header[..2], &[0x83, 0x70]);
    assert_eq!(header[4], 0x20);
    assert_eq!(header[5] & 15, 6);
    let size = usize::from(u16::from_be_bytes([header[2], header[3]])) + 8;
    assert!(size <= 4096);
    let mut body = vec![0; size - 6];
    stream.read_exact(&mut body).await.unwrap();
    let end = body.len() - 32;
    let mut plaintext = body[..end].to_vec();
    cbc::Decryptor::<Aes256>::new_from_slices(&bytes(&v3()["session_key"]), &[0; 16])
        .unwrap()
        .decrypt_padded::<NoPadding>(&mut plaintext)
        .unwrap();
    let mut digest = Sha256::new();
    digest.update(header);
    digest.update(&plaintext);
    assert_eq!(digest.finalize().as_slice(), &body[end..]);
    assert_eq!(u16::from_be_bytes([plaintext[0], plaintext[1]]), sequence);
    let end = plaintext.len() - usize::from(header[5] >> 4);
    assert_eq!(&plaintext[2..end], bytes(&fixture()[expected]));
}

async fn closed(stream: &mut TcpStream) {
    let result = stream.read(&mut [0; 1]).await;
    assert!(
        matches!(result, Ok(0))
            || matches!(result, Err(ref e) if e.kind() == std::io::ErrorKind::ConnectionReset)
    );
}

async fn fragments(stream: &mut TcpStream, response: &[u8]) {
    for part in response.chunks(3) {
        stream.write_all(part).await.unwrap();
        tokio::task::yield_now().await;
    }
}

enum Reply {
    Success,
    Failure { stage: u8, response: &'static str },
    Stall(u8),
}

struct Peer {
    endpoint: SocketAddrV4,
    ready: oneshot::Receiver<()>,
    task: JoinHandle<()>,
}

impl Peer {
    async fn start(reply: Reply) -> Self {
        let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 2), 0))
            .await
            .unwrap();
        let endpoint = match listener.local_addr().unwrap() {
            std::net::SocketAddr::V4(endpoint) => endpoint,
            _ => unreachable!(),
        };
        let (ready, received) = oneshot::channel();
        let task = tokio::spawn(async move {
            let (mut stream, address) = listener.accept().await.unwrap();
            assert_eq!(address.ip(), Ipv4Addr::LOCALHOST);
            let mut handshake = [0; 72];
            stream.read_exact(&mut handshake).await.unwrap();
            assert_eq!(handshake.as_slice(), bytes(&v3()["request"]));
            fragments(&mut stream, &bytes(&v3()["response"])).await;
            let mut ready = Some(ready);
            for (stage, (request, response)) in [
                ("appliance_request_packet", "appliance"),
                ("status_request_packet", "status"),
            ]
            .into_iter()
            .enumerate()
            {
                read_request(&mut stream, request, stage as u16).await;
                if matches!(reply, Reply::Stall(expected) if expected == stage as u8) {
                    let _ = ready.take().unwrap().send(());
                    closed(&mut stream).await;
                    assert!(
                        tokio::time::timeout(Duration::from_millis(25), listener.accept())
                            .await
                            .is_err()
                    );
                    return;
                }
                if let Reply::Failure {
                    stage: expected,
                    response,
                } = reply
                    && expected == stage as u8
                {
                    fragments(&mut stream, &bytes(&fixture()["responses"][response])).await;
                    closed(&mut stream).await;
                    assert!(
                        tokio::time::timeout(Duration::from_millis(25), listener.accept())
                            .await
                            .is_err()
                    );
                    return;
                }
                fragments(&mut stream, &bytes(&fixture()["responses"][response])).await;
            }
            closed(&mut stream).await;
            assert!(
                tokio::time::timeout(Duration::from_millis(25), listener.accept())
                    .await
                    .is_err()
            );
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
async fn negotiated_protocol_precedes_status_on_one_connection_without_freshness_claims() {
    let peer = Peer::start(Reply::Success).await;
    let state = query(peer.endpoint, &operation()).await.unwrap();
    assert_eq!(
        json!({"power":state.power,"mode_code":state.mode_code,"fan_speed_code":state.fan_speed_code,
        "target_celsius":state.target_celsius,"indoor_celsius":state.indoor_celsius,"outdoor_celsius":state.outdoor_celsius}),
        fixture()["expected"]
    );
    assert_eq!(format!("{state:?}"), "AcC0Status([REDACTED])");
    peer.finish().await;
}

#[tokio::test]
async fn invalid_negotiation_or_status_stops_without_partial_result_or_more_queries() {
    for (stage, response, code) in [
        (0, "wrong-appliance-target", "DEVICE_IDENTITY_CONFLICT"),
        (0, "wrong-appliance-kind", "CAPABILITY_UNSUPPORTED"),
        (0, "bad-appliance-checksum", "MIDEA_RESPONSE_INVALID"),
        (0, "bad-v3-digest", "MIDEA_RESPONSE_INVALID"),
        (1, "wrong-status-target", "DEVICE_IDENTITY_CONFLICT"),
        (1, "wrong-status-version", "MIDEA_RESPONSE_INVALID"),
        (1, "wrong-status-kind", "CAPABILITY_UNSUPPORTED"),
        (1, "bad-status-checksum", "MIDEA_RESPONSE_INVALID"),
    ] {
        let peer = Peer::start(Reply::Failure { stage, response }).await;
        let error = query(peer.endpoint, &operation()).await.unwrap_err();
        assert_eq!(error.code, code, "{response}");
        assert!(!error.message.contains("127.0.0"));
        peer.finish().await;
    }
}

#[tokio::test]
async fn cancellation_deadline_and_drop_close_either_query_stage_without_reconnect() {
    for stage in 0..2 {
        for outcome in ["cancel", "deadline", "drop"] {
            let mut peer = Peer::start(Reply::Stall(stage)).await;
            let operation = operation();
            let mut pending = Box::pin(query(peer.endpoint, &operation));
            tokio::select! {
                biased;
                result = &mut pending => panic!("Unexpected completion: {result:?}"),
                result = &mut peer.ready => result.unwrap(),
            }
            match outcome {
                "cancel" => {
                    operation.cancellation.cancel();
                    assert_eq!(pending.await.unwrap_err().code, "CANCELLED");
                }
                "deadline" => {
                    tokio::time::pause();
                    tokio::time::advance(Duration::from_secs(30)).await;
                    assert_eq!(pending.await.unwrap_err().code, "DEADLINE_EXCEEDED");
                    tokio::time::resume();
                }
                "drop" => drop(pending),
                _ => unreachable!(),
            }
            peer.finish().await;
        }
    }
}

#[tokio::test]
async fn invalid_inputs_and_stopped_operations_do_not_open_a_connection() {
    let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 2), 0))
        .await
        .unwrap();
    let endpoint = match listener.local_addr().unwrap() {
        std::net::SocketAddr::V4(endpoint) => endpoint,
        _ => unreachable!(),
    };
    for case in [
        "sequence",
        "target",
        "timestamp",
        "cancelled",
        "expired",
        "subnet",
    ] {
        let mut target = CloudId(fixture()["appliance_id"].as_str().unwrap().into());
        let mut timestamp = fixture()["timestamp"].as_str().unwrap().parse().unwrap();
        let mut sequence = 253;
        let mut selected = endpoint;
        let mut operation = operation();
        let code = match case {
            "sequence" => {
                sequence = 0;
                "INVALID_ARGUMENT"
            }
            "target" => {
                target = CloudId("281474976710656".into());
                "INVALID_ARGUMENT"
            }
            "timestamp" => {
                timestamp = "0000-01-01T00:00:00Z".parse().unwrap();
                "INVALID_ARGUMENT"
            }
            "cancelled" => {
                operation.cancellation.cancel();
                "CANCELLED"
            }
            "expired" => {
                operation.deadline = tokio::time::Instant::now();
                "DEADLINE_EXCEEDED"
            }
            "subnet" => {
                selected.set_ip(Ipv4Addr::new(127, 0, 1, 2));
                "INVALID_ARGUMENT"
            }
            _ => unreachable!(),
        };
        let error = query_ac_v3(
            &DiscoveryNetwork::loopback(),
            selected,
            credentials(),
            &target,
            &TestClock::new(timestamp),
            sequence,
            &operation,
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, code, "{case}");
        assert!(
            tokio::time::timeout(Duration::from_millis(25), listener.accept())
                .await
                .is_err()
        );
    }
}
