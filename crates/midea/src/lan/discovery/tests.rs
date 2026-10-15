use super::*;
use md5::{Digest, Md5};
use serde_json::Value;
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;

fn packet(name: &str) -> Vec<u8> {
    let fixture: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/lan-discovery.json"
    )))
    .unwrap();
    let text = fixture["valid"]
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["name"] == name)
        .unwrap()["packet"]
        .as_str()
        .unwrap();
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

fn with_id(mut packet: Vec<u8>, id: u64) -> Vec<u8> {
    let offset = if packet[0] == 0x83 { 8 } else { 0 };
    let end = packet.len() - if offset == 0 { 16 } else { 32 };
    packet[offset + 20..offset + 26].copy_from_slice(&id.to_le_bytes()[..6]);
    let mut checksum = Md5::new();
    checksum.update(&packet[offset..end]);
    checksum.update(super::super::SALT);
    packet[end..end + 16].copy_from_slice(&checksum.finalize());
    packet
}

fn operation() -> Operation {
    Operation::new(30_000, CancellationToken::new()).unwrap()
}

async fn peer(address: Ipv4Addr) -> UdpSocket {
    UdpSocket::bind(SocketAddrV4::new(address, 0))
        .await
        .unwrap()
}

fn v4(address: SocketAddr) -> SocketAddrV4 {
    match address {
        SocketAddr::V4(address) => address,
        _ => panic!("The fixture must use IPv4."),
    }
}

async fn start(
    operation: Operation,
) -> (
    tokio::task::JoinHandle<Result<DiscoveryReport>>,
    [UdpSocket; 2],
    SocketAddrV4,
) {
    let peers = [
        peer(Ipv4Addr::LOCALHOST).await,
        peer(Ipv4Addr::LOCALHOST).await,
    ];
    let targets = peers.each_ref().map(|peer| v4(peer.local_addr().unwrap()));
    // Only this private fixture admits loopback and overrides destination ports.
    let network = DiscoveryNetwork {
        address: Ipv4Addr::LOCALHOST,
        mask: 0xffff_ff00,
    };
    let task = tokio::spawn(async move { scan(&network, targets, &operation).await });
    let mut source = None;
    for peer in &peers {
        let mut bytes = [0; 73];
        let (size, address) =
            tokio::time::timeout(Duration::from_secs(30), peer.recv_from(&mut bytes))
                .await
                .unwrap()
                .unwrap();
        assert_eq!(&bytes[..size], &DISCOVERY_REQUEST);
        if let Some(source) = source {
            assert_eq!(source, v4(address));
        }
        source = Some(v4(address));
    }
    (task, peers, source.unwrap())
}

async fn assert_closed(address: SocketAddrV4) {
    let socket = UdpSocket::bind(address).await.unwrap();
    assert_eq!(socket.local_addr().unwrap(), SocketAddr::V4(address));
}

#[test]
fn discovery_requires_a_selected_local_subnet_and_filters_response_sources() {
    for (address, prefix) in [
        ("10.1.2.3", 8),
        ("172.16.2.3", 12),
        ("192.168.1.1", 30),
        ("169.254.1.1", 16),
    ] {
        let network = DiscoveryNetwork::new(address.parse().unwrap(), prefix).unwrap();
        assert_eq!(format!("{network:?}"), "DiscoveryNetwork([REDACTED])");
        assert!(network.admits(address.parse().unwrap()));
        assert!(!network.admits(network.broadcast()));
        assert!(!network.admits(Ipv4Addr::from(u32::from(network.address) & network.mask)));
        assert!(!network.admits(Ipv4Addr::new(203, 0, 113, 10)));
    }
    for (address, prefix) in [
        ("0.0.0.0", 24),
        ("127.0.0.1", 24),
        ("224.0.0.1", 24),
        ("203.0.113.10", 24),
        ("192.168.1.0", 24),
        ("192.168.1.255", 24),
        ("192.168.1.1", 8),
        ("169.254.1.1", 15),
        ("10.1.2.3", 0),
        ("10.1.2.3", 7),
        ("10.1.2.3", 31),
        ("10.1.2.3", 32),
        ("10.1.2.3", 255),
    ] {
        assert_eq!(
            DiscoveryNetwork::new(address.parse().unwrap(), prefix)
                .unwrap_err()
                .code,
            "INVALID_ARGUMENT"
        );
    }
}

