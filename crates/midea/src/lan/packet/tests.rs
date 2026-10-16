use super::*;
use chrono::TimeZone;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/lan-packet.json")).unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn target() -> CloudId {
    CloudId("123456789".to_owned())
}

fn timestamp() -> DateTime<Utc> {
    fixture()["timestamp"].as_str().unwrap().parse().unwrap()
}

#[test]
fn independent_packets_match_exact_outgoing_bytes_and_protected_plaintext() {
    assert_eq!(target().expose(), fixture()["appliance_id"]);
    for vector in fixture()["valid"].as_array().unwrap() {
        let plain = bytes(&vector["plaintext"]);
        let packet = bytes(&vector["packet"]);
        let encoded =
            encode_lan_packet(&target(), &SecretBytes::new(plain.clone()), timestamp()).unwrap();
        assert_eq!(encoded.expose(), packet, "{}", vector["name"]);
        assert_eq!(lan_packet_len(&packet).unwrap(), Some(packet.len()));
        let decoded = decode_lan_packet(&target(), &packet).unwrap();
        assert_eq!(decoded.expose(), plain, "{}", vector["name"]);
        assert_eq!(format!("{decoded:?}"), "SecretBytes([REDACTED])");
        assert_eq!(format!("{encoded:?}"), "SecretBytes([REDACTED])");
    }
}

#[test]
fn malformed_packets_and_every_truncation_expose_no_plaintext() {
    let fixture = fixture();
    for vector in fixture["invalid"].as_array().unwrap() {
        let error = decode_lan_packet(&target(), &bytes(&vector["packet"])).unwrap_err();
        assert_eq!(error.code, vector["error"], "{}", vector["name"]);
        assert!(!format!("{error:?}").contains(target().expose()));
    }
    for vector in fixture["valid"].as_array().unwrap() {
        let packet = bytes(&vector["packet"]);
        for length in 0..packet.len() {
            assert!(
                decode_lan_packet(&target(), &packet[..length]).is_err(),
                "{length}"
            );
        }
        for offset in [0, 4, 8, 20, 40, packet.len() - 1] {
            let mut corrupt = packet.clone();
            corrupt[offset] ^= 1;
            assert!(decode_lan_packet(&target(), &corrupt).is_err(), "{offset}");
        }
    }
}

#[test]
fn prefix_boundaries_split_coalesced_packets_without_claiming_validity() {
    let fixture = fixture();
    let first = bytes(&fixture["valid"][0]["packet"]);
    let second = bytes(&fixture["valid"][3]["packet"]);
    for length in 0..6 {
        assert_eq!(lan_packet_len(&first[..length]).unwrap(), None);
    }
    let mut coalesced = first.clone();
    coalesced.extend_from_slice(&second);
    for length in 6..=coalesced.len() {
        assert_eq!(
            lan_packet_len(&coalesced[..length]).unwrap(),
            Some(first.len())
        );
    }
    assert!(decode_lan_packet(&target(), &coalesced).is_err());
    assert!(decode_lan_packet(&target(), &coalesced[..first.len()]).is_ok());
    assert!(decode_lan_packet(&target(), &coalesced[first.len()..]).is_ok());
    for size in [0_u16, 6, 40, 56, 71, 73, 4041, 4056, u16::MAX] {
        let mut prefix = first[..6].to_vec();
        prefix[4..6].copy_from_slice(&size.to_le_bytes());
        assert_eq!(
            lan_packet_len(&prefix).unwrap_err().code,
            "MIDEA_RESPONSE_INVALID"
        );
    }
}

