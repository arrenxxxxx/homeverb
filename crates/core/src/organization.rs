use crate::{Core, Operation};
use homeverb_domain::*;

impl Core {
    pub async fn clear_cached_state(&self, operation: &Operation) -> Result<usize> {
        operation
            .run(async { self.store.clear_cached_state() })
            .await
    }

    pub async fn clear_execution_history(&self, operation: &Operation) -> Result<usize> {
        operation
            .run(async { self.store.clear_execution_history() })
            .await
    }

    pub fn list_rooms(&self) -> Result<Vec<Room>> {
        self.store.rooms()
    }

    pub fn create_room(&self, name: &str) -> Result<Room> {
        self.store
            .create_room(&uuid::Uuid::new_v4().to_string(), &local_name(name)?)
    }

    pub fn rename_room(&self, id: &str, name: &str) -> Result<Room> {
        self.store.rename_room(id, &local_name(name)?)
    }

    pub fn delete_room(&self, id: &str) -> Result<()> {
        self.store.delete_room(id)
    }

    pub fn reorder_rooms(&self, ids: &[String]) -> Result<Vec<Room>> {
        self.store.reorder_rooms(ids)
    }

    pub fn list_device_cards(&self) -> Result<Vec<DeviceCard>> {
        self.store.device_cards()
    }

    pub fn set_device_preferences(
        &self,
        device_id: &str,
        mut input: PreferencesInput,
    ) -> Result<DevicePreferences> {
        input.alias = input.alias.as_deref().map(local_name).transpose()?;
        // The store requires an exact ID and checks room/device existence atomically.
        self.store.set_preferences(device_id, &input)
    }

    pub fn reorder_devices(&self, ids: &[String]) -> Result<Vec<DevicePreferences>> {
        self.store.reorder_devices(ids)
    }

    pub fn assign_devices_to_room(&self, input: RoomAssignment) -> Result<Vec<DevicePreferences>> {
        input.validate()?;
        self.store.assign_devices_to_room(&input)
    }
}
