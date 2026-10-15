use homeverb_core::{Core, Operation, Policy, Store, SystemClock};
use homeverb_domain::*;
use homeverb_mock::{DEVICE_ID, MockAdapter, Scenario};
use homeverb_storage::SqliteStore;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
};
use tokio_util::sync::CancellationToken;

struct Profile {
    path: PathBuf,
    core: Core,
    store: Arc<SqliteStore>,
    adapter: Arc<MockAdapter>,
}
impl Profile {
    async fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("homeverb-organization-{}", uuid::Uuid::new_v4()));
        let store = Arc::new(SqliteStore::open(&path).unwrap());
        let clock = Arc::new(SystemClock);
        let adapter = Arc::new(
            MockAdapter::open(&path.join("mock.sqlite3"), clock.clone(), Scenario::Normal).unwrap(),
        );
        let core = Core::new(
            store.clone(),
            vec![adapter.clone()],
            clock,
            Policy::default(),
        );
        core.discover(&Operation::new(10000, CancellationToken::new()).unwrap())
            .await
            .unwrap();
        Self {
            path,
            core,
            store,
            adapter,
        }
    }
}
impl Drop for Profile {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[tokio::test]
async fn organization_survives_restart_and_rediscovery_without_changing_device_identity() {
    let p = Profile::new().await;
    let original = p.core.describe(DEVICE_ID).unwrap();
    let room = p.core.create_room("  Bedroom  ").unwrap();
    assert_eq!(room.name, "Bedroom");
    let preferences = p
        .core
        .set_device_preferences(
            DEVICE_ID,
            PreferencesInput {
                alias: Some("Quiet AC".into()),
                room_id: Some(room.id.clone()),
                favorite: true,
            },
        )
        .unwrap();
    assert_eq!(preferences.device_id, original.device.id);
    p.core
        .discover(&Operation::new(10000, CancellationToken::new()).unwrap())
        .await
        .unwrap();
    let reopened = homeverb_host::open_mock(&p.path, Scenario::Normal, false).unwrap();
    let card = reopened.list_device_cards().unwrap().remove(0);
    assert_eq!(card.preferences, preferences);
    assert_eq!(card.device.alias, "Quiet AC");
    assert_eq!(card.device.id, original.device.id);
    assert_eq!(
        serde_json::to_value(&card.device.connections).unwrap(),
        serde_json::to_value(&original.device.connections).unwrap()
    );
    assert_eq!(reopened.describe("Quiet AC").unwrap().device.id, DEVICE_ID);
    assert_eq!(p.adapter.sends.load(Ordering::SeqCst), 0);
    assert_eq!(p.adapter.reads.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn room_deletion_unassigns_devices_and_preserves_other_preferences() {
    let p = Profile::new().await;
    let room = p.core.create_room("Office").unwrap();
    p.core
        .set_device_preferences(
            DEVICE_ID,
            PreferencesInput {
                alias: Some("Office AC".into()),
                room_id: Some(room.id.clone()),
                favorite: true,
            },
        )
        .unwrap();
    p.core.delete_room(&room.id).unwrap();
    let card = p.core.list_device_cards().unwrap().remove(0);
    assert!(card.preferences.room_id.is_none());
    assert!(card.preferences.favorite);
    assert_eq!(card.device.alias, "Office AC");
    assert_eq!(p.core.list().unwrap().len(), 1);
    assert_eq!(
        p.core.delete_room(&room.id).unwrap_err().code,
        "ROOM_NOT_FOUND"
    );
    p.core
        .set_device_preferences(
            DEVICE_ID,
            PreferencesInput {
                alias: None,
                room_id: None,
                favorite: false,
            },
        )
        .unwrap();
    assert_eq!(p.core.describe(DEVICE_ID).unwrap().device.alias, "mock-ac");
}

#[tokio::test]
async fn invalid_names_rooms_and_reorders_fail_atomically() {
    let p = Profile::new().await;
    let first = p.core.create_room("Bedroom").unwrap();
    let second = p.core.create_room("Kitchen").unwrap();
    for name in ["", "  ", "bad\nname", &"x".repeat(81)] {
        assert_eq!(
            p.core.create_room(name).unwrap_err().code,
            "INVALID_ARGUMENT"
        );
    }
    assert_eq!(
        p.core.create_room("bedroom").unwrap_err().code,
        "ROOM_NAME_CONFLICT"
    );
    assert_eq!(
        p.core
            .rename_room(&second.id, " BEDROOM ")
            .unwrap_err()
            .code,
        "ROOM_NAME_CONFLICT"
    );
    assert_eq!(
        p.core.rename_room("missing", "Other").unwrap_err().code,
        "ROOM_NOT_FOUND"
    );
    let initial = p.core.list_rooms().unwrap();
    for ids in [
        vec![first.id.clone()],
        vec![first.id.clone(), first.id.clone()],
        vec![first.id.clone(), "foreign".into()],
    ] {
        assert_eq!(
            p.core.reorder_rooms(&ids).unwrap_err().code,
            "INVALID_ARGUMENT"
        );
        assert_eq!(p.core.list_rooms().unwrap(), initial);
    }
    assert_eq!(
        p.core
            .set_device_preferences(
                DEVICE_ID,
                PreferencesInput {
                    alias: Some("New name".into()),
                    room_id: Some("missing".into()),
                    favorite: true,
                }
            )
            .unwrap_err()
            .code,
        "ROOM_NOT_FOUND"
    );
    let card = p.core.list_device_cards().unwrap().remove(0);
    assert!(!card.preferences.favorite);
    assert!(card.preferences.alias.is_none());
}

#[tokio::test]
async fn ordering_and_alias_collisions_preserve_stable_ids() {
    let p = Profile::new().await;
    let first = p.core.create_room("First").unwrap();
    let second = p.core.create_room("Second").unwrap();
    let rooms = p
        .core
        .reorder_rooms(&[second.id.clone(), first.id.clone()])
        .unwrap();
    assert_eq!(rooms[0].id, second.id);
    assert_eq!(rooms[0].position, 0);
    assert_eq!(rooms[1].position, 1);
    let mut other = MockAdapter::descriptor();
    other.device.id = "dev-mock-second".into();
    p.store.save_device(&other).unwrap();
    for id in [DEVICE_ID, "dev-mock-second"] {
        p.core
            .set_device_preferences(
                id,
                PreferencesInput {
                    alias: Some("Shared label".into()),
                    room_id: None,
                    favorite: false,
                },
            )
            .unwrap();
    }
    assert_eq!(
        p.core.describe("Shared label").unwrap_err().code,
        "AMBIGUOUS_DEVICE"
    );
    assert_eq!(p.core.describe(DEVICE_ID).unwrap().device.id, DEVICE_ID);
    p.core
        .reorder_devices(&["dev-mock-second".into(), DEVICE_ID.into()])
        .unwrap();
    assert_eq!(
        p.core.list_device_cards().unwrap()[0].device.id,
        "dev-mock-second"
    );
    assert!(p.core.reorder_devices(&[DEVICE_ID.into()]).is_err());
    assert_eq!(
        p.core.list_device_cards().unwrap()[0].device.id,
        "dev-mock-second"
    );
}

#[tokio::test]
async fn rooms_are_profile_local_and_do_not_grant_writes() {
    let p = Profile::new().await;
    let other = Profile::new().await;
    let room = p.core.create_room("Private room").unwrap();
    assert!(other.core.list_rooms().unwrap().is_empty());
    assert_eq!(
        other
            .core
            .set_device_preferences(
                DEVICE_ID,
                PreferencesInput {
                    alias: None,
                    room_id: Some(room.id),
                    favorite: true,
                }
            )
            .unwrap_err()
            .code,
        "ROOM_NOT_FOUND"
    );
    let mut command: Command =
        serde_json::from_str(include_str!("../../../examples/command.json")).unwrap();
    command.device_id = DEVICE_ID.into();
    let result = p.core.execute(command, CancellationToken::new()).await;
    assert_eq!(result.error.unwrap().code, "POLICY_DENIED");
    assert_eq!(p.adapter.sends.load(Ordering::SeqCst), 0);
}

#[test]
fn preference_replacement_requires_explicit_nullable_fields() {
    assert!(
        serde_json::from_value::<PreferencesInput>(serde_json::json!({"favorite": true})).is_err()
    );
    assert!(
        serde_json::from_value::<PreferencesInput>(
            serde_json::json!({"alias": null, "room_id": null, "favorite": true})
        )
        .is_ok()
    );
    assert!(serde_json::from_value::<PreferencesInput>(serde_json::json!({"alias": null, "room_id": null, "favorite": true, "principal": "admin"})).is_err());
}

#[tokio::test]
async fn batch_room_assignment_preserves_preferences_and_returns_requested_order() {
    let p = Profile::new().await;
    let mut second = MockAdapter::descriptor();
    second.device.id = "dev-mock-second".into();
    p.store.save_device(&second).unwrap();
    let old = p.core.create_room("Old room").unwrap();
    let destination = p.core.create_room("New room").unwrap();
    for (id, favorite) in [(DEVICE_ID, true), ("dev-mock-second", false)] {
        p.core
            .set_device_preferences(
                id,
                PreferencesInput {
                    alias: Some(format!("Label {id}")),
                    room_id: Some(old.id.clone()),
                    favorite,
                },
            )
            .unwrap();
    }
    let before = p.core.list_device_cards().unwrap();
    let ids = vec!["dev-mock-second".into(), DEVICE_ID.into()];
    let moved = p
        .core
        .assign_devices_to_room(RoomAssignment {
            device_ids: ids.clone(),
            room_id: Some(destination.id.clone()),
        })
        .unwrap();
    assert_eq!(
        moved
            .iter()
            .map(|p| p.device_id.clone())
            .collect::<Vec<_>>(),
        ids
    );
    for preference in &moved {
        let prior = &before
            .iter()
            .find(|card| card.device.id == preference.device_id)
            .unwrap()
            .preferences;
        assert_eq!(preference.alias, prior.alias);
        assert_eq!(preference.favorite, prior.favorite);
        assert_eq!(preference.position, prior.position);
        assert_eq!(preference.room_id.as_deref(), Some(destination.id.as_str()));
    }
    let reopened = homeverb_host::open_mock(&p.path, Scenario::Normal, false).unwrap();
    for card in reopened.list_device_cards().unwrap() {
        assert_eq!(card.preferences.room_id.as_ref(), Some(&destination.id));
        let original = &before
            .iter()
            .find(|prior| prior.device.id == card.device.id)
            .unwrap()
            .device;
        assert_eq!(
            serde_json::to_value(&card.device).unwrap(),
            serde_json::to_value(original).unwrap()
        );
    }
    let unassigned = p
        .core
        .assign_devices_to_room(RoomAssignment {
            device_ids: ids,
            room_id: None,
        })
        .unwrap();
    assert!(unassigned.iter().all(|p| p.room_id.is_none()));
    assert_eq!(p.adapter.sends.load(Ordering::SeqCst), 0);
    assert_eq!(p.adapter.reads.load(Ordering::SeqCst), 0);
    assert!(p.core.recent_executions().unwrap().is_empty());
}

#[tokio::test]
async fn invalid_batch_assignments_change_nothing() {
    let p = Profile::new().await;
    let room = p.core.create_room("Destination").unwrap();
    let before = p.core.list_device_cards().unwrap().remove(0).preferences;
    for (ids, destination, code) in [
        (vec![], Some(room.id.clone()), "INVALID_ARGUMENT"),
        (
            vec![DEVICE_ID.into(), DEVICE_ID.into()],
            Some(room.id.clone()),
            "INVALID_ARGUMENT",
        ),
        (
            vec![DEVICE_ID.into(), "missing".into()],
            Some(room.id.clone()),
            "DEVICE_NOT_FOUND",
        ),
        (
            vec![DEVICE_ID.into()],
            Some("missing".into()),
            "ROOM_NOT_FOUND",
        ),
        (
            vec!["mock-ac".into()],
            Some(room.id.clone()),
            "DEVICE_NOT_FOUND",
        ),
        (
            (0..257).map(|i| format!("dev-{i}")).collect(),
            None,
            "INVALID_ARGUMENT",
        ),
    ] {
        let error = p
            .core
            .assign_devices_to_room(RoomAssignment {
                device_ids: ids,
                room_id: destination,
            })
            .unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(p.core.list_device_cards().unwrap()[0].preferences, before);
    }
    assert!(
        serde_json::from_value::<RoomAssignment>(serde_json::json!({"device_ids":[DEVICE_ID]}))
            .is_err()
    );
    assert!(
        serde_json::from_value::<RoomAssignment>(
            serde_json::json!({"device_ids":[DEVICE_ID], "room_id":null})
        )
        .is_ok()
    );
}

#[tokio::test]
async fn device_history_filters_before_the_query_limit_and_never_reads_an_adapter() {
    let p = Profile::new().await;
    let mut second = MockAdapter::descriptor();
    second.device.id = "dev-mock-second".into();
    p.store.save_device(&second).unwrap();
    let mut command: Command =
        serde_json::from_str(include_str!("../../../examples/command.json")).unwrap();
    command.device_id = DEVICE_ID.into();
    let mut result = Execution::new(&command, chrono::Utc::now());
    result.status = Status::Unknown;
    result.error = Some(Error::new(
        "WRITE_OUTCOME_UNKNOWN",
        "Fictional unresolved operation.",
    ));
    p.store.finish_execution(&command, &result).unwrap();
    let original = result.request_id.clone();
    // Active records on another device must not hide this device's retained receipt.
    for index in 0..105 {
        command.request_id = format!("other-{index}");
        command.device_id = second.device.id.clone();
        result.request_id = command.request_id.clone();
        result.device_id = command.device_id.clone();
        p.store.save_execution(&command, &result).unwrap();
    }
    assert!(
        p.core
            .recent_executions()
            .unwrap()
            .iter()
            .all(|r| r.device_id == second.device.id)
    );
    let selected = p.core.device_executions(DEVICE_ID).unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].request_id, original);
    assert_eq!(
        p.core.device_executions("dev-mock-second").unwrap().len(),
        100
    );
    assert_eq!(
        p.core.device_executions("missing").unwrap_err().code,
        "DEVICE_NOT_FOUND"
    );
    assert_eq!(
        p.core.device_executions("mock-ac").unwrap_err().code,
        "AMBIGUOUS_DEVICE"
    );
    assert_eq!(p.adapter.reads.load(Ordering::SeqCst), 0);
    assert_eq!(p.adapter.sends.load(Ordering::SeqCst), 0);
}
