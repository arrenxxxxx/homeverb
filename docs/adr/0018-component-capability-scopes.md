# ADR-0018: Scope capability definitions to device components

Status: accepted for the unreleased Core MVP.

## Decision

Add optional `component_id` to a Capability definition. Omission retains the existing shared definition for every component that references that capability ID. A present value binds the complete definition to one exact component of the same device. It is not an override, inheritance layer, command alias, or new device identity.

Definitions are unique by `(component_id, id)`. A device may contain separate `climate` definitions for its fridge and freezer, each with its own properties, argument schemas, routes, risk, verification method, revision, and freshness limits. If any definition of an ID is scoped, every definition of that ID must be scoped. Mixing a shared and scoped definition of the same ID is rejected, including when their target components differ. A scoped definition must name an existing component that references its capability ID. Every component reference must resolve exactly once. Duplicate definitions, missing scope bindings, and ambiguous property names fail descriptor admission.

`Descriptor::capability(component_id, capability_id)` is the shared unique-definition resolver. Core uses it for command/action validation and observation preparation, including cached reads and verification. Commands still identify one physical device, one component, and one semantic capability ID. Grants remain component-specific and do not transfer between compartments. Device locks and identity remain device-wide; snapshots remain keyed by device and component.

## Compatibility

Existing shared definitions omit the new field and retain their behavior and JSON representation. Explicit null is invalid, not an alias for shared scope. Adding the optional Rust field requires existing Capability struct literals to use `component_id: None`. SQLite schema and C ABI functions are unchanged because descriptors are already stored and returned as JSON.

Older strict capability readers reject scoped definitions. They must not strip the field or choose the first matching capability ID. Updated consumers resolve definitions for the selected component; no brand-based logic is needed. Existing global duplicate-ID fixtures must remain invalid. The change does not enable a real refrigerator adapter or alter trusted Mock grants.

## Verification

Fictional fridge/freezer fixtures exercise different temperature ranges, steps, and freshness limits under the same `climate` ID. Tests cover reads, cache, write validation/verification, component-specific grants, malformed discovery without partial saves, persisted invalid metadata, shared legacy definitions, and CLI descriptor/cache behavior. All physical-device compatibility rows remain unverified.
