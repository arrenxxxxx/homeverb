use homeverb_host::HostRequest;
use serde_json::json;

#[test]
fn no_argument_methods_reject_undeclared_fields() {
    for method in [
        "get_api_info",
        "list_devices",
        "discover",
        "recent_executions",
        "list_rooms",
        "list_device_cards",
        "diagnostics",
    ] {
        let mut request = json!({
            "schema_version": "1.0", "operation_id": "contract-test",
            "timeout_ms": 10000, "request": {"method": method}
        });
        assert!(serde_json::from_value::<HostRequest>(request.clone()).is_ok());
        request["request"]["principal"] = json!("administrator");
        assert!(
            serde_json::from_value::<HostRequest>(request).is_err(),
            "{method} must not silently accept undeclared fields"
        );
    }
}
