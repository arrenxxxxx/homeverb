use crate::{SqliteStore, storage_error};
use homeverb_domain::*;
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::{HashMap, HashSet};

fn room_missing() -> Error {
    Error::new(
        "ROOM_NOT_FOUND",
        "The room does not exist in this local profile.",
    )
}
fn name_conflict() -> Error {
    Error::new("ROOM_NAME_CONFLICT", "A room already uses this name.")
}

fn rooms(connection: &Connection) -> Result<Vec<Room>> {
    let mut statement = connection
        .prepare("SELECT id, name, position FROM rooms ORDER BY position, id")
        .map_err(storage_error)?;
    statement
        .query_map([], |r| {
            Ok(Room {
                id: r.get(0)?,
                name: r.get(1)?,
                position: r.get(2)?,
            })
        })
        .map_err(storage_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(storage_error)
}

fn preferences(connection: &Connection) -> Result<Vec<DevicePreferences>> {
    let mut statement = connection.prepare("SELECT device_id, alias, room_id, favorite, position FROM device_preferences ORDER BY position, device_id").map_err(storage_error)?;
    statement
        .query_map([], |r| {
            Ok(DevicePreferences {
                device_id: r.get(0)?,
                alias: r.get(1)?,
                room_id: r.get(2)?,
                favorite: r.get(3)?,
                position: r.get(4)?,
            })
        })
        .map_err(storage_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(storage_error)
}

fn check_permutation(ids: &[String], current: &[String]) -> Result<()> {
    let unique: HashSet<_> = ids.iter().collect();
    if ids.len() != current.len()
        || unique.len() != ids.len()
        || current.iter().any(|id| !unique.contains(id))
    {
        return Err(Error::new(
            "INVALID_ARGUMENT",
            "Ordering must contain every current ID exactly once.",
        ));
    }
    Ok(())
}

impl SqliteStore {
    pub(crate) fn assign_room(&self, input: &RoomAssignment) -> Result<Vec<DevicePreferences>> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if let Some(room) = &input.room_id {
            let exists: bool = transaction
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM rooms WHERE id=?1)",
                    [room],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            if !exists {
                return Err(room_missing());
            }
        }
        let mut current: HashMap<_, _> = preferences(&transaction)?
            .into_iter()
            .map(|p| (p.device_id.clone(), p))
            .collect();
        let mut selected = Vec::with_capacity(input.device_ids.len());
        // Resolve every ID before changing any membership. Only this transaction writes the selection.
        for id in &input.device_ids {
            let mut preference = current.remove(id).ok_or_else(|| {
                Error::new(
                    "DEVICE_NOT_FOUND",
                    "Use exact device IDs from this profile.",
                )
            })?;
            preference.room_id = input.room_id.clone();
            selected.push(preference);
        }
        for id in &input.device_ids {
            transaction
                .execute(
                    "UPDATE device_preferences SET room_id=?1 WHERE device_id=?2",
                    params![input.room_id, id],
                )
                .map_err(storage_error)?;
        }
        transaction.commit().map_err(storage_error)?;
        Ok(selected)
    }

    pub(crate) fn load_cards(&self) -> Result<Vec<DeviceCard>> {
        let connection = self.connection.lock().map_err(storage_error)?;
        let mut statement = connection
            .prepare(
                "SELECT d.descriptor, p.device_id, p.alias, p.room_id, p.favorite, p.position
            FROM devices d JOIN device_preferences p ON p.device_id=d.id ORDER BY p.position, d.id",
            )
            .map_err(storage_error)?;
        statement
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    DevicePreferences {
                        device_id: r.get(1)?,
                        alias: r.get(2)?,
                        room_id: r.get(3)?,
                        favorite: r.get(4)?,
                        position: r.get(5)?,
                    },
                ))
            })
            .map_err(storage_error)?
            .map(|row| {
                let (document, preferences) = row.map_err(storage_error)?;
                let mut descriptor: Descriptor = crate::decode(document)?;
                if let Some(alias) = &preferences.alias {
                    descriptor.device.alias = alias.clone();
                }
                Ok(DeviceCard {
                    device: descriptor.device,
                    preferences,
                })
            })
            .collect()
    }

    pub(crate) fn load_rooms(&self) -> Result<Vec<Room>> {
        rooms(&*self.connection.lock().map_err(storage_error)?)
    }

    pub(crate) fn insert_room(&self, id: &str, name: &str) -> Result<Room> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let conflict: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM rooms WHERE name_key=?1)",
                [name.to_lowercase()],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if conflict {
            return Err(name_conflict());
        }
        transaction
            .execute(
                "INSERT INTO rooms SELECT ?1, ?2, ?3, COALESCE(MAX(position)+1, 0) FROM rooms",
                params![id, name, name.to_lowercase()],
            )
            .map_err(storage_error)?;
        let room = rooms(&transaction)?
            .into_iter()
            .find(|room| room.id == id)
            .ok_or_else(room_missing)?;
        transaction.commit().map_err(storage_error)?;
        Ok(room)
    }

    pub(crate) fn update_room(&self, id: &str, name: &str) -> Result<Room> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let mut room = rooms(&transaction)?
            .into_iter()
            .find(|room| room.id == id)
            .ok_or_else(room_missing)?;
        let conflict: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM rooms WHERE name_key=?1 AND id!=?2)",
                params![name.to_lowercase(), id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if conflict {
            return Err(name_conflict());
        }
        transaction
            .execute(
                "UPDATE rooms SET name=?1, name_key=?2 WHERE id=?3",
                params![name, name.to_lowercase(), id],
            )
            .map_err(storage_error)?;
        room.name = name.into();
        transaction.commit().map_err(storage_error)?;
        Ok(room)
    }

    pub(crate) fn remove_room(&self, id: &str) -> Result<()> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        if transaction
            .execute("DELETE FROM rooms WHERE id=?1", [id])
            .map_err(storage_error)?
            == 0
        {
            return Err(room_missing());
        }
        // Keep a contiguous local ordering after removal. FK deletion only unassigns devices.
        for (position, room) in rooms(&transaction)?.iter().enumerate() {
            transaction
                .execute(
                    "UPDATE rooms SET position=?1 WHERE id=?2",
                    params![position, room.id],
                )
                .map_err(storage_error)?;
        }
        transaction.commit().map_err(storage_error)
    }

    pub(crate) fn order_rooms(&self, ids: &[String]) -> Result<Vec<Room>> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        check_permutation(
            ids,
            &rooms(&transaction)?
                .into_iter()
                .map(|room| room.id)
                .collect::<Vec<_>>(),
        )?;
        for (position, id) in ids.iter().enumerate() {
            transaction
                .execute(
                    "UPDATE rooms SET position=?1 WHERE id=?2",
                    params![position, id],
                )
                .map_err(storage_error)?;
        }
        let ordered = rooms(&transaction)?;
        transaction.commit().map_err(storage_error)?;
        Ok(ordered)
    }

    pub(crate) fn load_preferences(&self) -> Result<Vec<DevicePreferences>> {
        preferences(&*self.connection.lock().map_err(storage_error)?)
    }

    pub(crate) fn update_preferences(
        &self,
        device: &str,
        input: &PreferencesInput,
    ) -> Result<DevicePreferences> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let position: Option<u32> = transaction
            .query_row(
                "SELECT position FROM device_preferences WHERE device_id=?1",
                [device],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        let position = position.ok_or_else(|| {
            Error::new(
                "DEVICE_NOT_FOUND",
                "Use an exact device ID from this profile.",
            )
        })?;
        if let Some(room) = &input.room_id {
            let exists: bool = transaction
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM rooms WHERE id=?1)",
                    [room],
                    |r| r.get(0),
                )
                .map_err(storage_error)?;
            if !exists {
                return Err(room_missing());
            }
        }
        transaction.execute("UPDATE device_preferences SET alias=?1, room_id=?2, favorite=?3 WHERE device_id=?4",
            params![input.alias, input.room_id, input.favorite, device]).map_err(storage_error)?;
        transaction.commit().map_err(storage_error)?;
        Ok(DevicePreferences {
            device_id: device.into(),
            alias: input.alias.clone(),
            room_id: input.room_id.clone(),
            favorite: input.favorite,
            position,
        })
    }

    pub(crate) fn order_devices(&self, ids: &[String]) -> Result<Vec<DevicePreferences>> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        check_permutation(
            ids,
            &preferences(&transaction)?
                .into_iter()
                .map(|p| p.device_id)
                .collect::<Vec<_>>(),
        )?;
        for (position, id) in ids.iter().enumerate() {
            transaction
                .execute(
                    "UPDATE device_preferences SET position=?1 WHERE device_id=?2",
                    params![position, id],
                )
                .map_err(storage_error)?;
        }
        let ordered = preferences(&transaction)?;
        transaction.commit().map_err(storage_error)?;
        Ok(ordered)
    }
}
