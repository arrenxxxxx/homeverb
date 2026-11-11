use super::*;

const DETAILS: &str = "vendor.midea.dishwasher.details";

fn inventory() -> Vec<Value> {
    super::dishwasher::inventory(json!({"sn8":"760064BA","modelNumber":3}))
}

#[tokio::test]
async fn gx_dishwasher_details_preserve_identity_cache_and_read_only_authority() {
    let f = Fixture::new(inventory()).await;
    let device = f.discover().await.remove(0);
    let descriptor = f.core.describe(&device.id).unwrap();
    let details = descriptor.capability("main", DETAILS).unwrap();
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../examples/capability.meiju-dishwasher-details.json"
    ))
    .unwrap();
    assert_eq!(json!(details), fixture);
    assert!(descriptor.capabilities.iter().all(|c| c.actions.is_empty()));
    *f.wire.state.lock().unwrap() = Some(json!({
        "work_status":"work_pause", "mode":"wash_dry", "base_door_on":0,
        "base_mode_time":26, "base_cur_temp":47.5, "more_auto_throw":1,
        "more_region_reinforce":3, "more_keep_device":0, "lock_on":true
    }));
    let before = f.read(&device, "main", false).await.unwrap();
    for (key, value) in [
        ("program", json!("wash_dry")),
        ("cycle_state", json!("work_pause")),
        ("door_state", json!("closed")),
        ("temperature", json!(47.5)),
        ("remaining_minutes", json!(26)),
        ("smart_dose", json!(true)),
        ("storage_enabled", json!(false)),
        ("child_lock", json!(true)),
    ] {
        assert_eq!(
            before.observations[&format!("{DETAILS}.{key}")].value,
            value
        );
    }
    assert!(
        !before
            .observations
            .contains_key(&format!("{DETAILS}.lower_zone_boost"))
    );
    assert!(
        !before
            .observations
            .contains_key(&format!("{DETAILS}.estimated_minutes"))
    );
    assert!(
        before
            .observations
            .values()
            .all(|v| v.stale && v.observed_at.is_none())
    );

    *f.wire.state.lock().unwrap() = Some(json!({"base_door_on":2,"base_cur_temp":60}));
    assert_eq!(
        f.read(&device, "main", false).await.unwrap_err().code,
        "MIDEA_RESPONSE_INVALID"
    );
    assert_eq!(
        json!(f.store.snapshot(&device.id, "main").unwrap().unwrap()),
        json!(before)
    );

    // A protected profile does not upgrade saved metadata during ordinary reads.
    let mut legacy = descriptor.clone();
    legacy.capabilities.retain(|c| c.id != DETAILS);
    for part in &mut legacy.device.components {
        part.capabilities.retain(|id| id != DETAILS);
    }
    f.store.save_device(&legacy).unwrap();
    *f.wire.state.lock().unwrap() = Some(json!({"base_door_on":1,"mode":"rapid"}));
    let old = f.read(&legacy.device, "main", false).await.unwrap();
    assert!(!old.observations.keys().any(|k| k.starts_with(DETAILS)));
    assert_eq!(json!(f.discover().await.remove(0)), json!(device));

    // Even explicit legacy command admission cannot turn the new read scope into writes.
    let adapter = MeijuAdapter::new(f.vault.clone(), f.wire.clone(), Arc::new(SystemClock))
        .with_dishwasher_control()
        .with_dishwasher_cycle_control();
    let refreshed = adapter.discover(&operation()).await.unwrap();
    assert!(
        refreshed[0]
            .capabilities
            .iter()
            .all(|c| c.actions.is_empty())
    );
    assert!(f.wire.controls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn gx_dishwasher_details_require_exact_protected_product_subtype_and_model() {
    for (key, value) in [
        ("sn8", json!("76006481")),
        ("modelNumber", json!(1)),
        ("modelNumber", Value::Null),
        ("sn8", Value::Null),
        ("productModel", json!("Other model")),
    ] {
        let mut entries = inventory();
        entries[0][key] = value;
        let f = Fixture::new(entries).await;
        let device = f.discover().await.remove(0);
        assert!(
            f.core
                .describe(&device.id)
                .unwrap()
                .capability("main", DETAILS)
                .is_none()
        );
    }
}
