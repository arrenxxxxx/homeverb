use homeverb_core::SecretBytes;
use homeverb_midea::lan::{V3Handshake, v3_frame_len};
use serde_json::Value;

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/lan-v3.json")).unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    let text = value.as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
        .collect()
}

fn handshake(fixture: &Value) -> V3Handshake {
    V3Handshake::new(
        SecretBytes::new(bytes(&fixture["token"])),
        SecretBytes::new(bytes(&fixture["key"])),
    )
    .unwrap()
}

#[test]
fn independent_handshake_and_encrypted_response_vectors() {
    let fixture = fixture();
    let exchange = handshake(&fixture);
    assert_eq!(exchange.request().expose(), bytes(&fixture["request"]));
    assert_eq!(format!("{exchange:?}"), "V3Handshake([REDACTED])");
    let session = exchange.finish(&bytes(&fixture["response"])).unwrap();
    assert_eq!(format!("{session:?}"), "V3Session([REDACTED])");
    for case in fixture["frames"].as_array().unwrap() {
        let frame = bytes(&case["frame"]);
        assert_eq!(v3_frame_len(&frame).unwrap(), Some(frame.len()));
        if case["type"] == 3 {
            let response = session.decode_response(&frame).unwrap();
            assert_eq!(response.sequence, 0);
            assert_eq!(response.payload.expose(), bytes(&case["payload"]));
        } else {
            assert_eq!(
                session.decode_response(&frame).unwrap_err().code,
                "MIDEA_RESPONSE_INVALID"
            );
        }
    }
}

#[test]
fn wrong_keys_corruption_and_incomplete_frames_never_release_plaintext() {
    let fixture = fixture();
    let response = bytes(&fixture["response"]);
    let wrong = V3Handshake::new(
        SecretBytes::new(bytes(&fixture["token"])),
        SecretBytes::new(vec![0xff; 32]),
    )
    .unwrap();
    assert_eq!(wrong.finish(&response).unwrap_err().code, "AUTH_REQUIRED");
    for length in 0..response.len() {
        assert!(handshake(&fixture).finish(&response[..length]).is_err());
    }
    for index in [0, 2, 4, 5, 8, 71] {
        let mut corrupted = response.clone();
        corrupted[index] ^= 1;
        assert!(handshake(&fixture).finish(&corrupted).is_err());
    }
    let session = handshake(&fixture).finish(&response).unwrap();
    for case in fixture["frames"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["type"] == 3)
    {
        let frame = bytes(&case["frame"]);
        for length in 0..frame.len() {
            assert!(session.decode_response(&frame[..length]).is_err());
        }
        for index in [0, 2, 4, 5, 6, frame.len() - 1] {
            let mut corrupted = frame.clone();
            corrupted[index] ^= 1;
            let error = session.decode_response(&corrupted).unwrap_err();
            assert_eq!(error.code, "MIDEA_RESPONSE_INVALID");
            assert_eq!(
                error.message,
                "The V3 LAN frame is malformed or unsupported."
            );
        }
    }
}

#[test]
fn credential_lengths_and_explicit_handshake_rejection_are_fixed_errors() {
    for (token, key) in [(0, 32), (63, 32), (65, 32), (64, 0), (64, 31), (64, 33)] {
        assert_eq!(
            V3Handshake::new(
                SecretBytes::new(vec![0; token]),
                SecretBytes::new(vec![0; key])
            )
            .unwrap_err()
            .code,
            "INVALID_ARGUMENT"
        );
    }
    let mut response = vec![0x83, 0x70, 0, 5, 0x20, 1, 0, 0];
    response.extend_from_slice(b"ERROR");
    let error = handshake(&fixture()).finish(&response).unwrap_err();
    assert_eq!(error.code, "AUTH_REQUIRED");
    assert_eq!(
        error.message,
        "The V3 LAN credentials could not be confirmed."
    );
}

#[test]
fn bounded_frame_prefixes_support_fragmentation_and_coalescing_without_buffering() {
    let fixture = fixture();
    let frame = bytes(&fixture["frames"][0]["frame"]);
    for length in 0..6 {
        assert_eq!(v3_frame_len(&frame[..length]).unwrap(), None);
    }
    for length in 6..frame.len() {
        assert_eq!(v3_frame_len(&frame[..length]).unwrap(), Some(frame.len()));
    }
    let joined = [frame.clone(), frame.clone()].concat();
    assert_eq!(v3_frame_len(&joined).unwrap(), Some(frame.len()));
    let session = handshake(&fixture)
        .finish(&bytes(&fixture["response"]))
        .unwrap();
    assert!(session.decode_response(&joined).is_err());
    for header in [
        [0x83, 0x70, 0xff, 0xff, 0x20, 3],
        [0x83, 0x70, 0, 0, 0x20, 3],
        [0x83, 0x70, 0, 47, 0x20, 3],
        [0x83, 0x70, 0, 46, 0x20, 0xf3],
        [0x83, 0x70, 0, 64, 0x20, 0x11],
        [0x83, 0x70, 0, 64, 0x20, 2],
    ] {
        assert!(v3_frame_len(&header).is_err());
    }
}
