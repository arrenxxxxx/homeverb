use homeverb_domain::{Action, Idempotency, Risk, Transport, VerificationMethod};
use serde_json::{Value, json};

#[test]
fn root_schema_forms_keep_core_argument_validation() {
    for (keyword, value) in [
        ("additionalProperties", json!({})),
        ("minProperties", json!(1.0)),
    ] {
        let mut parameters = json!({
            "type": "object",
            "properties": {
                "target_temperature": {"type": "number"},
                "mode": {
                    "type": "string",
                    "enum": ["normal", "eco", "dry"],
                    "pattern": "^(normal|eco)$"
                }
            },
            "required": [],
            "minProperties": 1
        });
        parameters[keyword] = value;
        let action = Action {
            parameters,
            transports: vec![Transport::Lan],
            idempotency: Idempotency::SetValue,
            risk: Risk::Low,
            atomic: true,
            independent_fields: true,
            verification: VerificationMethod::StateMatch,
        };
        assert!(
            action.validate_arguments(&json!({"mode": "eco"})).is_ok(),
            "{keyword}: valid mode"
        );
        assert!(
            action.validate_arguments(&json!({"mode": "dry"})).is_err(),
            "{keyword}: invalid mode"
        );
        assert!(
            action
                .validate_arguments(&Value::Object(Default::default()))
                .is_err(),
            "{keyword}: missing argument"
        );
    }
}
