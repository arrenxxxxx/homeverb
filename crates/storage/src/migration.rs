use crate::{private_directory, storage_error};
use homeverb_domain::{Error, Result};
use rusqlite::{
    Connection,
    backup::{Backup, StepResult},
};
use std::{fs::File, path::Path};

pub const SCHEMA_VERSION: u32 = 3;

pub fn unsupported() -> Error {
    Error::new(
        "STORAGE_VERSION_UNSUPPORTED",
        "The database format is newer than this application or is not a HomeVerb store.",
    )
}

pub fn version(connection: &Connection) -> Result<u32> {
    connection
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(storage_error)
}

pub fn upgrade(connection: &mut Connection, directory: &Path) -> Result<()> {
    let version = version(connection)?;
    if version > SCHEMA_VERSION {
        return Err(unsupported());
    }
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    if version == 0 {
        let tables: u32 = connection.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'", [], |r| r.get(0)).map_err(storage_error)?;
        if tables != 0 {
            return Err(unsupported());
        }
    } else {
        backup(connection, directory, version)?;
    }
    connection
        .execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")
        .map_err(storage_error)?;
    let transaction = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    if version == 0 {
        transaction.execute_batch("CREATE TABLE devices (id TEXT PRIMARY KEY, descriptor TEXT NOT NULL);
            CREATE TABLE snapshots (device TEXT NOT NULL, component TEXT NOT NULL, snapshot TEXT NOT NULL, PRIMARY KEY(device, component));
            CREATE TABLE executions (request_id TEXT PRIMARY KEY, command TEXT NOT NULL, result TEXT NOT NULL);").map_err(storage_error)?;
    }
    if version < 2 {
        transaction.execute_batch("CREATE TABLE rooms (
            id TEXT PRIMARY KEY, name TEXT NOT NULL, name_key TEXT NOT NULL UNIQUE,
            position INTEGER NOT NULL CHECK(position>=0));
        CREATE TABLE device_preferences (
            device_id TEXT PRIMARY KEY REFERENCES devices(id) ON DELETE CASCADE,
            alias TEXT, room_id TEXT REFERENCES rooms(id) ON DELETE SET NULL,
            favorite INTEGER NOT NULL DEFAULT 0 CHECK(favorite IN (0,1)),
            position INTEGER NOT NULL CHECK(position>=0));
        INSERT INTO device_preferences(device_id, position) SELECT id, ROW_NUMBER() OVER(ORDER BY id)-1 FROM devices;
        ALTER TABLE executions ADD COLUMN finalized INTEGER NOT NULL DEFAULT 0 CHECK(finalized IN (0,1));
        CREATE TABLE retired_requests (request_id TEXT PRIMARY KEY);
        ").map_err(storage_error)?;
    }
    transaction
        .execute_batch(
            "CREATE TABLE profile_namespace (
            singleton INTEGER PRIMARY KEY CHECK(singleton=1), namespace TEXT NOT NULL UNIQUE);
        CREATE TABLE accounts (
            id TEXT PRIMARY KEY, provider TEXT NOT NULL, region TEXT NOT NULL,
            label TEXT NOT NULL, credential_ref TEXT NOT NULL UNIQUE,
            credential_state TEXT NOT NULL CHECK(credential_state IN ('pending','stored')));
        PRAGMA user_version=3;",
        )
        .map_err(storage_error)?;
    transaction
        .execute(
            "INSERT INTO profile_namespace VALUES (1, ?1)",
            [uuid::Uuid::new_v4().to_string()],
        )
        .map_err(storage_error)?;
    transaction.commit().map_err(storage_error)
}

fn backup(connection: &Connection, directory: &Path, version: u32) -> Result<()> {
    let backups = directory.join("backups");
    private_directory(&backups)?;
    let path = backups.join(format!("schema-{version}-{}.sqlite3", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut destination = Connection::open(&path).map_err(storage_error)?;
        {
            let backup = Backup::new(connection, &mut destination).map_err(storage_error)?;
            if !matches!(backup.step(-1).map_err(storage_error)?, StepResult::Done) {
                return Err(Error::new(
                    "RESOURCE_BUSY",
                    "The database backup could not acquire a consistent snapshot.",
                ));
            }
        }
        let check: String = destination
            .query_row("PRAGMA quick_check", [], |r| r.get(0))
            .map_err(storage_error)?;
        if check != "ok" {
            return Err(storage_error(()));
        }
        drop(destination);
        File::open(&path)
            .and_then(|file| file.sync_all())
            .map_err(storage_error)?;
        #[cfg(unix)]
        File::open(&backups)
            .and_then(|file| file.sync_all())
            .map_err(storage_error)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(path);
    }
    result
}
