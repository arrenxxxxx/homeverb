# ADR-0025: Validate protected Meiju bindings before Core state reads

Status: accepted for offline native integration. Live authentication remains gated by ADR-0022.

## Decision

Add `MeijuAccount::read_bound_state` as the adapter-facing default state path. Under the existing account lease, load the protected map, locate the exact local device ID, and compare the selected cloud connection's ID, adapter, transport, account and identity reference with the stored binding. Require that connection in the supplied device, preserve known model evidence, and check the selected component against both the device and supported decoder family. Reject mismatches before any transport request. Names and brands do not select identity or behavior.

Construct the private appliance target from the stored mapping, make one default cloud query using the original Operation, and project the response onto that exact local device/component. There are no additional queries, retries or capability discovery. Core remains responsible for descriptor admission and full property validation before caching. Undeclared properties or malformed data preserve the previous cache; stale cloud readings never establish write verification. The low-level decoder projection alone still does not validate identity.

## Native concurrency contract

Require `AccountLock: Send + Sync`. Its purpose is to retain a native account lock while asynchronous secure-store and adapter operations run. The previous Send-only trait prevented immutable leased account reads from satisfying the existing Send future requirement of `Adapter::read`. The current SQLite file-lock implementation supports both bounds. This is a source-level constraint change for custom implementations of the unreleased Rust trait; they must supply a thread-safe lifetime guard. No locking policy, public native JSON, C ABI or SQLite schema changes.

## Evidence and limits

A test-only adapter carries fictional AC/E1/CA descriptors through real Core admission, bound reads, complete observation validation and SQLite caching. Tests cover account/connection/model/component mismatch before sending, conservative freshness, separate fridge/freezer snapshots, malformed and undeclared readings, unchanged cache, unavailable/pending credentials, cancellation/deadline handling and account-lock release. Read-only fixtures declare no actions and cannot authorize controls.

This phase does not admit actual model capabilities, register a live adapter, authenticate an account, or implement cloud controls. Those remain required for the real-device journey. Native UI API 1.0.0 and its 22 methods are unchanged.
