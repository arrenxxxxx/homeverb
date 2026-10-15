# ADR-0020: Retrieve Meiju account inventory through a bounded read flow

Status: accepted for the first Midea protocol implementation.

## Decision

Add the in-process `homeverb-midea` library, beginning with the Meiju cloud account inventory protocol. Use the pinned community client in [the integration guide](../development/05-midea.md) as wire-format evidence. Implement the actual code/data envelope, home listing, nested appliance listing, device-family classification, and one bounded account-scoped retrieval flow. Do not register a live adapter until native authentication, request signing and HTTPS transport are established.

A trusted transport receives only the two reviewed read endpoint variants and the caller's Operation. The inventory layer never retries, refreshes a session, executes downloaded protocols, sends controls, or starts background tasks. Authentication failure stops remaining home requests. Individual home failures produce an explicitly partial result; a failed top-level home list is not an empty inventory. Cancellation and deadline expiry stop retrieval and return the corresponding error without persisting a partial replacement.

[ADR-0023](0023-meiju-cloud-state-evidence.md) subsequently adds a separate appliance-state endpoint to the shared read transport. Inventory retrieval remains restricted to its original two endpoints and does not query device state implicitly.

Vendor identifiers and response bodies are private native data, not serializable UI DTOs. Inventory is tied to one opaque local account reference. Equal appliance IDs can deduplicate only within that account; contradictory family/model evidence fails rather than selecting a winner. A complete result describes the finite response received, not permanent account completeness or a verified physical-device state. Device online hints are separate from timestamped state observations.

Recognized AC/E1/CA types are candidate families, not enabled capabilities. Preserve other returned types for an honest unsupported-device UI later. Missing/null optional metadata stays unknown; missing required listing containers, invalid IDs, malformed envelopes and oversized responses are errors. Provider messages and raw bodies never appear in errors or Debug output.

## Compatibility and next integration

This adds a Rust library without changing the native UI API, Core execution semantics, database, or registered adapters. A transport seam is necessary to verify the real response protocol before using accounts and to inject reviewed HTTP later; it is not a Python service or HA dependency. No real-platform login or control is claimed.

The next integration must persist a random account namespace and encrypted identity mapping before admitting inventory into Device descriptors. Secure sessions, native authentication, signed HTTPS, model-specific capability mapping, UI account methods and physical acceptance remain required. Partial/error inventories must never remove existing cards or overwrite their organization.

## Verification

Use synthetic responses with the inspected provider field names. Cover nested multi-home enumeration, duplicates, conflicting identities, unknown families/status hints, malformed and rejected responses, mid-flow session expiry, cancellation/deadline, request bounds, no retries, and redacted Debug/errors. No owner credentials or physical devices are involved in these tests.
