use super::*;
use homeverb_domain::{Phase, Status};

fn inventory(f: &Fixture, appliances: Vec<Value>) {
    f.wire
        .inventories
        .lock()
        .unwrap()
        .insert("FIRST".into(), appliances);
}

#[tokio::test]
async fn missing_membership_blocks_live_work_but_retains_local_data_and_reappearing_identity() {
    let f = Fixture::new(vec![appliances()[0].clone()]).await;
    let device = f.discover().await.remove(0);
    let core = f.control_core(&device, false);
    core.discover(&operation()).await.unwrap();
    let descriptor = core.describe(&device.id).unwrap();
    f.read(&device, "main", false).await.unwrap();
    let room = core.create_room("Study").unwrap();
    core.set_device_preferences(
        &device.id,
        PreferencesInput {
            alias: Some("Desk AC".into()),
            room_id: Some(room.id),
            favorite: true,
        },
    )
    .unwrap();
    let cards = json!(core.list_device_cards().unwrap());
    let first_command = power_command(&device, false);
    let accepted = core
        .execute(first_command.clone(), CancellationToken::new())
        .await;
    assert_eq!(accepted.status, Status::Accepted);

    // Partial retrieval retains prior membership, even when this response lists no device.
    inventory(&f, vec![]);
    f.wire.partial.store(true, Ordering::Relaxed);
    assert_eq!(
        core.discover(&operation()).await.unwrap_err().code,
        "MIDEA_INVENTORY_PARTIAL"
    );
    f.read(&device, "main", false).await.unwrap();
    let cache = json!(f.store.snapshot(&device.id, "main").unwrap());
    f.wire.partial.store(false, Ordering::Relaxed);
    core.discover(&operation()).await.unwrap();

    // Publishing an older descriptor cannot restore private membership.
    core.publish_discovery("midea.meiju", &[descriptor], &operation())
        .await
        .unwrap();
    let calls = f.calls();
    let sends = f.wire.controls.lock().unwrap().len();
    let reopened = f.control_core(&device, false);
    assert_eq!(
        f.read(&device, "main", false).await.unwrap_err().code,
        "MIDEA_DEVICE_NOT_LISTED"
    );
    let command = power_command(&device, false);
    reopened.plan(&command).unwrap();
    let refused = reopened
        .execute(command.clone(), CancellationToken::new())
        .await;
    assert_eq!(refused.status, Status::Failed);
    assert_eq!(
        refused.error.as_ref().unwrap().code,
        "MIDEA_DEVICE_NOT_LISTED"
    );
    assert_eq!(refused.attempts[0].phase, Phase::NotSent);
    assert_eq!(
        json!(
            reopened
                .execute(first_command, CancellationToken::new())
                .await
        ),
        json!(accepted)
    );
    f.read(&device, "main", true).await.unwrap();
    assert_eq!(json!(f.store.snapshot(&device.id, "main").unwrap()), cache);
    assert_eq!(json!(core.list_device_cards().unwrap()), cards);
    assert_eq!(f.calls(), calls);
    assert_eq!(f.wire.controls.lock().unwrap().len(), sends);

    inventory(&f, vec![appliances()[0].clone()]);
    assert_eq!(
        reopened.discover(&operation()).await.unwrap()[0].id,
        device.id
    );
    assert_eq!(json!(core.list_device_cards().unwrap()), cards);
    f.read(&device, "main", false).await.unwrap();
    // Reappearance never automatically replays a previously refused command.
    assert_eq!(
        json!(reopened.execute(command, CancellationToken::new()).await),
        json!(refused)
    );
    assert_eq!(f.wire.controls.lock().unwrap().len(), sends);
    assert_eq!(
        reopened
            .execute(power_command(&device, false), CancellationToken::new())
            .await
            .status,
        Status::Accepted
    );
}

#[tokio::test]
async fn legacy_bindings_need_explicit_complete_retrieval_and_keep_the_same_ids() {
    let f = Fixture::new(vec![appliances()[0].clone()]).await;
    let device = f.discover().await.remove(0);
    let core = f.control_core(&device, false);
    core.discover(&operation()).await.unwrap();
    for bytes in f.secrets.values.lock().unwrap().values_mut() {
        let mut payload: Value = serde_json::from_slice(bytes).unwrap();
        payload["version"] = json!(1);
        for binding in payload["bindings"].as_array_mut().unwrap() {
            binding.as_object_mut().unwrap().remove("listed");
        }
        *bytes = serde_json::to_vec(&payload).unwrap();
    }
    let legacy = f.secrets.values.lock().unwrap().clone();
    let calls = f.calls();
    assert_eq!(
        f.read(&device, "main", false).await.unwrap_err().code,
        "MIDEA_INVENTORY_REQUIRED"
    );
    let refused = core
        .execute(power_command(&device, false), CancellationToken::new())
        .await;
    assert_eq!(refused.error.unwrap().code, "MIDEA_INVENTORY_REQUIRED");
    assert_eq!(refused.attempts[0].phase, Phase::NotSent);
    assert_eq!(f.calls(), calls);
    assert_eq!(*f.secrets.values.lock().unwrap(), legacy);
    f.wire.partial.store(true, Ordering::Relaxed);
    assert_eq!(
        core.discover(&operation()).await.unwrap_err().code,
        "MIDEA_INVENTORY_PARTIAL"
    );
    assert_eq!(
        f.read(&device, "main", false).await.unwrap_err().code,
        "MIDEA_INVENTORY_REQUIRED"
    );
    assert_eq!(*f.secrets.values.lock().unwrap(), legacy);
    f.wire.partial.store(false, Ordering::Relaxed);
    assert_eq!(
        core.discover(&operation()).await.unwrap()[0].connections,
        device.connections
    );
    f.read(&device, "main", false).await.unwrap();
    for bytes in f.secrets.values.lock().unwrap().values() {
        let payload: Value = serde_json::from_slice(bytes).unwrap();
        assert_eq!(payload["version"], 2);
        assert_eq!(payload["bindings"][0]["listed"], true);
    }
    assert!(f.wire.controls.lock().unwrap().is_empty());
}
