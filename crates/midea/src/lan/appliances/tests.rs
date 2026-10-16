use super::*;
use serde_json::{Value, json};

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/lan-appliances.json")).unwrap()
}
fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn decode(family: u64, version: u8, frame: Vec<u8>) -> Result<Value> {
    let message = SecretBytes::new(frame);
    if family == 0xe1 {
        let status = decode_dishwasher_status(version, &message)?;
        assert_eq!(format!("{status:?}"), "DishwasherStatus([REDACTED])");
        Ok(json!({
            "status_code":status.status_code,"mode_code":status.mode_code,"additional_code":status.additional_code,
            "temperature_code":status.temperature_code,"door_open":status.door_open,
            "rinse_aid_low":status.rinse_aid_low,"salt_low":status.salt_low,"water_low":status.water_low,
            "remaining_minutes":status.remaining_minutes,
        }))
    } else {
        let status = decode_refrigerator_status(version, &message)?;
        assert_eq!(format!("{status:?}"), "RefrigeratorStatus([REDACTED])");
        Ok(json!({
            "mode_flags":status.mode_flags,"fridge_power":status.fridge_power,"freezer_power":status.freezer_power,
            "fridge_target_celsius":status.fridge_target_celsius,"freezer_target_celsius":status.freezer_target_celsius,
            "fridge_current_celsius":status.fridge_current_celsius,"freezer_current_celsius":status.freezer_current_celsius,
        }))
    }
}

#[test]
fn standard_queries_match_independent_vectors_for_each_version_without_global_state() {
    for case in fixture()["queries"].as_array().unwrap() {
        let family = case["family"].as_u64().unwrap();
        let version = case["protocol"].as_u64().unwrap() as u8;
        for _ in 0..2 {
            let query = if family == 0xe1 {
                encode_dishwasher_status_query(version)
            } else {
                encode_refrigerator_status_query(version)
            };
            assert_eq!(query.expose(), bytes(&case["frame"]));
            assert_eq!(format!("{query:?}"), "SecretBytes([REDACTED])");
            assert_eq!(
                decode(family, version, query.expose().to_vec())
                    .unwrap_err()
                    .code,
                "MIDEA_RESPONSE_INVALID"
            );
        }
    }
}

#[test]
fn selected_fields_match_source_checked_candidates_and_preserve_extension_boundaries() {
    for case in fixture()["valid"].as_array().unwrap() {
        let actual = decode(case["family"].as_u64().unwrap(), 8, bytes(&case["frame"])).unwrap();
        assert_eq!(actual, case["expected"], "{}", case["name"]);
    }
}

#[test]
fn invalid_and_unsupported_messages_never_return_partial_or_default_candidates() {
    let fixture = fixture();
    for case in fixture["invalid"].as_array().unwrap() {
        let error = decode(case["family"].as_u64().unwrap(), 8, bytes(&case["frame"])).unwrap_err();
        assert_eq!(error.code, case["error"], "{}", case["name"]);
        assert!(!error.message.contains(case["frame"].as_str().unwrap()));
    }
    for case in fixture["valid"].as_array().unwrap() {
        let frame = bytes(&case["frame"]);
        let family = case["family"].as_u64().unwrap();
        for length in 0..frame.len() {
            assert!(
                decode(family, 8, frame[..length].to_vec()).is_err(),
                "{length}"
            );
        }
        for index in 0..frame.len() {
            let mut corrupted = frame.clone();
            corrupted[index] ^= 1;
            assert!(decode(family, 8, corrupted).is_err(), "{index}");
        }
    }
}

#[test]
fn every_flag_byte_keeps_source_polarity_without_interpreting_other_flags() {
    let fixture = fixture();
    for family in [0xe1, 0xca] {
        let case = fixture["valid"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["family"] == family)
            .unwrap();
        for flags in 0..=255_u8 {
            let mut frame = bytes(&case["frame"]);
            frame[10 + if family == 0xe1 { 5 } else { 6 }] = flags;
            let last = frame.len() - 1;
            frame[last] = frame[1..last]
                .iter()
                .fold(0_u8, |sum, byte| sum.wrapping_sub(*byte));
            let state = decode(family, 8, frame).unwrap();
            if family == 0xe1 {
                assert_eq!(state["door_open"], flags & 1 == 0);
                assert_eq!(state["rinse_aid_low"], flags & 2 != 0);
                assert_eq!(state["salt_low"], flags & 4 != 0);
                assert_eq!(state["water_low"], flags & 0x80 != 0);
            } else {
                assert_eq!(state["fridge_power"], flags & 1 == 0);
                assert_eq!(state["freezer_power"], flags & 0x10 == 0);
            }
        }
    }
}

#[test]
fn packet_target_and_inner_family_context_are_checked_independently() {
    use crate::{
        CloudId,
        lan::{decode_lan_packet, encode_lan_packet},
    };
    for case in fixture()["valid"].as_array().unwrap() {
        let target = CloudId("123456789".into());
        let frame = SecretBytes::new(bytes(&case["frame"]));
        let packet =
            encode_lan_packet(&target, &frame, "2026-10-16T04:05:06Z".parse().unwrap()).unwrap();
        let inner = decode_lan_packet(&target, packet.expose()).unwrap();
        let family = case["family"].as_u64().unwrap();
        assert_eq!(
            decode(family, 8, inner.expose().to_vec()).unwrap(),
            case["expected"]
        );
        assert_eq!(
            decode_lan_packet(&CloudId("1".into()), packet.expose())
                .unwrap_err()
                .code,
            "DEVICE_IDENTITY_CONFLICT"
        );
        assert_eq!(
            decode(family, 9, inner.expose().to_vec()).unwrap_err().code,
            "MIDEA_RESPONSE_INVALID"
        );
        assert_eq!(
            decode(
                if family == 0xe1 { 0xca } else { 0xe1 },
                8,
                inner.expose().to_vec()
            )
            .unwrap_err()
            .code,
            "MIDEA_RESPONSE_INVALID"
        );
    }
}
