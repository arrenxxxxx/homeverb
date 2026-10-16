use crate::*;

impl SqliteStore {
    pub(crate) fn append_connection(&self, link: ValidatedConnectionLink) -> Result<Descriptor> {
        let mut connection = self.connection.lock().map_err(storage_error)?;
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        let (document, alias): (String, Option<String>) = transaction
            .query_row(
                "SELECT d.descriptor, p.alias FROM devices d LEFT JOIN device_preferences p ON p.device_id=d.id WHERE d.id=?1",
                [&link.descriptor().device.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(storage_error)?
            .ok_or_else(|| Error::new("DEVICE_NOT_FOUND", "The device is not present in this local profile."))?;
        let mut raw: Descriptor = decode(document)?;
        let mut effective = raw.clone();
        if let Some(alias) = alias {
            effective.device.alias = alias;
        }
        link.validate_current(&effective)?;
        raw.device.connections.push(link.connection().clone());
        accounts::admit_connections(&transaction, &raw.device)?;
        transaction
            .execute(
                "UPDATE devices SET descriptor=?1 WHERE id=?2",
                params![encode(&raw)?, raw.device.id],
            )
            .map_err(storage_error)?;
        transaction.commit().map_err(storage_error)?;
        effective.device.connections.push(link.connection().clone());
        Ok(effective)
    }
}
