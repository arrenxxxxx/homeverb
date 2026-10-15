use crate::observations::{matches_target, prepare_cached, prepare_live};
use crate::*;
use serde_json::{Value, json};

pub struct Core {
    pub(crate) store: Arc<dyn Store>,
    adapters: Vec<Arc<dyn Adapter>>,
    pub(crate) clock: Arc<dyn Clock>,
    policy: Policy,
    pub(crate) approval_context: Arc<std::sync::atomic::AtomicU64>,
}

impl Core {
    pub fn new(
        store: Arc<dyn Store>,
        adapters: Vec<Arc<dyn Adapter>>,
        clock: Arc<dyn Clock>,
        policy: Policy,
    ) -> Self {
        Self {
            store,
            adapters,
            clock,
            policy,
            approval_context: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn list(&self) -> Result<Vec<Device>> {
        Ok(self
            .store
            .devices()?
            .into_iter()
            .map(|d| d.device)
            .collect())
    }

    pub fn describe(&self, selector: &str) -> Result<Descriptor> {
        let descriptor = self.resolve(selector)?;
        crate::descriptors::validate(&descriptor)?;
        Ok(descriptor)
    }

    /// Resolve local identity for organization/history without admitting operational metadata.
    pub fn resolve_device_id(&self, selector: &str) -> Result<String> {
        Ok(self.resolve(selector)?.device.id)
    }

    fn resolve(&self, selector: &str) -> Result<Descriptor> {
        let devices = self.store.devices()?;
        if let Some(exact) = devices.iter().find(|d| d.device.id == selector) {
            return Ok(exact.clone());
        }
        let mut aliases = devices.into_iter().filter(|d| d.device.alias == selector);
        let found = aliases.next().ok_or_else(|| {
            Error::new(
                "DEVICE_NOT_FOUND",
                "The device is not present in this local profile.",
            )
        })?;
        if aliases.next().is_some() {
            return Err(Error::new(
                "AMBIGUOUS_DEVICE",
                "Use an exact device ID for this ambiguous alias.",
            ));
        }
        Ok(found)
    }

    pub async fn discover(&self, operation: &Operation) -> Result<Vec<Device>> {
        for adapter in &self.adapters {
            let descriptors = operation.run(adapter.discover(operation)).await?;
            self.publish_discovery(adapter.id(), &descriptors, operation)
                .await?;
        }
        self.list()
    }

    /// Admit a trusted, registered adapter's already-retrieved batch without fetching twice.
    /// This is an in-process integration boundary, never a public JSON import operation.
    pub async fn publish_discovery(
        &self,
        adapter_id: &str,
        descriptors: &[Descriptor],
        operation: &Operation,
    ) -> Result<()> {
        operation
            .run(async {
                if !self
                    .adapters
                    .iter()
                    .any(|adapter| adapter.id() == adapter_id)
                {
                    return Err(crate::descriptors::invalid());
                }
                Ok(())
            })
            .await?;
        let stored = operation.run(async { self.store.devices() }).await?;
        let mut device_ids = std::collections::HashSet::new();
        for descriptor in descriptors {
            operation
                .run(async { crate::descriptors::validate(descriptor) })
                .await?;
            if !device_ids.insert(&descriptor.device.id) {
                return Err(crate::descriptors::invalid());
            }
            if descriptor
                .device
                .connections
                .iter()
                .any(|c| c.adapter != adapter_id)
            {
                return Err(Error::new(
                    "CAPABILITY_UNSUPPORTED",
                    "The adapter returned a foreign connection.",
                ));
            }
            if let Some(previous) = stored.iter().find(|d| d.device.id == descriptor.device.id) {
                previous.device.validate_rediscovery(&descriptor.device)?;
            }
        }
        for descriptor in descriptors {
            operation.run(async { Ok(()) }).await?;
            self.store.save_device(descriptor)?;
        }
        Ok(())
    }

    pub(crate) fn routes(
        &self,
        device: &Device,
        route: Route,
        action: Option<&Action>,
    ) -> Vec<Connection> {
        let mut connections: Vec<_> = device
            .connections
            .iter()
            .filter(|c| {
                route.permits(c.transport)
                    && action.is_none_or(|a| a.transports.contains(&c.transport))
                    && self.adapters.iter().any(|a| a.id() == c.adapter)
            })
            .cloned()
            .collect();
        connections.sort_by_key(|c| if c.transport == Transport::Lan { 0 } else { 1 });
        connections
    }

    fn adapter(&self, connection: &Connection) -> &Arc<dyn Adapter> {
        self.adapters
            .iter()
            .find(|a| a.id() == connection.adapter)
            .expect("routes only includes registered adapters")
    }

    pub async fn read(
        &self,
        selector: &str,
        component: &str,
        route: Route,
        cached: bool,
        operation: &Operation,
    ) -> Result<Snapshot> {
        let descriptor = operation.run(async { self.describe(selector) }).await?;
        if !descriptor
            .device
            .components
            .iter()
            .any(|c| c.id == component)
        {
            return Err(unsupported());
        }
        if cached {
            let mut snapshot = operation
                .run(async { self.store.snapshot(&descriptor.device.id, component) })
                .await?
                .ok_or_else(|| {
                    Error::new("CACHE_MISS", "No observation is cached for this component.")
                })?;
            return operation
                .run(async {
                    prepare_cached(&mut snapshot, &descriptor, component, self.clock.now())?;
                    Ok(snapshot)
                })
                .await;
        }
        let mut last = unreachable(route);
        for connection in self.routes(&descriptor.device, route, None) {
            match operation
                .run(self.adapter(&connection).read(
                    &descriptor.device,
                    component,
                    &connection,
                    operation,
                ))
                .await
            {
                Ok(mut snapshot) => {
                    prepare_live(
                        &mut snapshot,
                        &descriptor,
                        component,
                        connection.transport,
                        self.clock.now(),
                    )?;
                    self.store.save_snapshot(&snapshot)?;
                    return Ok(snapshot);
                }
                Err(error) => {
                    if !matches!(
                        error.code.as_str(),
                        "LAN_UNREACHABLE" | "CLOUD_UNREACHABLE" | "READ_TIMEOUT"
                    ) {
                        return Err(error);
                    }
                    last = error;
                }
            }
        }
        Err(last)
    }

    pub fn recent_executions(&self) -> Result<Vec<Execution>> {
        self.store.recent_executions()
    }

    /// Read a saved receipt without device access, locking, or journal mutation.
    pub async fn execution(&self, request_id: &str, operation: &Operation) -> Result<Execution> {
        operation
            .run(async {
                if request_id.trim().is_empty() {
                    return Err(Error::invalid());
                }
                self.store
                    .execution(request_id)?
                    .map(|(_, receipt)| receipt)
                    .ok_or_else(|| {
                        Error::new(
                            "REQUEST_NOT_FOUND",
                            "No receipt exists for this request ID in the selected local profile.",
                        )
                    })
            })
            .await
    }

    pub fn device_executions(&self, device_id: &str) -> Result<Vec<Execution>> {
        if self.resolve(device_id)?.device.id != device_id {
            return Err(Error::invalid());
        }
        self.store.device_executions(device_id)
    }

    pub fn plan(&self, command: &Command) -> Result<Value> {
        self.plan_with_parts(command).map(|(plan, _)| plan)
    }

    pub(crate) fn plan_with_parts(&self, command: &Command) -> Result<(Value, Vec<Command>)> {
        let (descriptor, action) = self.validate(command)?;
        let device = &descriptor.device;
        let parts = self.authorized_parts(command, &action)?;
        let routes = self.routes(device, command.connection, Some(&action));
        if routes.is_empty() {
            return Err(unreachable(command.connection));
        }
        let mut plan = json!({"command": command, "transports": routes.iter().map(|c| c.transport).collect::<Vec<_>>(),
            "warnings": ["Local validation only; capability metadata may be stale. No write or login was performed."]});
        let mut commands = Vec::with_capacity(parts.len());
        if !parts.is_empty() {
            let mut items = Vec::with_capacity(parts.len());
            for (part, action) in parts {
                let routes = self.routes(device, part.connection, Some(&action));
                if routes.is_empty() {
                    return Err(unreachable(part.connection));
                }
                items.push(json!({"command": part, "transports": routes.iter().map(|c| c.transport).collect::<Vec<_>>()}));
                commands.push(part);
            }
            plan["items"] = json!(items);
        }
        Ok((plan, commands))
    }

    fn validate(&self, command: &Command) -> Result<(Descriptor, Action)> {
        self.validate_command(command, false)
    }

    pub(crate) fn validate_elevated(&self, command: &Command) -> Result<(Descriptor, Action)> {
        let (descriptor, action) = self.validate_command(command, true)?;
        if action.independent_fields {
            return Err(crate::approval::denied());
        }
        Ok((descriptor, action))
    }

    fn validate_command(&self, command: &Command, elevated: bool) -> Result<(Descriptor, Action)> {
        command.validate()?;
        let descriptor = self.describe(&command.device_id)?;
        // Commands use stable IDs. Alias resolution belongs to trusted entry points.
        if descriptor.device.id != command.device_id {
            return Err(Error::invalid());
        }
        if !descriptor
            .device
            .components
            .iter()
            .any(|c| c.id == command.component_id && c.capabilities.contains(&command.capability))
        {
            return Err(unsupported());
        }
        let capability = descriptor
            .capability(&command.component_id, &command.capability)
            .ok_or_else(unsupported)?;
        let mut action = capability
            .actions
            .get(&command.action)
            .ok_or_else(unsupported)?
            .clone();
        action.validate_arguments(&command.arguments)?;
        action.transports = if elevated {
            self.policy.authorize_elevated(command, &action)?
        } else {
            self.policy.authorize(command, &action)?
        };
        Ok((descriptor, action))
    }

    fn authorized_parts(
        &self,
        command: &Command,
        action: &Action,
    ) -> Result<Vec<(Command, Action)>> {
        let arguments = command.arguments.as_object().expect("validated object");
        if !action.independent_fields || arguments.len() <= 1 {
            return Ok(Vec::new());
        }
        arguments
            .iter()
            .enumerate()
            .map(|(index, (key, value))| {
                let mut part = command.clone();
                part.arguments = json!({key: value});
                let (_, mut part_action) = self.validate(&part)?;
                // The caller's ID is already validated. Internal receipt IDs append
                // a suffix without reducing the supported caller ID length.
                part.request_id = format!("{}:{index}", command.request_id);
                // Splitting a command can narrow its route authority, never expand it.
                part_action
                    .transports
                    .retain(|transport| action.transports.contains(transport));
                if part_action.transports.is_empty() {
                    return Err(crate::policy::denied());
                }
                Ok((part, part_action))
            })
            .collect()
    }

    pub async fn execute(&self, command: Command, cancellation: CancellationToken) -> Execution {
        match Operation::new(command.timeout_ms, cancellation) {
            Ok(operation) => self.execute_with_operation(command, &operation).await,
            Err(error) => {
                let mut execution = Execution::new(&command, self.clock.now());
                execution.error = Some(error);
                execution
            }
        }
    }

    /// Preserve an earlier host deadline while also enforcing the command's own budget.
    pub async fn execute_with_operation(
        &self,
        command: Command,
        operation: &Operation,
    ) -> Execution {
        self.execute_authorized(command, operation, None).await
    }

    /// Trusted host only: consume exactly the preview the user confirmed.
    pub async fn execute_user_approved(
        &self,
        approval: PendingApproval,
        operation: &Operation,
    ) -> Execution {
        self.execute_authorized(approval.command().clone(), operation, Some(&approval))
            .await
    }

    async fn execute_authorized(
        &self,
        command: Command,
        operation: &Operation,
        approval: Option<&PendingApproval>,
    ) -> Execution {
        let mut execution = Execution::new(&command, self.clock.now());
        let result = async {
            let mut scoped = Operation::new(command.timeout_ms, operation.cancellation.clone())?;
            scoped.deadline = scoped.deadline.min(operation.deadline);
            if let Some(approval) = approval {
                scoped.deadline = scoped.deadline.min(approval.deadline);
            }
            self.execute_inner(&command, &scoped, &mut execution, approval)
                .await
        }
        .await;
        if let Err(error) = result {
            preserve_outcome(&mut execution, error);
            execution.completed_at = self.clock.now();
        }
        execution
    }

    async fn execute_inner(
        &self,
        command: &Command,
        operation: &Operation,
        execution: &mut Execution,
        approval: Option<&PendingApproval>,
    ) -> Result<()> {
        let (descriptor, action) = if let Some(approval) = approval {
            let result = self.validate_elevated(command)?;
            approval.validate(self, &result.0)?;
            result
        } else {
            self.validate(command)?
        };
        let _lock = self.store.lock_device(&descriptor.device.id)?;
        if let Some((previous_command, previous)) = self.store.execution(&command.request_id)? {
            if previous_command != *command {
                return Err(Error::new(
                    "REQUEST_CONFLICT",
                    "This request ID already identifies a different command.",
                ));
            }
            *execution = previous;
            return Ok(());
        }
        if approval.is_some() {
            execution
                .warnings
                .push("User approval was supplied for this exact command.".into());
        }
        let authorized_parts = self.authorized_parts(command, &action)?;
        if !authorized_parts.is_empty() {
            // All subcommands are authorized before reserving or sending any write.
            for (part, _) in &authorized_parts {
                if self.store.execution(&part.request_id)?.is_some() {
                    return Err(Error::new(
                        "REQUEST_CONFLICT",
                        "A sub-operation request ID is already in use.",
                    ));
                }
            }
            execution.status = Status::Unknown;
            execution.error = Some(uncertain());
            if let Err(error) = self.store.save_execution(command, execution) {
                execution.status = Status::Failed;
                execution.error = None;
                return Err(error);
            }
            let mut items = Vec::new();
            let mut stop = false;
            for (part, part_action) in authorized_parts {
                let mut item = Execution::new(&part, self.clock.now());
                if stop {
                    item.error = Some(Error::new(
                        "NOT_EXECUTED",
                        "An earlier sub-operation prevented this write.",
                    ));
                } else {
                    if let Err(error) = self
                        .transmit(
                            &part,
                            &descriptor,
                            &part_action,
                            operation,
                            &mut item,
                            false,
                        )
                        .await
                    {
                        preserve_outcome(&mut item, error);
                        stop = true;
                    }
                    stop |= matches!(item.status, Status::Failed | Status::Unknown);
                }
                item.completed_at = self.clock.now();
                if let Err(error) = self.store.finish_execution(&part, &item) {
                    preserve_outcome(&mut item, error);
                    stop = true;
                }
                items.push(item);
            }
            execution.status = if items.iter().all(|i| i.status == Status::Verified) {
                Status::Verified
            } else if items.iter().all(|i| i.status == Status::Accepted) {
                Status::Accepted
            } else if items.iter().all(|i| i.status == Status::Failed) {
                Status::Failed
            } else {
                Status::Partial
            };
            execution.error = match execution.status {
                Status::Verified => None,
                Status::Accepted if command.wait_for == WaitFor::Accepted => None,
                Status::Accepted => Some(Error::new(
                    "VERIFICATION_PENDING",
                    "The sub-operations were accepted but remain unverified.",
                )),
                Status::Failed => items.first().and_then(|i| i.error.clone()),
                _ => Some(Error::new(
                    "PARTIAL_EXECUTION",
                    "Inspect each non-atomic sub-operation result.",
                )),
            };
            if execution.status == Status::Verified {
                execution.verification = items.first().and_then(|i| i.verification.clone()).map(|mut evidence| {
                    evidence.note = "Each sub-operation has separate verification evidence; inspect items. This is not an atomic or exactly-once guarantee.".into();
                    evidence
                });
                execution.data = Some(command.arguments.clone());
            }
            execution.items = Some(items);
        } else {
            self.transmit(
                command,
                &descriptor,
                &action,
                operation,
                execution,
                approval.is_some(),
            )
            .await?;
        }
        execution.completed_at = self.clock.now();
        self.store.finish_execution(command, execution)
    }

    async fn transmit(
        &self,
        command: &Command,
        descriptor: &Descriptor,
        action: &Action,
        operation: &Operation,
        execution: &mut Execution,
        user_approved: bool,
    ) -> Result<()> {
        let device = &descriptor.device;
        let routes = self.routes(device, command.connection, Some(action));
        execution.error = Some(unreachable(command.connection));
        for connection in routes {
            let progress = WriteProgress::default();
            let start = Instant::now();
            let sent_at = self.clock.now();
            execution.status = Status::Unknown;
            execution.error = Some(uncertain());
            execution.attempts.push(Attempt {
                transport: connection.transport,
                phase: Phase::PossiblySent,
                duration_ms: 0,
            });
            // A crash from this point must not cause replay on the next startup.
            if let Err(error) = self.store.save_execution(command, execution) {
                execution.status = Status::Failed;
                execution.error = None;
                execution.attempts.last_mut().expect("attempt exists").phase = Phase::NotSent;
                return Err(error);
            }
            let validated = ValidatedCommand {
                command: command.clone(),
                user_approved,
            };
            let outcome = operation
                .run(self.adapter(&connection).execute(
                    &validated,
                    &connection,
                    progress.clone(),
                    operation,
                ))
                .await;
            let (phase, error, completion) = match outcome {
                Ok(outcome) => {
                    let phase =
                        if outcome.phase == Phase::NotSent && progress.phase() != Phase::NotSent {
                            progress.phase()
                        } else {
                            outcome.phase
                        };
                    (phase, outcome.error, outcome.completion)
                }
                Err(error) => (progress.phase(), Some(error), None),
            };
            let attempt = execution.attempts.last_mut().expect("attempt was recorded");
            attempt.phase = phase;
            attempt.duration_ms = start.elapsed().as_millis() as u64;
            match phase {
                Phase::NotSent => {
                    execution.status = Status::Failed;
                    execution.error =
                        Some(error.unwrap_or_else(|| unreachable(command.connection)));
                    if execution.error.as_ref().is_some_and(|e| {
                        matches!(e.code.as_str(), "LAN_UNREACHABLE" | "CLOUD_UNREACHABLE")
                    }) {
                        continue;
                    }
                    return Ok(());
                }
                Phase::Rejected => {
                    execution.status = Status::Failed;
                    execution.error = Some(error.unwrap_or_else(|| {
                        Error::new(
                            "DEVICE_REJECTED",
                            "The device explicitly rejected the command.",
                        )
                    }));
                    return Ok(());
                }
                Phase::PossiblySent => {
                    execution.status = Status::Unknown;
                    execution.error = Some(uncertain());
                    return Ok(());
                }
                Phase::Accepted => {
                    execution.status = Status::Accepted;
                    execution.error = None;
                    if command.wait_for == WaitFor::Verified
                        && action.verification == VerificationMethod::CorrelatedCompletion
                        && let Some(evidence) = completion
                        && let Some(verification) =
                            self.verify_completion(command, &connection, evidence, sent_at)
                    {
                        execution.status = Status::Verified;
                        execution.verification = Some(verification);
                        execution.data = Some(command.arguments.clone());
                    }
                    // Completion evidence may already be known before this storage call.
                    self.store.save_execution(command, execution)?;
                }
            }
            if command.wait_for == WaitFor::Verified
                && action.verification == VerificationMethod::StateMatch
            {
                let observation = operation
                    .run(self.adapter(&connection).read(
                        device,
                        &command.component_id,
                        &connection,
                        operation,
                    ))
                    .await;
                if let Ok(mut snapshot) = observation
                    && let Some(verification) = self.verify(
                        command,
                        descriptor,
                        &mut snapshot,
                        connection.transport,
                        sent_at,
                    )
                {
                    execution.status = Status::Verified;
                    execution.verification = Some(verification);
                    execution.data = Some(command.arguments.clone());
                    self.store.save_snapshot(&snapshot)?;
                }
            }
            if execution.status == Status::Accepted && command.wait_for == WaitFor::Verified {
                execution.error = Some(Error::new(
                    "VERIFICATION_PENDING",
                    "The request was accepted, but the required verification evidence is unavailable.",
                ));
            }
            return Ok(());
        }
        Ok(())
    }

    fn verify_completion(
        &self,
        command: &Command,
        connection: &Connection,
        evidence: CorrelatedCompletion,
        sent_at: chrono::DateTime<chrono::Utc>,
    ) -> Option<Verification> {
        if evidence.request_id != command.request_id
            || evidence.device_id != command.device_id
            || evidence.component_id != command.component_id
            || evidence.connection_id != connection.id
            || evidence.received_at < sent_at
            || evidence.received_at > self.clock.now()
            || evidence
                .completed_at
                .is_some_and(|t| t < sent_at || t > evidence.received_at)
        {
            return None;
        }
        Some(Verification {
            kind: VerificationMethod::CorrelatedCompletion,
            source: connection.transport,
            observed_at: evidence.completed_at,
            received_at: evidence.received_at,
            matches_target: true,
            note: "The adapter validated successful completion correlated to this command; this does not establish current state or exactly-once execution.".into(),
        })
    }

    fn verify(
        &self,
        command: &Command,
        descriptor: &Descriptor,
        snapshot: &mut Snapshot,
        source: Transport,
        sent_at: chrono::DateTime<chrono::Utc>,
    ) -> Option<Verification> {
        prepare_live(
            snapshot,
            descriptor,
            &command.component_id,
            source,
            self.clock.now(),
        )
        .ok()?;
        let mut oldest = None;
        for (key, value) in command.arguments.as_object()? {
            let observation = snapshot
                .observations
                .get(&format!("{}.{key}", command.capability))?;
            if observation.stale
                || !matches_target(&observation.value, value)
                || observation.received_at < sent_at
                || observation.observed_at.is_some_and(|t| t < sent_at)
            {
                return None;
            }
            if oldest
                .as_ref()
                .is_none_or(|v: &&Observation| observation.received_at < v.received_at)
            {
                oldest = Some(observation);
            }
        }
        let evidence = oldest?;
        Some(Verification {
            kind: VerificationMethod::StateMatch,
            source,
            observed_at: evidence.observed_at,
            received_at: evidence.received_at,
            matches_target: true,
            note: "Fresh state matches the target; this does not prove request causality.".into(),
        })
    }
}

fn preserve_outcome(execution: &mut Execution, error: Error) {
    // Infrastructure failures cannot erase observation evidence or item outcomes.
    if matches!(
        execution.status,
        Status::Verified | Status::Unknown | Status::Partial
    ) || execution.error.is_some()
    {
        execution.warnings.push(error.message);
    } else {
        execution.error = Some(error);
    }
}

fn unsupported() -> Error {
    Error::new(
        "CAPABILITY_UNSUPPORTED",
        "The device does not support this component or action.",
    )
}
fn uncertain() -> Error {
    Error::new(
        "WRITE_OUTCOME_UNKNOWN",
        "The request may have executed; refresh state without replaying the write.",
    )
}
fn unreachable(route: Route) -> Error {
    Error::new(
        if route == Route::Cloud {
            "CLOUD_UNREACHABLE"
        } else {
            "LAN_UNREACHABLE"
        },
        "No permitted route is available.",
    )
}
