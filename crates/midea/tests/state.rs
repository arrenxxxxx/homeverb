use async_trait::async_trait;
use chrono::{DateTime, Utc};
use homeverb_core::{Clock, CredentialRef, Operation, SecretBytes};
use homeverb_domain::{Error, Result, Transport as Route};
use homeverb_midea::*;
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio_util::sync::CancellationToken;

const APPLIANCE_ID: &str = "9007199254740993";

struct Transport {
    account: CredentialRef,
    family: &'static str,
    product: Value,
    replies: Mutex<VecDeque<Result<SecretBytes>>>,
    queries: Mutex<Vec<StateQuery>>,
    calls: AtomicUsize,
    delay: bool,
    inflight: AtomicBool,
}

struct Inflight<'a>(&'a AtomicBool);
impl Drop for Inflight<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

fn bytes(value: Value) -> SecretBytes {
    SecretBytes::new(serde_json::to_vec(&value).unwrap())
}

impl Transport {
    fn new(family: &'static str, data: Value) -> Self {
        Self {
            account: CredentialRef::new(),
            family,
            product: json!({}),
            replies: Mutex::new([Ok(bytes(json!({"code": "0", "data": data})))].into()),
            queries: Mutex::new(Vec::new()),
            calls: AtomicUsize::new(0),
            delay: false,
            inflight: AtomicBool::new(false),
        }
    }

    async fn appliance(&self) -> Appliance {
        retrieve_inventory(self, self.account, &operation())
            .await
            .unwrap()
            .appliances
            .remove(0)
    }
}

#[async_trait]
impl ReadTransport for Transport {
    async fn request(
        &self,
        account: &CredentialRef,
        endpoint: ReadEndpoint<'_>,
        _: &Operation,
    ) -> Result<SecretBytes> {
        assert_eq!(*account, self.account);
        match endpoint {
            ReadEndpoint::Homes => Ok(bytes(
                json!({"code": 0, "data": {"homeList": [{"homegroupId": 1}]}}),
            )),
            ReadEndpoint::HomeAppliances(_) => {
                let mut appliance = json!({"applianceCode": APPLIANCE_ID, "type": self.family});
                appliance
                    .as_object_mut()
                    .unwrap()
                    .extend(self.product.as_object().unwrap().clone());
                Ok(bytes(
                    json!({"code": 0, "data": {"homeList": [{"roomList": [{"applianceList": [appliance]}]}]}}),
                ))
            }
            ReadEndpoint::ApplianceState { appliance, query } => {
                assert_eq!(endpoint.path(), "/mjl/v1/device/status/lua/get");
                assert_eq!(appliance.expose(), APPLIANCE_ID);
                self.calls.fetch_add(1, Ordering::SeqCst);
                self.queries.lock().unwrap().push(query);
                self.inflight.store(true, Ordering::SeqCst);
                let _guard = Inflight(&self.inflight);
                if self.delay {
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
                self.replies
                    .lock()
                    .unwrap()
                    .pop_front()
                    .expect("Unexpected extra state query")
            }
        }
    }
}

struct FixedClock;
impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        "2026-10-15T01:00:00Z".parse().unwrap()
    }
}

fn operation() -> Operation {
    Operation::new(1000, CancellationToken::new()).unwrap()
}

async fn read(transport: &Transport) -> Result<CloudState> {
    let appliance = transport.appliance().await;
    read_cloud_state(
        transport,
        transport.account,
        &appliance,
        StateQuery::Default,
        &FixedClock,
        &operation(),
    )
    .await
}