#[tokio::test]
async fn probes_once_retains_conflicting_hints_and_rejects_oversized_or_foreign_responses() {
    let (task, peers, source) = start(operation()).await;
    for name in ["v2-ac", "v2-ac", "v3-ac", "v2-e1"] {
        peers[0].send_to(&packet(name), source).await.unwrap();
    }
    peers[1].send_to(b"invalid", source).await.unwrap();
    let foreign = peer(Ipv4Addr::new(127, 0, 1, 1)).await;
    foreign
        .send_to(&with_id(packet("v2-ac"), 54321), source)
        .await
        .unwrap();
    let mut oversized = with_id(packet("v3-maximum-datagram"), 98765);
    oversized.push(0);
    peers[1].send_to(&oversized, source).await.unwrap();
    let report = task.await.unwrap().unwrap();
    assert_eq!(report.received_datagrams, 7);
    assert_eq!(report.candidates.len(), 3);
    assert!(!report.limit_reached);
    assert!(report.candidates.iter().all(|candidate| {
        candidate.appliance_id.expose() == "12345"
            && *candidate.endpoint.ip() == Ipv4Addr::LOCALHOST
            && candidate.endpoint.port() == 6444
    }));
    assert!(!format!("{report:?}").contains("12345"));
    for peer in peers {
        assert_eq!(
            peer.try_recv_from(&mut [0; 73]).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
    assert_closed(source).await;
}

#[tokio::test]
async fn candidate_and_datagram_limits_stop_without_waiting_for_quiet() {
    for scenario in ["unique", "duplicate", "invalid"] {
        let (task, peers, source) = start(operation()).await;
        let count = if scenario == "unique" {
            MAX_CANDIDATES
        } else {
            MAX_RESPONSES
        };
        let base = packet("v2-ac");
        for index in 0..count {
            let response = match scenario {
                "unique" => with_id(base.clone(), index as u64 + 1),
                "duplicate" => base.clone(),
                _ => b"invalid".to_vec(),
            };
            peers[0].send_to(&response, source).await.unwrap();
            tokio::task::yield_now().await;
        }
        let report = task.await.unwrap().unwrap();
        assert!(report.limit_reached);
        assert_eq!(report.received_datagrams, count);
        assert_eq!(
            report.candidates.len(),
            match scenario {
                "unique" => count,
                "duplicate" => 1,
                _ => 0,
            }
        );
        assert_closed(source).await;
    }
}

#[tokio::test]
async fn cancelled_or_expired_discovery_does_not_try_to_bind_an_interface() {
    let network = DiscoveryNetwork::new(Ipv4Addr::new(10, 254, 254, 1), 24).unwrap();
    for cancelled in [true, false] {
        let mut operation = operation();
        let expected = if cancelled {
            operation.cancellation.cancel();
            "CANCELLED"
        } else {
            operation.deadline = Instant::now();
            "DEADLINE_EXCEEDED"
        };
        assert_eq!(
            discover(&network, &operation).await.unwrap_err().code,
            expected
        );
    }
}

#[tokio::test]
async fn cancellation_and_dropped_future_release_the_owned_socket() {
    for abort in [false, true] {
        let operation = operation();
        let cancellation = operation.cancellation.clone();
        let (task, _, source) = start(operation).await;
        if abort {
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
        } else {
            cancellation.cancel();
            assert_eq!(task.await.unwrap().unwrap_err().code, "CANCELLED");
        }
        assert_closed(source).await;
    }
}

#[tokio::test]
async fn collection_has_a_fixed_window_and_honors_an_expired_parent_budget() {
    for expire_parent in [false, true] {
        let (task, _, source) = start(operation()).await;
        // Real I/O completes before pausing; no auto-advance races socket setup.
        tokio::time::pause();
        tokio::time::advance(if expire_parent {
            Duration::from_secs(31)
        } else {
            WINDOW
        })
        .await;
        let result = task.await.unwrap();
        tokio::time::resume();
        if expire_parent {
            assert_eq!(result.unwrap_err().code, "DEADLINE_EXCEEDED");
        } else {
            let report = result.unwrap();
            assert!(report.candidates.is_empty());
            assert!(!report.limit_reached);
            assert_eq!(report.received_datagrams, 0);
        }
        assert_closed(source).await;
    }
}
