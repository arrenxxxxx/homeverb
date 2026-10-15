//! Local metadata, observations, and execution journal. Credentials are excluded.

use homeverb_core::{DeviceLock, Store};
use homeverb_domain::*;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};

mod accounts;
mod journal;
mod migration;
mod organization;

pub const RECEIPT_LIMIT: usize = 100;

pub struct SqliteStore {
    connection: Mutex<Connection>,
    locks: PathBuf,
    _schema_lock: File,
}

fn storage_error<T>(_: T) -> Error {
    Error::new(
        "STORAGE_ERROR",
        "The local store could not complete the operation.",
    )
}
fn encode<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(storage_error)
}
fn decode<T: DeserializeOwned>(value: String) -> Result<T> {
    serde_json::from_str(&value).map_err(storage_error)
}

pub fn private_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(storage_error)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
            .map_err(storage_error)?;
    }
    Ok(())
}

impl SqliteStore {
    pub fn open(directory: &Path) -> Result<Self> {
        private_directory(directory)?;
        let locks = directory.join("locks");
        private_directory(&locks)?;
        let schema_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(directory.join("migration.lock"))
            .map_err(storage_error)?;
        schema_lock.try_lock_shared().map_err(|_| busy())?;
        let mut connection =
            Connection::open(directory.join("homeverb.sqlite3")).map_err(storage_error)?;
        connection
            .busy_timeout(Duration::from_millis(250))
            .map_err(storage_error)?;
        let version = migration::version(&connection)?;
        if version > migration::SCHEMA_VERSION {
            return Err(migration::unsupported());
        }
        if version < migration::SCHEMA_VERSION {
            schema_lock.unlock().map_err(storage_error)?;
            schema_lock.try_lock().map_err(|_| busy())?;
            migration::upgrade(&mut connection, directory)?;
            schema_lock.unlock().map_err(storage_error)?;
            schema_lock.try_lock_shared().map_err(|_| busy())?;
        }
        connection
            .execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")
            .map_err(storage_error)?;
        let store = Self {
            connection: Mutex::new(connection),
            locks,
            _schema_lock: schema_lock,
        };
        store.recover_orphaned_executions()?;
        Ok(store)
    }

    fn documents<T: DeserializeOwned>(&self, sql: &str) -> Result<Vec<T>> {
        let connection = self.connection.lock().map_err(storage_error)?;
        let mut statement = connection.prepare(sql).map_err(storage_error)?;
        let rows = statement
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(storage_error)?;
        rows.map(|r| decode(r.map_err(storage_error)?)).collect()
    }
}