#[tokio::test]
async fn air_conditioner_decodes_numeric_strings_without_inventing_power_mode_or_fan_semantics() {
    let transport = Transport::new(
        "AC",
        json!({
            "applianceCode": APPLIANCE_ID,
            "power": "off", "mode": "cool", "wind_speed": "102",
            "temperature": "25", "small_temperature": "0.5", "indoor_temperature": "27.2",
            "temperature_unit": "celsius", "sn": "PRIVATE-SERIAL"
        }),
    );
    let state = read(&transport).await.unwrap();
    assert_eq!(state.account_ref, transport.account);
    assert_eq!(state.appliance_id.expose(), APPLIANCE_ID);
    let snapshot = state.snapshot("dev-local-ac", "main").unwrap();
    assert_eq!(snapshot.observations["power.state"].value, json!(false));
    assert_eq!(
        snapshot.observations["climate.target_temperature"].value,
        json!(25.5)
    );
    assert_eq!(
        snapshot.observations["climate.current_temperature"].value,
        json!(27.2)
    );
    assert_eq!(
        snapshot.observations["vendor.midea.ac.mode"].value,
        json!("cool")
    );
    assert_eq!(
        snapshot.observations["vendor.midea.ac.wind_speed"].value,
        json!(102)
    );
    assert!(!snapshot.observations.contains_key("climate.mode"));
    assert!(!snapshot.observations.contains_key("fan.speed"));
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    assert_eq!(*transport.queries.lock().unwrap(), [StateQuery::Default]);
    assert_eq!(StateQuery::Default.payload(), json!({}));
}