#[test]
fn six_byte_identity_ignores_reserved_bytes_but_never_accepts_another_target() {
    let packet = bytes(&fixture()["reserved_bytes"]);
    assert_eq!(
        decode_lan_packet(&target(), &packet).unwrap().expose(),
        &[0]
    );
    assert_eq!(
        decode_lan_packet(&CloudId("1".into()), &packet)
            .unwrap_err()
            .code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    for id in ["0", "281474976710656", "18446744073709551616", "invalid"] {
        let target = CloudId(id.into());
        assert_eq!(
            decode_lan_packet(&target, &packet).unwrap_err().code,
            "INVALID_ARGUMENT"
        );
        assert_eq!(
            encode_lan_packet(&target, &SecretBytes::new(vec![1]), timestamp())
                .unwrap_err()
                .code,
            "INVALID_ARGUMENT"
        );
    }
    let largest = CloudId("281474976710655".into());
    let packet = encode_lan_packet(&largest, &SecretBytes::new(vec![1]), timestamp()).unwrap();
    assert_eq!(
        &packet.expose()[20..28],
        &[255, 255, 255, 255, 255, 255, 0, 0]
    );
    assert_eq!(
        decode_lan_packet(&largest, packet.expose())
            .unwrap()
            .expose(),
        &[1]
    );
}

#[test]
fn outgoing_size_time_and_padding_boundaries_never_wrap_or_truncate() {
    for size in [0, 3984, 65536] {
        assert_eq!(
            encode_lan_packet(&target(), &SecretBytes::new(vec![1; size]), timestamp())
                .unwrap_err()
                .code,
            "INVALID_ARGUMENT"
        );
    }
    let message = SecretBytes::new(vec![1]);
    for time in [
        Utc.with_ymd_and_hms(0, 1, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(10000, 1, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 59)
            .unwrap()
            .with_nanosecond(1_500_000_000)
            .unwrap(),
    ] {
        assert_eq!(
            encode_lan_packet(&target(), &message, time)
                .unwrap_err()
                .code,
            "INVALID_ARGUMENT"
        );
    }
    for (time, expected) in [
        (
            Utc.with_ymd_and_hms(1, 1, 1, 0, 0, 0).unwrap(),
            [0, 0, 0, 0, 1, 1, 1, 0],
        ),
        (
            Utc.with_ymd_and_hms(9999, 12, 31, 23, 59, 59)
                .unwrap()
                .with_nanosecond(999_999_999)
                .unwrap(),
            [99, 59, 59, 23, 31, 12, 99, 99],
        ),
    ] {
        let packet = encode_lan_packet(&target(), &message, time).unwrap();
        assert_eq!(&packet.expose()[12..20], &expected);
    }
}

#[test]
fn packets_fit_v3_and_independent_nested_response_preserves_identity_checks() {
    use crate::lan::{V3Handshake, v3_frame_len};
    let v3: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/lan-v3.json")).unwrap();
    let mut session = V3Handshake::new(
        SecretBytes::new(bytes(&v3["token"])),
        SecretBytes::new(bytes(&v3["key"])),
    )
    .unwrap()
    .finish(&bytes(&v3["response"]))
    .unwrap();
    let fixture = fixture();
    for vector in fixture["valid"].as_array().unwrap() {
        let packet = encode_lan_packet(
            &target(),
            &SecretBytes::new(bytes(&vector["plaintext"])),
            timestamp(),
        )
        .unwrap();
        let frame = session.encode_request(&packet).unwrap();
        assert!(frame.expose().len() <= 4096);
        assert_eq!(
            v3_frame_len(frame.expose()).unwrap(),
            Some(frame.expose().len())
        );
    }
    let response = session
        .decode_response(&bytes(&fixture["v3"]["response"]))
        .unwrap();
    assert_eq!(response.payload.expose(), bytes(&fixture["v3"]["packet"]));
    assert_eq!(
        decode_lan_packet(&target(), response.payload.expose())
            .unwrap()
            .expose(),
        bytes(&fixture["v3"]["plaintext"])
    );
    assert_eq!(
        decode_lan_packet(&CloudId("1".into()), response.payload.expose())
            .unwrap_err()
            .code,
        "DEVICE_IDENTITY_CONFLICT"
    );
}