impl Store for SqliteStore {
    fn devices(&self) -> Result<Vec<Descriptor>> {
        let connection = self.connection.lock().map_err(storage_error)?;
        let mut statement = connection.prepare("SELECT d.descriptor, p.alias FROM devices d LEFT JOIN device_preferences p ON p.device_id=d.id ORDER BY d.id").map_err(storage_error)?;
        statement
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
            })
            .map_err(storage_error)?
            .map(|row| {
                let (document, alias) = row.map_err(storage_error)?;
                let mut descriptor: Descriptor = decode(document)?;
                if let Some(alias) = alias {
                    descriptor.device.alias = alias;
                }
                Ok(descriptor)
            })
            .collect()
    }
    fn save_device(&self, descriptor: &Descriptor) -> Result<()> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        accounts::admit_connections(&transaction, &descriptor.device)?;
        let previous: Option<String> = transaction
            .query_row(
                "SELECT descriptor FROM devices WHERE id=?1",
                [&descriptor.device.id],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        if let Some(previous) = previous {
            let previous: Descriptor = decode(previous)?;
            previous.device.validate_rediscovery(&descriptor.device)?;
        }
        transaction.execute("INSERT INTO devices VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET descriptor=excluded.descriptor",
            params![descriptor.device.id, encode(descriptor)?]).map_err(storage_error)?;
        transaction.execute("INSERT INTO device_preferences (device_id, position) SELECT ?1, COALESCE(MAX(position)+1, 0) FROM device_preferences WHERE true ON CONFLICT(device_id) DO NOTHING", [&descriptor.device.id]).map_err(storage_error)?;
        transaction.commit().map_err(storage_error)
    }
    fn snapshot(&self, device: &str, component: &str) -> Result<Option<Snapshot>> {
        let document = self
            .connection
            .lock()
            .map_err(storage_error)?
            .query_row(
                "SELECT snapshot FROM snapshots WHERE device=?1 AND component=?2",
                params![device, component],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(storage_error)?;
        document.map(decode).transpose()
    }
    fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let document: String = transaction
            .query_row(
                "SELECT descriptor FROM devices WHERE id=?1",
                [&snapshot.device_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?
            .ok_or_else(|| {
                Error::new(
                    "DEVICE_NOT_FOUND",
                    "The device is not present in this local profile.",
                )
            })?;
        let descriptor: Descriptor = decode(document)?;
        accounts::admit_connections(&transaction, &descriptor.device)?;
        transaction.execute(
            "INSERT INTO snapshots VALUES (?1, ?2, ?3) ON CONFLICT(device, component) DO UPDATE SET snapshot=excluded.snapshot",
            params![snapshot.device_id, snapshot.component_id, encode(snapshot)?]).map_err(storage_error)?;
        transaction.commit().map_err(storage_error)
    }
    fn clear_cached_state(&self) -> Result<usize> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let removed = transaction
            .execute("DELETE FROM snapshots", [])
            .map_err(storage_error)?;
        transaction.commit().map_err(storage_error)?;
        Ok(removed)
    }
    fn clear_execution_history(&self) -> Result<usize> {
        self.retire_all_executions()
    }
    fn lock_device(&self, device: &str) -> Result<Box<dyn DeviceLock>> {
        // Preserve existing lock names; only IDs that exceed a 255-byte filename
        // after hex encoding need a bounded, stable digest name.
        let name: String = if device.len() <= 127 {
            device
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect()
        } else {
            let digest: String = Sha256::digest(device.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            format!("sha256-{digest}")
        };
        let file: File = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.locks.join(name))
            .map_err(storage_error)?;
        file.try_lock().map_err(|_| {
            Error::new(
                "RESOURCE_BUSY",
                "Another process is operating on this device.",
            )
        })?;
        self.recover_device_executions(device)?;
        Ok(Box::new(file))
    }
    fn execution(&self, request_id: &str) -> Result<Option<(Command, Execution)>> {
        self.load_execution(request_id)
    }
    fn save_execution(&self, command: &Command, execution: &Execution) -> Result<()> {
        self.write_execution(command, execution, false)
    }
    fn finish_execution(&self, command: &Command, execution: &Execution) -> Result<()> {
        self.write_execution(command, execution, true)
    }
    fn recent_executions(&self) -> Result<Vec<Execution>> {
        self.documents("SELECT result FROM executions ORDER BY rowid DESC LIMIT 100")
    }
    fn device_executions(&self, device_id: &str) -> Result<Vec<Execution>> {
        self.load_device_executions(device_id)
    }
    fn rooms(&self) -> Result<Vec<Room>> {
        self.load_rooms()
    }
    fn create_room(&self, id: &str, name: &str) -> Result<Room> {
        self.insert_room(id, name)
    }
    fn rename_room(&self, id: &str, name: &str) -> Result<Room> {
        self.update_room(id, name)
    }
    fn delete_room(&self, id: &str) -> Result<()> {
        self.remove_room(id)
    }
    fn reorder_rooms(&self, ids: &[String]) -> Result<Vec<Room>> {
        self.order_rooms(ids)
    }
    fn preferences(&self) -> Result<Vec<DevicePreferences>> {
        self.load_preferences()
    }
    fn device_cards(&self) -> Result<Vec<DeviceCard>> {
        self.load_cards()
    }
    fn set_preferences(&self, device: &str, input: &PreferencesInput) -> Result<DevicePreferences> {
        self.update_preferences(device, input)
    }
    fn reorder_devices(&self, ids: &[String]) -> Result<Vec<DevicePreferences>> {
        self.order_devices(ids)
    }
    fn assign_devices_to_room(&self, input: &RoomAssignment) -> Result<Vec<DevicePreferences>> {
        self.assign_room(input)
    }
}

fn busy() -> Error {
    Error::new(
        "RESOURCE_BUSY",
        "Close other profile users before upgrading this database.",
    )
}

#[cfg(test)]
mod tests;