#[tokio::test]
async fn refrigerator_keeps_compartments_on_one_local_device_with_separate_readings() {
    let transport = Transport::new(
        "CA",
        json!({
            "storage_power": "on", "storage_temperature": "4", "refrigeration_real_temperature": 5.5,
            "freezing_power": "off", "freezing_temperature": -18, "freezing_real_temperature": "-16.5"
        }),
    );
    let state = read(&transport).await.unwrap();
    let fridge = state.snapshot("dev-local-fridge", "fridge").unwrap();
    let freezer = state.snapshot("dev-local-fridge", "freezer").unwrap();
    assert_eq!(fridge.device_id, freezer.device_id);
    assert_eq!(
        fridge.observations["climate.target_temperature"].value,
        json!(4.0)
    );
    assert_eq!(
        freezer.observations["climate.target_temperature"].value,
        json!(-18.0)
    );
    assert_eq!(
        freezer.observations["climate.current_temperature"].value,
        json!(-16.5)
    );
    assert_eq!(fridge.observations["power.state"].value, json!(true));
    assert_eq!(freezer.observations["power.state"].value, json!(false));
    assert_eq!(
        state.snapshot("dev-local-fridge", "main").unwrap_err().code,
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(
        state.snapshot(" ", "fridge").unwrap_err().code,
        "INVALID_ARGUMENT"
    );
}

#[tokio::test]
async fn refrigerator_optional_modes_preserve_absence_and_reject_malformed_values() {
    let transport = Transport::new(
        "CA",
        json!({
            "storage_mode":"on", "freezing_mode":"off", "preservation_mode":"on",
            "left_flexzone_temperature":"-2", "variable_mode":"fresh_product_mode",
            "right_flexzone_temperature": null, "private_unknown":"PRIVATE-SERIAL"
        }),
    );
    let state = read(&transport).await.unwrap();
    let snapshot = state.snapshot("fixture-fridge", "fridge").unwrap();
    let readings = &snapshot.observations;
    assert_eq!(
        readings["vendor.midea.refrigerator.storage_mode"].value,
        json!(true)
    );
    assert_eq!(
        readings["vendor.midea.refrigerator.freezing_mode"].value,
        json!(false)
    );
    assert_eq!(
        readings["vendor.midea.refrigerator.left_flexzone_temperature"].value,
        json!(-2.0)
    );
    assert!(!readings.contains_key("vendor.midea.refrigerator.right_flexzone_temperature"));
    assert!(!readings.contains_key("vendor.midea.refrigerator.icea_mode_power"));
    assert!(
        !serde_json::to_string(&snapshot)
            .unwrap()
            .contains("PRIVATE")
    );
    assert!(readings.values().all(|r| r.stale));
    for bad in [
        json!({"storage_mode":1}),
        json!({"variable_mode":true}),
        json!({"left_flexzone_temperature":"invalid"}),
    ] {
        assert!(read(&Transport::new("CA", bad)).await.is_err());
    }
}

#[tokio::test]
async fn dishwasher_preserves_model_specific_program_status_and_door_codes() {
    for program in ["normal", "standard_wash", "90min"] {
        let transport = Transport::new(
            "E1",
            json!({
                "mode": program, "work_status": "work", "doorswitch": "1",
                "left_time": "0", "temperature": "48.5", "water_lack": 0,
                "softwater_lack": 1, "bright_lack": null
            }),
        );
        let state = read(&transport).await.unwrap();
        let snapshot = state.snapshot("dev-local-dishwasher", "main").unwrap();
        let readings = snapshot.observations;
        assert_eq!(
            readings["vendor.midea.dishwasher.mode"].value,
            json!(program)
        );
        assert_eq!(
            readings["vendor.midea.dishwasher.work_status"].value,
            json!("work")
        );
        assert_eq!(
            readings["vendor.midea.dishwasher.doorswitch"].value,
            json!(1)
        );
        assert_eq!(
            readings["vendor.midea.dishwasher.remaining_minutes"].value,
            json!(0)
        );
        assert_eq!(
            readings["vendor.midea.dishwasher.temperature"].value,
            json!(48.5)
        );
        assert!(!readings.contains_key("vendor.midea.dishwasher.bright_lack"));
        assert!(!readings.contains_key("appliance_cycle.running"));
        assert!(!readings.contains_key("door.open"));
    }
}

#[tokio::test]
async fn dishwasher_operator_is_optional_raw_stale_evidence_not_a_cycle_boolean() {
    for value in [
        json!(null),
        json!(""),
        json!("pause"),
        json!("start"),
        json!("future_operator"),
    ] {
        let state = read(&Transport::new("E1", json!({"operator":value})))
            .await
            .unwrap();
        let snapshot = state.snapshot("dev-local", "main").unwrap();
        if value.is_null() || value == json!("") {
            assert!(snapshot.observations.is_empty());
        } else {
            let observed = &snapshot.observations["vendor.midea.dishwasher.operator"];
            assert_eq!(observed.value, value);
            assert!(observed.stale && observed.observed_at.is_none());
            assert_eq!(snapshot.observations.len(), 1);
        }
    }
    for value in [
        json!(0),
        json!(true),
        json!("private value"),
        json!({"pause":true}),
    ] {
        assert_eq!(
            read(&Transport::new("E1", json!({"operator":value})))
                .await
                .unwrap_err()
                .code,
            "MIDEA_RESPONSE_INVALID"
        );
    }
}

#[tokio::test]
async fn dishwasher_empty_operator_preserves_other_readings_without_inventing_cycle_state() {
    let state = read(&Transport::new(
        "E1",
        json!({
            "operator":"", "work_status":"power_off", "mode":"eco_wash",
            "temperature":24, "left_time":0, "doorswitch":1,
            "water_lack":0, "softwater_lack":1, "bright_lack":0
        }),
    ))
    .await
    .unwrap();
    let snapshot = state.snapshot("dev-local", "main").unwrap();
    assert_eq!(snapshot.observations.len(), 8);
    assert_eq!(
        snapshot.observations["vendor.midea.dishwasher.work_status"].value,
        json!("power_off")
    );
    assert_eq!(
        snapshot.observations["vendor.midea.dishwasher.temperature"].value,
        json!(24.0)
    );
    assert_eq!(
        snapshot.observations["vendor.midea.dishwasher.remaining_minutes"].value,
        json!(0)
    );
    assert!(
        !snapshot
            .observations
            .contains_key("vendor.midea.dishwasher.operator")
    );
    assert!(
        !snapshot
            .observations
            .contains_key("appliance_cycle.running")
    );
    assert!(
        snapshot
            .observations
            .values()
            .all(|o| o.stale && o.observed_at.is_none())
    );
    // This sentinel is specific to operator; other empty tokens remain malformed.
    for field in ["mode", "work_status"] {
        assert_eq!(
            read(&Transport::new("E1", json!({field:""})))
                .await
                .unwrap_err()
                .code,
            "MIDEA_RESPONSE_INVALID"
        );
    }
}

#[tokio::test]
async fn missing_fields_and_partial_temperature_never_become_zero_or_false() {
    for (family, component, data) in [
        (
            "AC",
            "main",
            json!({"temperature": 25, "small_temperature": null}),
        ),
        ("AC", "main", json!({"small_temperature": 0.5})),
        (
            "AC",
            "main",
            json!({"power": null, "indoor_temperature": null}),
        ),
        ("CA", "fridge", json!({"freezing_temperature": -18})),
        ("CA", "freezer", json!({"storage_temperature": 4})),
        ("E1", "main", json!({"doorswitch": null, "left_time": null})),
        ("E1", "main", json!({})),
    ] {
        let state = read(&Transport::new(family, data)).await.unwrap();
        assert!(
            state
                .snapshot("dev-local", component)
                .unwrap()
                .observations
                .is_empty()
        );
    }
}

#[tokio::test]
async fn cloud_receipt_never_proves_freshness_and_projection_never_exports_raw_identity() {
    let transport = Transport::new(
        "AC",
        json!({
            "power": "on", "indoor_temperature": 26,
            "observed_at": "2026-10-15T01:00:00Z", "timestamp": 1792026000000_u64,
            "accessToken": "PRIVATE-TOKEN", "sn": "PRIVATE-SERIAL"
        }),
    );
    let state = read(&transport).await.unwrap();
    let snapshot = state.snapshot("dev-local", "main").unwrap();
    assert!(!snapshot.from_cache);
    assert!(!snapshot.warnings.is_empty());
    for observation in snapshot.observations.values() {
        assert_eq!(observation.source, Route::Cloud);
        assert_eq!(observation.received_at, FixedClock.now());
        assert!(observation.observed_at.is_none());
        assert!(observation.stale);
    }
    for text in [
        format!("{state:?}"),
        serde_json::to_string(&snapshot).unwrap(),
    ] {
        assert!(!text.contains("PRIVATE"));
        assert!(!text.contains(APPLIANCE_ID));
        assert!(!text.contains(&transport.account.to_string()));
    }
}

#[tokio::test]
async fn malformed_recognized_fields_fail_the_whole_response_without_raw_errors() {
    for (family, data) in [
        ("AC", json!({"power": true})),
        (
            "AC",
            json!({"power": "on", "indoor_temperature": "PRIVATE-MARKER"}),
        ),
        ("AC", json!({"indoor_temperature": "NaN"})),
        ("AC", json!({"indoor_temperature": "inf"})),
        ("AC", json!({"indoor_temperature": {"nested": 25}})),
        ("AC", json!({"temperature": 25.5, "small_temperature": 0.5})),
        ("AC", json!({"small_temperature": 1})),
        ("AC", json!({"small_temperature": -0.5})),
        ("AC", json!({"applianceCode": "10", "power": "on"})),
        (
            "CA",
            json!({"storage_temperature": [], "freezing_temperature": -18}),
        ),
        ("E1", json!({"left_time": -1})),
        ("E1", json!({"left_time": 1.5})),
        ("E1", json!({"doorswitch": false})),
        ("E1", json!({"mode": "PRIVATE TOKEN"})),
        ("E1", json!({"work_status": 1})),
    ] {
        let transport = Transport::new(family, data);
        let error = read(&transport).await.unwrap_err();
        assert_eq!(error.code, "MIDEA_RESPONSE_INVALID");
        assert!(!error.to_string().contains("PRIVATE"));
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn response_errors_are_bounded_sanitized_and_never_retried() {
    for (reply, expected) in [
        (
            Ok(bytes(json!({"code": 7400, "msg": "PRIVATE-TOKEN"}))),
            "AUTH_REQUIRED",
        ),
        (Ok(bytes(json!({"code": 40002}))), "AUTH_REQUIRED"),
        (
            Ok(bytes(json!({"code": 1014, "msg": "PRIVATE-TOKEN"}))),
            "MIDEA_REQUEST_REJECTED",
        ),
        (
            Ok(bytes(json!({"code": 0, "data": null}))),
            "MIDEA_RESPONSE_INVALID",
        ),
        (
            Ok(SecretBytes::new(b"PRIVATE-NONJSON".to_vec())),
            "MIDEA_RESPONSE_INVALID",
        ),
        (
            Ok(SecretBytes::new(vec![b'x'; MAX_RESPONSE_BYTES + 1])),
            "MIDEA_RESPONSE_INVALID",
        ),
        (
            Err(Error::new("RAW_NETWORK_ERROR", "PRIVATE-URL")),
            "MIDEA_REQUEST_FAILED",
        ),
    ] {
        let transport = Transport::new("AC", json!({}));
        *transport.replies.lock().unwrap() = [reply].into();
        let error = read(&transport).await.unwrap_err();
        assert_eq!(error.code, expected);
        assert!(!error.to_string().contains("PRIVATE"));
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn queries_remain_explicit_and_unsupported_combinations_never_send() {
    let transport = Transport::new("AC", json!({"indoor_temperature": 26}));
    let appliance = transport.appliance().await;
    read_cloud_state(
        &transport,
        transport.account,
        &appliance,
        StateQuery::IndoorTemperature,
        &FixedClock,
        &operation(),
    )
    .await
    .unwrap();
    assert_eq!(
        *transport.queries.lock().unwrap(),
        [StateQuery::IndoorTemperature]
    );
    assert_eq!(
        StateQuery::IndoorTemperature.payload(),
        json!({"query_type": "indoor_temperature"})
    );
    for (family, query) in [
        ("E1", StateQuery::IndoorTemperature),
        ("CA", StateQuery::IndoorTemperature),
        ("34", StateQuery::Default),
    ] {
        let transport = Transport::new(family, json!({}));
        let appliance = transport.appliance().await;
        assert_eq!(
            read_cloud_state(
                &transport,
                transport.account,
                &appliance,
                query,
                &FixedClock,
                &operation()
            )
            .await
            .unwrap_err()
            .code,
            "CAPABILITY_UNSUPPORTED"
        );
        assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
    }
    assert_eq!(
        read(&Transport::new(
            "CA",
            json!({"temperature_unit": "fahrenheit", "storage_temperature": 40})
        ))
        .await
        .unwrap_err()
        .code,
        "CAPABILITY_UNSUPPORTED"
    );
}

#[tokio::test(start_paused = true)]
async fn cancellation_and_deadlines_prevent_or_drop_work_without_retries() {
    let mut transport = Transport::new("AC", json!({}));
    let appliance = transport.appliance().await;
    let cancelled = operation();
    cancelled.cancellation.cancel();
    let mut expired = operation();
    expired.deadline = tokio::time::Instant::now();
    for (op, expected) in [(&cancelled, "CANCELLED"), (&expired, "DEADLINE_EXCEEDED")] {
        assert_eq!(
            read_cloud_state(
                &transport,
                transport.account,
                &appliance,
                StateQuery::Default,
                &FixedClock,
                op
            )
            .await
            .unwrap_err()
            .code,
            expected
        );
    }
    assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
    transport.delay = true;
    let op = operation();
    assert_eq!(
        read_cloud_state(
            &transport,
            transport.account,
            &appliance,
            StateQuery::Default,
            &FixedClock,
            &op
        )
        .await
        .unwrap_err()
        .code,
        "DEADLINE_EXCEEDED"
    );
    assert!(!transport.inflight.load(Ordering::SeqCst));
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    let op = operation();
    let cancel = async {
        tokio::task::yield_now().await;
        op.cancellation.cancel();
    };
    let (result, ()) = tokio::join!(
        read_cloud_state(
            &transport,
            transport.account,
            &appliance,
            StateQuery::Default,
            &FixedClock,
            &op
        ),
        cancel
    );
    assert_eq!(result.unwrap_err().code, "CANCELLED");
    assert!(!transport.inflight.load(Ordering::SeqCst));
    assert_eq!(transport.calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn scalar_ac_targets_follow_protected_mapping_without_write_constraints() {
    for (data, expected) in [
        (json!({"temperature":26}), Some(json!(26.0))),
        (json!({"temperature":"15.5"}), Some(json!(15.5))),
        (json!({"temperature":31}), Some(json!(31.0))),
        (
            json!({"temperature":26,"small_temperature":0.5}),
            Some(json!(26.0)),
        ),
        (
            json!({"temperature":26,"small_temperature":"unrelated"}),
            Some(json!(26.0)),
        ),
        (json!({"temperature":null,"small_temperature":0.5}), None),
        (json!({"small_temperature":0.5}), None),
    ] {
        let mut transport = Transport::new("AC", data);
        transport.product = json!({"modelNumber":1,"sn8":"23096245"});
        let snapshot = read(&transport)
            .await
            .unwrap()
            .snapshot("local-ac", "main")
            .unwrap();
        assert_eq!(
            snapshot
                .observations
                .get("climate.target_temperature")
                .map(|v| v.value.clone()),
            expected
        );
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
        for observation in snapshot.observations.values() {
            assert!(observation.stale && observation.observed_at.is_none());
            assert_eq!(observation.received_at, FixedClock.now());
        }
    }
    for value in [
        json!(true),
        json!({}),
        json!("NaN"),
        json!("inf"),
        json!("bad"),
    ] {
        let mut transport = Transport::new("AC", json!({"temperature":value}));
        transport.product = json!({"modelNumber":1,"sn8":"23096245"});
        assert_eq!(
            read(&transport).await.err().unwrap().code,
            "MIDEA_RESPONSE_INVALID"
        );
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn scalar_target_decoding_never_overrides_subtype_or_guesses_missing_evidence() {
    for product in [
        json!({}),
        json!({"sn8":"23096245"}),
        json!({"modelNumber":524,"sn8":"23096245"}),
        json!({"modelNumber":1,"sn8":"22397091"}),
        json!({"modelNumber":1,"sn8":"TEST1234","category":"wall_air_conditioner"}),
    ] {
        let mut transport = Transport::new("AC", json!({"temperature":26}));
        transport.product = product.clone();
        let snapshot = read(&transport)
            .await
            .unwrap()
            .snapshot("local-ac", "main")
            .unwrap();
        assert!(
            !snapshot
                .observations
                .contains_key("climate.target_temperature")
        );
        let mut transport = Transport::new("AC", json!({"temperature":26,"small_temperature":0.5}));
        transport.product = product;
        let snapshot = read(&transport)
            .await
            .unwrap()
            .snapshot("local-ac", "main")
            .unwrap();
        assert_eq!(
            snapshot.observations["climate.target_temperature"].value,
            json!(26.5)
        );
    }
}

#[tokio::test]
async fn additional_observations_require_the_reviewed_profile_and_preserve_unknown_codes() {
    let key = "vendor.midea.dishwasher.additional";
    for value in [
        json!(null),
        json!(0),
        json!(9),
        json!(10),
        json!(13),
        json!(14),
        json!(255),
        json!("9"),
    ] {
        let mut transport = Transport::new("E1", json!({"additional":value}));
        transport.product = json!({"modelNumber":1,"sn8":"760Y0026"});
        let state = read(&transport).await.unwrap();
        let snapshot = state.snapshot("dev-local", "main").unwrap();
        if value.is_null() {
            assert!(!snapshot.observations.contains_key(key));
        } else {
            let observation = &snapshot.observations[key];
            assert_eq!(
                observation.value,
                if value.is_string() { json!(9) } else { value }
            );
            assert!(observation.stale && observation.observed_at.is_none());
            assert_eq!(observation.source, Route::Cloud);
        }
    }
    for value in [
        json!(true),
        json!(-1),
        json!(9.5),
        json!("unknown"),
        json!({}),
        json!(4294967296_u64),
    ] {
        let mut transport = Transport::new("E1", json!({"additional":value}));
        transport.product = json!({"modelNumber":1,"sn8":"760Y0026"});
        assert_eq!(
            read(&transport).await.unwrap_err().code,
            "MIDEA_RESPONSE_INVALID"
        );
    }
    for profile in [
        json!({}),
        json!({"sn8":"760Y0026"}),
        json!({"modelNumber":1,"sn8":"7600016L"}),
        json!({"modelNumber":1,"sn8":"76006481"}),
    ] {
        let mut transport = Transport::new("E1", json!({"additional":"unreviewed"}));
        transport.product = profile;
        let state = read(&transport).await.unwrap();
        assert!(
            !state
                .snapshot("dev-local", "main")
                .unwrap()
                .observations
                .contains_key(key)
        );
    }
}
