use homeverb_midea::{
    ApplianceFamily,
    lan::{DISCOVERY_REQUEST, LanProtocol, decode_discovery},
};
use md5::{Digest, Md5};
use serde_json::Value;
use std::net::Ipv4Addr;

fn vectors() -> Value {
    serde_json::from_str(include_str!("fixtures/lan-discovery.json")).unwrap()
}

fn hex(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect()
}

#[test]
fn independent_discovery_vectors_return_only_untrusted_native_candidates() {
    let fixture = vectors();
    let source = fixture["source"].as_str().unwrap().parse().unwrap();
    for case in fixture["valid"].as_array().unwrap() {
        let packet = hex(case["packet"].as_str().unwrap());
        let candidate = decode_discovery(&packet, source).unwrap();
        assert_eq!(candidate.appliance_id.expose(), fixture["appliance_id"]);
        assert_eq!(*candidate.endpoint.ip(), source);
        assert_eq!(
            candidate.endpoint.port(),
            fixture["port"].as_u64().unwrap() as u16
        );
        assert_eq!(
            candidate.protocol,
            if case["protocol"] == 2 {
                LanProtocol::V2
            } else {
                LanProtocol::V3
            }
        );
        assert_eq!(
            candidate.family,
            match case["family"].as_u64().unwrap() {
                0xac => ApplianceFamily::AirConditioner,
                0xe1 => ApplianceFamily::Dishwasher,
                0xca => ApplianceFamily::Refrigerator,
                value => ApplianceFamily::Other(value as u16),
            }
        );
        assert_eq!(format!("{candidate:?}"), "DiscoveryCandidate([REDACTED])");
        let changed = decode_discovery(&packet, Ipv4Addr::new(192, 0, 2, 11)).unwrap();
        assert_eq!(*changed.endpoint.ip(), Ipv4Addr::new(192, 0, 2, 11));
    }
}

#[test]
fn malformed_discovery_never_exposes_bytes_or_returns_a_partial_candidate() {
    let fixture = vectors();
    let source = fixture["source"].as_str().unwrap().parse().unwrap();
    for case in fixture["invalid"].as_array().unwrap() {
        let error = decode_discovery(&hex(case["packet"].as_str().unwrap()), source).unwrap_err();
        assert_eq!(error.code, "MIDEA_RESPONSE_INVALID", "{}", case["name"]);
        assert_eq!(
            error.message,
            "The LAN discovery response is malformed or unsupported."
        );
    }
    for case in fixture["valid"].as_array().unwrap() {
        let packet = hex(case["packet"].as_str().unwrap());
        for length in 0..packet.len() {
            assert!(
                decode_discovery(&packet[..length], source).is_err(),
                "{length}"
            );
        }
        let offset = if case["protocol"] == 2 { 0 } else { 8 };
        let checksum_end = packet.len() - if offset == 0 { 0 } else { 16 };
        for index in [0, offset + 4, offset + 20, offset + 40, checksum_end - 1] {
            let mut corrupted = packet.clone();
            corrupted[index] ^= 1;
            assert!(decode_discovery(&corrupted, source).is_err(), "{index}");
        }
    }
    for packet in [
        vec![0; 513],
        b"<?xml version=\"1.0\"?><root/>".to_vec(),
        DISCOVERY_REQUEST.to_vec(),
    ] {
        assert!(decode_discovery(&packet, source).is_err());
    }
    let valid = hex(fixture["valid"][0]["packet"].as_str().unwrap());
    for source in [
        Ipv4Addr::UNSPECIFIED,
        Ipv4Addr::BROADCAST,
        Ipv4Addr::new(224, 0, 0, 1),
    ] {
        assert!(decode_discovery(&valid, source).is_err());
    }
}

#[test]
fn discovery_request_matches_the_pinned_noncredential_probe() {
    assert_eq!(DISCOVERY_REQUEST.len(), 72);
    assert_eq!(
        &DISCOVERY_REQUEST[..8],
        &[0x5a, 0x5a, 1, 0x11, 72, 0, 0x92, 0]
    );
    assert!(DISCOVERY_REQUEST[8..40].iter().all(|byte| *byte == 0));
    let salt = hex("78686469776a6e6368656b6434643531326368646a783564386534633339344432443753");
    let mut checksum = Md5::new();
    checksum.update(&DISCOVERY_REQUEST[..56]);
    checksum.update(salt);
    assert_eq!(&DISCOVERY_REQUEST[56..], checksum.finalize().as_slice());
}
