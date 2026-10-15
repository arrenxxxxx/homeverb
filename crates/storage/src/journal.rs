use crate::{RECEIPT_LIMIT, SqliteStore, decode, encode, storage_error};
use homeverb_core::Store;
use homeverb_domain::*;
use rusqlite::{OptionalExtension, Transaction, params};

fn retired() -> Error {
    Error::new(
        "REQUEST_RETIRED",
        "This request was already recorded; its detailed receipt was retired. Refresh state without automatically replacing the write.",
    )
}

fn prune(transaction: &Transaction<'_>) -> Result<()> {
    transaction.execute("INSERT OR IGNORE INTO retired_requests SELECT request_id FROM executions WHERE finalized=1 ORDER BY rowid DESC LIMIT -1 OFFSET ?1", [RECEIPT_LIMIT]).map_err(storage_error)?;
    transaction
        .execute(
            "DELETE FROM executions WHERE rowid IN (SELECT rowid FROM executions WHERE finalized=1 ORDER BY rowid DESC LIMIT -1 OFFSET ?1)",
            [RECEIPT_LIMIT],
        )
        .map_err(storage_error)?;
    Ok(())
}

impl SqliteStore {
    pub(crate) fn load_device_executions(&self, device_id: &str) -> Result<Vec<Execution>> {
        let connection = self.connection.lock().map_err(storage_error)?;
        let mut statement = connection.prepare("SELECT result FROM executions WHERE json_extract(command, '$.device_id')=?1 ORDER BY rowid DESC LIMIT 100").map_err(storage_error)?;
        statement
            .query_map([device_id], |r| r.get::<_, String>(0))
            .map_err(storage_error)?
            .map(|row| decode(row.map_err(storage_error)?))
            .collect()
    }

    pub(crate) fn load_execution(&self, request_id: &str) -> Result<Option<(Command, Execution)>> {
        let connection = self.connection.lock().map_err(storage_error)?;
        let row = connection
            .query_row(
                "SELECT command, result FROM executions WHERE request_id=?1",
                [request_id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(storage_error)?;
        if let Some((command, execution)) = row {
            return Ok(Some((decode(command)?, decode(execution)?)));
        }
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM retired_requests WHERE request_id=?1)",
                [request_id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if exists {
            return Err(retired());
        }
        Ok(None)
    }

    pub(crate) fn write_execution(
        &self,
        command: &Command,
        execution: &Execution,
        finalized: bool,
    ) -> Result<()> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let is_retired: bool = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM retired_requests WHERE request_id=?1)",
                [&command.request_id],
                |r| r.get(0),
            )
            .map_err(storage_error)?;
        if is_retired {
            return Err(retired());
        }
        let existing: Option<String> = transaction
            .query_row(
                "SELECT command FROM executions WHERE request_id=?1",
                [&command.request_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        if let Some(existing) = existing
            && decode::<Command>(existing)? != *command
        {
            return Err(Error::new(
                "REQUEST_CONFLICT",
                "This request ID already identifies a different command.",
            ));
        }
        transaction.execute("INSERT INTO executions(request_id, command, result, finalized) VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(request_id) DO UPDATE SET result=excluded.result, finalized=excluded.finalized",
            params![command.request_id, encode(command)?, encode(execution)?, finalized]).map_err(storage_error)?;
        if finalized {
            prune(&transaction)?;
        }
        transaction.commit().map_err(storage_error)
    }

    pub(crate) fn recover_device_executions(&self, device: &str) -> Result<()> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        transaction.execute("UPDATE executions SET finalized=1 WHERE finalized=0 AND json_extract(command, '$.device_id')=?1", [device]).map_err(storage_error)?;
        prune(&transaction)?;
        transaction.commit().map_err(storage_error)
    }

    pub(crate) fn recover_orphaned_executions(&self) -> Result<()> {
        let devices = {
            let connection = self.connection.lock().map_err(storage_error)?;
            let mut statement = connection.prepare("SELECT DISTINCT json_extract(command, '$.device_id') FROM executions WHERE finalized=0").map_err(storage_error)?;
            statement
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(storage_error)?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(storage_error)?
        };
        for device in devices {
            match self.lock_device(&device) {
                Ok(lock) => drop(lock),
                Err(error) if error.code == "RESOURCE_BUSY" => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}
