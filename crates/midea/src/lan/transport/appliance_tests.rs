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
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/lan-appliances-read.json"
    ))
    .unwrap()
}
fn conversation(family: u16) -> Value {
    fixture()["conversations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["family"] == family)
        .unwrap()
        .clone()
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
    Operation::new(10_000, CancellationToken::new()).unwrap()
}

struct TestClock {
    calls: AtomicI64,
}
impl Clock for TestClock {
    fn now(&self) -> DateTime<Utc> {
        let start: DateTime<Utc> = fixture()["timestamp"].as_str().unwrap().parse().unwrap();
        start + chrono::Duration::seconds(self.calls.fetch_add(1, Ordering::Relaxed))
    }
}
async fn query(
    family: u16,
    endpoint: SocketAddrV4,
    clock: &dyn Clock,
    operation: &Operation,
) -> Result<Value> {
    let target = CloudId(fixture()["appliance_id"].as_str().unwrap().into());
    if family == 0xe1 {
        let status = query_dishwasher_v3(
            &DiscoveryNetwork::loopback(),
            endpoint,
            credentials(),
            &target,
            clock,
            operation,
        )
        .await?;
        Ok(json!({
            "status_code":status.status_code,"mode_code":status.mode_code,"additional_code":status.additional_code,
            "temperature_code":status.temperature_code,"door_open":status.door_open,
            "rinse_aid_low":status.rinse_aid_low,"salt_low":status.salt_low,"water_low":status.water_low,
            "remaining_minutes":status.remaining_minutes,
        }))
    } else {
        let status = query_refrigerator_v3(
            &DiscoveryNetwork::loopback(),
            endpoint,
            credentials(),
            &target,
            clock,
            operation,
        )
        .await?;
        Ok(json!({
            "mode_flags":status.mode_flags,"fridge_power":status.fridge_power,"freezer_power":status.freezer_power,
            "fridge_target_celsius":status.fridge_target_celsius,"freezer_target_celsius":status.freezer_target_celsius,
            "fridge_current_celsius":status.fridge_current_celsius,"freezer_current_celsius":status.freezer_current_celsius,
        }))
    }
}

async fn read_request(stream: &mut TcpStream, expected: &Value, sequence: u16) {
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
    assert_eq!(&plaintext[2..end], bytes(expected));
}
async fn closed(stream: &mut TcpStream) {
    let result = stream.read(&mut [0; 1]).await;
    assert!(
        matches!(result, Ok(0))
            || matches!(result, Err(ref e) if e.kind() == std::io::ErrorKind::ConnectionReset)
    );
}
async fn fragments(stream: &mut TcpStream, response: &Value) {
    for part in bytes(response).chunks(3) {
        stream.write_all(part).await.unwrap();
        tokio::task::yield_now().await;
    }
}

enum Reply {
    Success,
    Failure { stage: u8, name: String },
    Stall(u8),
}
struct Peer {
    endpoint: SocketAddrV4,
    ready: oneshot::Receiver<()>,
    task: JoinHandle<()>,
}
impl Peer {
    async fn start(family: u16, reply: Reply) -> Self {
        let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 2), 0))
            .await
            .unwrap();
        let endpoint = match listener.local_addr().unwrap() {
            std::net::SocketAddr::V4(endpoint) => endpoint,
            _ => unreachable!(),
        };
        let (ready, received) = oneshot::channel();
        let task = tokio::spawn(async move {
            let case = conversation(family);
            let (mut stream, address) = listener.accept().await.unwrap();
            assert_eq!(address.ip(), Ipv4Addr::LOCALHOST);
            let mut handshake = [0; 72];
            stream.read_exact(&mut handshake).await.unwrap();
            assert_eq!(handshake.as_slice(), bytes(&v3()["request"]));
            fragments(&mut stream, &v3()["response"]).await;
            let mut ready = Some(ready);
            for (stage, (request, response)) in [
                ("appliance_request_packet", "appliance"),
                ("status_request_packet", "status"),
            ]
            .into_iter()
            .enumerate()
            {
                read_request(&mut stream, &case[request], stage as u16).await;
                if matches!(reply, Reply::Stall(expected) if expected == stage as u8) {
                    let _ = ready.take().unwrap().send(());
                    break;
                }
                if let Reply::Failure {
                    stage: expected,
                    ref name,
                } = reply
                    && expected == stage as u8
                {
                    fragments(&mut stream, &case["responses"][name]).await;
                    break;
                }
                fragments(&mut stream, &case["responses"][response]).await;
            }
            closed(&mut stream).await;
            assert!(
                tokio::time::timeout(Duration::from_millis(30), listener.accept())
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
        tokio::time::timeout(Duration::from_secs(2), self.task)
            .await
            .unwrap()
            .unwrap();
    }
}

#[tokio::test]
async fn both_families_negotiate_then_query_once_with_fresh_packet_timestamps() {
    for family in [0xe1, 0xca] {
        let peer = Peer::start(family, Reply::Success).await;
        let clock = TestClock {
            calls: AtomicI64::new(0),
        };
        let result = query(family, peer.endpoint, &clock, &operation())
            .await
            .unwrap();
        assert_eq!(result, conversation(family)["expected"]);
        assert_eq!(clock.calls.load(Ordering::Relaxed), 2);
        peer.finish().await;
    }
}

#[tokio::test]
async fn wrong_target_family_kind_version_or_payload_stops_without_another_query() {
    for family in [0xe1, 0xca] {
        for (name, expected) in conversation(family)["errors"].as_object().unwrap() {
            let stage = expected["stage"].as_u64().unwrap() as u8;
            let peer = Peer::start(
                family,
                Reply::Failure {
                    stage,
                    name: name.clone(),
                },
            )
            .await;
            let clock = TestClock {
                calls: AtomicI64::new(0),
            };
            let result = query(family, peer.endpoint, &clock, &operation())
                .await
                .unwrap_err();
            assert_eq!(result.code, expected["error"], "{family}: {name}");
            assert_eq!(clock.calls.load(Ordering::Relaxed), i64::from(stage) + 1);
            peer.finish().await;
        }
    }
}

#[tokio::test]
async fn cancellation_deadline_and_drop_close_either_query_without_reconnect() {
    for family in [0xe1, 0xca] {
        for stage in [0, 1] {
            for reason in ["cancel", "deadline", "drop"] {
                let mut peer = Peer::start(family, Reply::Stall(stage)).await;
                let operation = operation();
                let clock = TestClock {
                    calls: AtomicI64::new(0),
                };
                let mut future = Box::pin(query(family, peer.endpoint, &clock, &operation));
                tokio::select! {
                    ready = &mut peer.ready => ready.unwrap(),
                    result = &mut future => panic!("The peer must reach the selected query: {result:?}"),
                }
                match reason {
                    "cancel" => {
                        operation.cancellation.cancel();
                        assert_eq!(future.await.unwrap_err().code, "CANCELLED");
                    }
                    "deadline" => {
                        tokio::time::pause();
                        tokio::time::advance(Duration::from_secs(10)).await;
                        assert_eq!(future.await.unwrap_err().code, "DEADLINE_EXCEEDED");
                        tokio::time::resume();
                    }
                    _ => drop(future),
                }
                peer.finish().await;
            }
        }
    }
}

#[tokio::test]
async fn cancelled_expired_invalid_target_and_endpoint_inputs_never_connect() {
    let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 2), 0))
        .await
        .unwrap();
    let endpoint = match listener.local_addr().unwrap() {
        std::net::SocketAddr::V4(endpoint) => endpoint,
        _ => unreachable!(),
    };
    for family in [0xe1, 0xca] {
        for reason in ["cancel", "deadline", "target", "endpoint"] {
            let mut operation = operation();
            if reason == "cancel" {
                operation.cancellation.cancel();
            }
            if reason == "deadline" {
                operation.deadline = tokio::time::Instant::now();
            }
            let target = CloudId(if reason == "target" { "0" } else { "123456789" }.into());
            let selected_endpoint = if reason == "endpoint" {
                SocketAddrV4::new(*endpoint.ip(), 0)
            } else {
                endpoint
            };
            let clock = TestClock {
                calls: AtomicI64::new(0),
            };
            let network = DiscoveryNetwork::loopback();
            let result = if family == 0xe1 {
                query_dishwasher_v3(
                    &network,
                    selected_endpoint,
                    credentials(),
                    &target,
                    &clock,
                    &operation,
                )
                .await
                .map(|_| ())
            } else {
                query_refrigerator_v3(
                    &network,
                    selected_endpoint,
                    credentials(),
                    &target,
                    &clock,
                    &operation,
                )
                .await
                .map(|_| ())
            };
            assert_eq!(
                result.unwrap_err().code,
                match reason {
                    "cancel" => "CANCELLED",
                    "deadline" => "DEADLINE_EXCEEDED",
                    _ => "INVALID_ARGUMENT",
                }
            );
        }
    }
    assert!(
        tokio::time::timeout(Duration::from_millis(30), listener.accept())
            .await
            .is_err()
    );
}
