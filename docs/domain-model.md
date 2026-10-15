# Domain model

## Core entities

| Entity | Key fields | Meaning |
|---|---|---|
| Provider | id / region / auth_methods | Account platform, not necessarily a brand |
| Account | id / provider / profile / credential_ref | Local account reference without plaintext credentials |
| Device | id / brand / model / alias / components / connections | Unified identity for one physical appliance |
| Connection | id / adapter / transport / account_id / identity_ref | A potentially available LAN/cloud route |
| Component | id / capabilities | A main unit, fridge compartment, freezer compartment, etc. |
| Capability | id / optional component_id / revision / properties / actions / events | Typed readable state and executable behavior, shared or bound to one component |
| Observation | value / source / observed_at / received_at / stale | Observed state, not a requested target |
| Command | device / component / capability / action / arguments | Brand-independent request |
| Execution | request_id / attempts / status / verification | Outcome from transmission to observation |
| BatchCommand | schema_version / commands | Explicit commands for 1-32 distinct devices, each with its own request ID |
| BatchExecution | status / started_at / completed_at / items | Ordered outcomes from one bounded call; no aggregate persistence or shared device identity |
| Room | id / name / position | Local user-created organization, independent of physical identity |
| DevicePreferences | device_id / alias / room_id / favorite / position | Local presentation metadata that grants no access |
| DeviceCard | device / preferences | One joined presentation record, with the local alias applied |
| RoomAssignment | device_ids / room_id | Atomic local membership change for a selection; explicit null means unassigned |

## Identity

Persist a random internal `device.id`. Do not use an IP, mutable name, or exposed serial number as the internal ID. IDs remain stable within one installation; without synchronization, cross-installation identity equality is not promised. Vendor IDs and MAC addresses are local evidence and must be redacted on export.

The implemented rediscovery guard preserves an existing local ID's complete connection bindings: connection ID, adapter, transport, account ID, and identity reference. Reordering and non-identity metadata updates are allowed; binding changes return `DEVICE_IDENTITY_CONFLICT`. Identity references must remain stable across network-address changes and reauthentication. This guard does not prove physical identity or implement route merging; see [ADR-0013](adr/0013-preserve-device-connection-identity.md).

Deduplicate using strong vendor correlation IDs or verified protocol mappings. Matching names, rooms, or models is not enough for automatic merging. Ambiguous matches become user-confirmed merge suggestions. Namespace identifiers by provider and region where necessary.

## Local organization

Room IDs are generated locally. Names are trimmed, limited to 80 characters, and unique after Unicode lowercasing. Device preferences survive rediscovery. Deleting a room unassigns its devices without removing them or changing their permissions. Null alias restores the adapter's label. Duplicate aliases are allowed, so ambiguous selection requires an exact device ID. Renaming or moving a device never changes its identity, routes, credentials, or capabilities.

Preference updates require explicit `alias`, `room_id`, and `favorite` fields; nullable fields must not be omitted. Room and device reorder requests are complete permutations of current IDs and apply atomically. User-entered names are independent of UI locale. Virtual All/Unassigned filters are not stored rooms. See [ADR-0009](adr/0009-local-organization-and-retention.md).

A RoomAssignment accepts 1-256 distinct exact device IDs and an explicit nullable room ID. It preserves aliases, favorites, and card positions. All references are validated in the transaction; failure changes no memberships. This local transaction is separate from physical batch actions, whose outcomes cannot be rolled back atomically.

## Components

One refrigerator is one Device; the fridge and freezer are Components with their own temperature properties. Do not create two refrigerators merely to reuse a thermostat model. Commands default to `main` only when it exists unambiguously; otherwise reject and list the available components.

Capability definitions may now carry an exact `component_id`. Separate `climate` definitions can describe the fridge's and freezer's different ranges, steps, freshness limits, and actions while commands retain the same semantic capability ID. Omission keeps a shared definition for all referencing components. Mixing shared and scoped definitions of the same ID is invalid; there is no override or inheritance rule. Each reference resolves exactly once through `Descriptor::capability(component_id, capability_id)`. Null scope, duplicate keys, nonexistent components, and unreferenced scoped definitions are rejected. See [ADR-0018](adr/0018-component-capability-scopes.md) and the [fictional refrigerator](../examples/device.refrigerator.json).

## Capabilities and units

Property types and units must be machine-readable. Core temperature values use `celsius`; explicitly convert other input units at the entry point and validate the converted value against device step constraints. Do not pass Fahrenheit as Celsius. Stable IDs do not change with display language. Repository examples and baseline interface text are English.

## State and time

`observed_at` is a trustworthy observation time when available. If only local receipt time exists, use `received_at` and identify the source timestamp as unavailable. Do not invent a device clock value. Connectivity and last-known state are separate. Missing observations are unknown/absent, not fabricated zero or false values.

Compute `stale` when reading cache and state why an entry is stale. Desired/requested values must not overwrite observed values and be presented as confirmed execution.

The implemented MVP checks complete snapshot identity, version, source, timestamp consistency, and component-specific readable property values before caching or verification. Bounds and steps come from property descriptors, including read-only and vendor properties. Missing readings are omitted; the current capability schema does not support nullable property values. Each property's optional `max_age_ms` defines its age limit, defaulting to 30 seconds for existing metadata. Core uses the source observation time when available, so receiving an old value again does not make it fresh. Age equal to the limit is stale; existing stale flags remain true. A cached future receipt is retained as stale with a clock warning; an invalid live receipt is rejected. See [implemented observation rules](routing-and-execution.md#implemented-mock-behavior). Real-device freshness requirements still need adapter-specific evidence.

## Versioning

`schema_version` versions the overall contract; `capability.revision` versions a device capability description. Optional additive fields can be compatible. Removing fields, changing units, or redefining enum meanings is breaking. Old clients may display unknown capability metadata but must not execute unfamiliar high-risk actions.

The Property / Action / Event model takes inspiration from W3C WoT. This project is not a complete WoT Thing Description implementation and claims no certification. [R2](references.md)
