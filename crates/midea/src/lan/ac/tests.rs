use super::*;
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/lan-ac.json")).unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn explicit_query_matches_independent_frames_without_global_sequence_or_wrap() {
    for case in fixture()["queries"].as_array().unwrap() {
        let protocol = case["protocol"].as_u64().unwrap() as u8;
        let sequence = case["sequence"].as_u64().unwrap() as u8;
        for _ in 0..2 {
            let query = encode_ac_status_query(protocol, sequence).unwrap();
            assert_eq!(query.expose(), bytes(&case["frame"]));
            assert_eq!(format!("{query:?}"), "SecretBytes([REDACTED])");
            assert_eq!(
                decode_ac_c0_status(protocol, &query).unwrap_err().code,
                "CAPABILITY_UNSUPPORTED"
            );
        }
    }
    for sequence in [0, 254, 255] {
        assert_eq!(
            encode_ac_status_query(8, sequence).unwrap_err().code,
            "INVALID_ARGUMENT"
        );
    }
}

#[test]
fn bitwise_crc_matches_all_upstream_lookup_entries_and_multiple_bytes() {
    let table = bytes(&fixture()["crc8_table"]);
    assert_eq!(table.len(), 256);
    for value in 0..=255_u8 {
        assert_eq!(crc8(&[value]), table[usize::from(value)]);
    }
    for length in [0, 1, 2, 21, 255, 256] {
        let input: Vec<_> = (0..length).map(|i| i as u8).collect();
        let expected = input
            .iter()
            .fold(0_u8, |crc, byte| table[usize::from(crc ^ byte)]);
        assert_eq!(crc8(&input), expected);
    }
}

#[test]
fn c0_candidates_preserve_raw_codes_temperatures_and_unknown_sensors() {
    for case in fixture()["valid"].as_array().unwrap() {
        let state = decode_ac_c0_status(8, &SecretBytes::new(bytes(&case["frame"]))).unwrap();
        let projected = json!({
            "power":state.power,"mode_code":state.mode_code,"fan_speed_code":state.fan_speed_code,
            "target_celsius":state.target_celsius,"indoor_celsius":state.indoor_celsius,
            "outdoor_celsius":state.outdoor_celsius,
        });
        assert_eq!(projected, case["expected"], "{}", case["name"]);
        assert_eq!(format!("{state:?}"), "AcC0Status([REDACTED])");
    }
}

#[test]
fn malformed_or_unsupported_messages_never_return_partial_candidates() {
    let fixture = fixture();
    for case in fixture["invalid"].as_array().unwrap() {
        let error = decode_ac_c0_status(8, &SecretBytes::new(bytes(&case["frame"]))).unwrap_err();
        assert_eq!(error.code, case["error"], "{}", case["name"]);
        assert!(!error.message.contains(case["frame"].as_str().unwrap()));
    }
    for case in fixture["valid"].as_array().unwrap() {
        let frame = bytes(&case["frame"]);
        for length in 0..frame.len() {
            assert!(
                decode_ac_c0_status(8, &SecretBytes::new(frame[..length].to_vec())).is_err(),
                "{length}"
            );
        }
        for offset in [0, 1, 2, 8, 9, 10, 11, frame.len() - 1] {
            let mut corrupt = frame.clone();
            corrupt[offset] ^= 1;
            assert!(
                decode_ac_c0_status(8, &SecretBytes::new(corrupt)).is_err(),
                "{offset}"
            );
        }
    }
}

#[test]
fn packet_to_c0_chain_keeps_target_and_inner_context_checks_separate() {
    use crate::{
        CloudId,
        lan::{decode_lan_packet, encode_lan_packet},
    };
    let target = CloudId("123456789".into());
    let timestamp = "2026-10-16T04:05:06Z".parse().unwrap();
    let frame = SecretBytes::new(bytes(&fixture()["valid"][0]["frame"]));
    let packet = encode_lan_packet(&target, &frame, timestamp).unwrap();
    let inner = decode_lan_packet(&target, packet.expose()).unwrap();
    let status = decode_ac_c0_status(8, &inner).unwrap();
    assert_eq!(status.indoor_celsius, Some(25.2));
    assert_eq!(
        decode_lan_packet(&CloudId("1".into()), packet.expose())
            .unwrap_err()
            .code,
        "DEVICE_IDENTITY_CONFLICT"
    );
    assert_eq!(
        decode_ac_c0_status(1, &inner).unwrap_err().code,
        "MIDEA_RESPONSE_INVALID"
    );
    let query = encode_ac_status_query(8, 1).unwrap();
    let packet = encode_lan_packet(&target, &query, timestamp).unwrap();
    let inner = decode_lan_packet(&target, packet.expose()).unwrap();
    assert_eq!(
        decode_ac_c0_status(8, &inner).unwrap_err().code,
        "CAPABILITY_UNSUPPORTED"
    );
}
