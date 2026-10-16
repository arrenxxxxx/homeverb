use super::*;
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/lan-appliance.json")).unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn family(value: &Value) -> ApplianceFamily {
    match value.as_u64().unwrap() {
        0xac => ApplianceFamily::AirConditioner,
        0xe1 => ApplianceFamily::Dishwasher,
        0xca => ApplianceFamily::Refrigerator,
        _ => unreachable!(),
    }
}

#[test]
fn appliance_queries_and_protocol_fields_match_independent_source_checked_vectors() {
    for case in fixture()["queries"].as_array().unwrap() {
        let query = encode_appliance_query(family(&case["family"])).unwrap();
        assert_eq!(query.expose(), bytes(&case["frame"]));
        assert_eq!(format!("{query:?}"), "SecretBytes([REDACTED])");
    }
    for case in fixture()["valid"].as_array().unwrap() {
        let message = SecretBytes::new(bytes(&case["frame"]));
        assert_eq!(
            u64::from(decode_appliance_protocol(family(&case["family"]), &message).unwrap()),
            case["protocol"]
        );
    }
}

#[test]
fn malformed_unrelated_and_truncated_appliance_replies_do_not_supply_a_default() {
    for case in fixture()["invalid"].as_array().unwrap() {
        let error = decode_appliance_protocol(
            ApplianceFamily::AirConditioner,
            &SecretBytes::new(bytes(&case["frame"])),
        )
        .unwrap_err();
        assert_eq!(error.code, case["error"], "{}", case["name"]);
        assert!(!error.message.contains(case["frame"].as_str().unwrap()));
    }
    for case in fixture()["valid"].as_array().unwrap() {
        let message = bytes(&case["frame"]);
        for length in 0..message.len() {
            assert!(
                decode_appliance_protocol(
                    family(&case["family"]),
                    &SecretBytes::new(message[..length].to_vec())
                )
                .is_err()
            );
        }
    }
    for family in [ApplianceFamily::Other(0xac), ApplianceFamily::Other(0xff)] {
        assert_eq!(
            encode_appliance_query(family).unwrap_err().code,
            "CAPABILITY_UNSUPPORTED"
        );
        assert_eq!(
            decode_appliance_protocol(family, &SecretBytes::new(vec![]))
                .unwrap_err()
                .code,
            "CAPABILITY_UNSUPPORTED"
        );
    }
}
