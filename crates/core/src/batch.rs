use crate::*;
use serde_json::{Value, json};
use std::collections::HashSet;

impl Core {
    /// Validate the entire explicit selection without reserving or sending any write.
    pub async fn plan_batch(&self, batch: &BatchCommand, operation: &Operation) -> Result<Value> {
        operation.run(async { batch.validate() }).await?;
        let mut ids = HashSet::new();
        let mut plans = Vec::with_capacity(batch.commands.len());
        for command in &batch.commands {
            let (plan, parts) = operation
                .run(async { self.plan_with_parts(command) })
                .await?;
            let previous = operation
                .run(async { self.store.execution(&command.request_id) })
                .await?;
            if previous.as_ref().is_some_and(|(saved, _)| saved != command) {
                return Err(batch_conflict());
            }
            for item in std::iter::once(command).chain(parts.iter()) {
                if !ids.insert(item.request_id.clone()) {
                    return Err(batch_conflict());
                }
                if item.request_id != command.request_id
                    && previous.is_none()
                    && operation
                        .run(async { self.store.execution(&item.request_id) })
                        .await?
                        .is_some()
                {
                    return Err(batch_conflict());
                }
            }
            plans.push(plan);
        }
        Ok(json!({"items": plans, "max_concurrency": 4,
            "warnings": ["Local preflight only; the batch is not an atomic or durable job."]}))
    }

    /// Run independent devices in bounded groups, retaining input order and the host deadline.
    pub async fn execute_batch(
        &self,
        batch: BatchCommand,
        operation: &Operation,
    ) -> Result<BatchExecution> {
        let started_at = self.clock.now();
        self.plan_batch(&batch, operation).await?;
        let mut commands = batch.commands.into_iter();
        let mut items = Vec::new();
        while let Some(first) = commands.next() {
            let run = |command: Option<Command>| async {
                match command {
                    Some(command) => Some(self.execute_with_operation(command, operation).await),
                    None => None,
                }
            };
            let (first, second, third, fourth) = tokio::join!(
                run(Some(first)),
                run(commands.next()),
                run(commands.next()),
                run(commands.next())
            );
            items.extend([first, second, third, fourth].into_iter().flatten());
        }
        let first = items.first().expect("validated nonempty batch").status;
        let status = if items.iter().all(|item| item.status == first) {
            first
        } else {
            Status::Partial
        };
        Ok(BatchExecution {
            schema_version: SCHEMA_VERSION.into(),
            status,
            started_at,
            completed_at: self.clock.now(),
            items,
        })
    }
}

fn batch_conflict() -> Error {
    Error::new(
        "REQUEST_CONFLICT",
        "A batch command or derived child ID conflicts with another request.",
    )
}
