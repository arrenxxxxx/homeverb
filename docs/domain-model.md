# Domain model

## Core entities

| Entity | Key fields | Meaning |
|---|---|---|
| Provider | id / region / auth_methods | Account platform, not necessarily a brand |
| Account | id / provider / profile / credential_ref | Local account reference without plaintext credentials |
| Device | id / brand / model / alias / components / connections | Unified identity for one physical appliance |
| Connection | id / adapter / transport / account_id / identity_ref | A potentially available LAN/cloud route |
| Component | id / capabilities | A main unit, fridge compartment, freezer compartment, etc. |
| Capability | id / revision / properties / actions / events | Typed readable state and executable behavior |
| Observation | value / source / observed_at / received_at / stale | Observed state, not a requested target |
| Command | device / component / capability / action / arguments | Brand-independent request |
| Execution | request_id / attempts / status / verification | Outcome from transmission to observation |

## Identity

Persist a random internal `device.id`. Do not use an IP, mutable name, or exposed serial number as the internal ID. IDs remain stable within one installation; without synchronization, cross-installation identity equality is not promised. Vendor IDs and MAC addresses are local evidence and must be redacted on export.

Deduplicate using strong vendor correlation IDs or verified protocol mappings. Matching names, rooms, or models is not enough for automatic merging. Ambiguous matches become user-confirmed merge suggestions. Namespace identifiers by provider and region where necessary.

## Components

One refrigerator is one Device; the fridge and freezer are Components with their own temperature properties. Do not create two refrigerators merely to reuse a thermostat model. Commands default to `main` only when it exists unambiguously; otherwise reject and list the available components.

## Capabilities and units

Property types and units must be machine-readable. Core temperature values use `celsius`; explicitly convert other input units at the entry point and validate the converted value against device step constraints. Do not pass Fahrenheit as Celsius. Stable IDs do not change with display language. Repository examples and baseline interface text are English.

## State and time

`observed_at` is a trustworthy observation time when available. If only local receipt time exists, use `received_at` and identify the source timestamp as unavailable. Do not invent a device clock value. Connectivity and last-known state are separate. Missing observations are unknown/absent, not fabricated zero or false values.

Compute `stale` when reading cache and state why an entry is stale. Desired/requested values must not overwrite observed values and be presented as confirmed execution.

## Versioning

`schema_version` versions the overall contract; `capability.revision` versions a device capability description. Optional additive fields can be compatible. Removing fields, changing units, or redefining enum meanings is breaking. Old clients may display unknown capability metadata but must not execute unfamiliar high-risk actions.

The Property / Action / Event model takes inspiration from W3C WoT. This project is not a complete WoT Thing Description implementation and claims no certification. [R2](references.md)
