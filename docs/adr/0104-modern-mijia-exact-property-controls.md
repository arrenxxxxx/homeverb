# ADR-0104: Modern Mijia exact property controls

Status: Accepted; implemented with fictional software evidence.
Date: October 30, 2026.

## Evidence and scope

The digest-pinned October 22 reference contract statically recovers POST
/miotspec/prop/set with params containing did, siid, piid and a specification-typed
value. It explicitly does not establish runtime firmware/plugin conversion or
physical execution. ADR-0094 establishes the modern core.api.mijia.tech origin,
continuous RC4-drop1024 encoding and bounded encrypted response profile. This
implementation combines those protocol facts with the independently admitted
atomic scalar semantics in ADR-0078, ADR-0083, ADR-0084, ADR-0091 and ADR-0092.

Only the existing reviewed exact model/spec/property/domain mappings are admitted:
reference fan, light, purifier, outlet and hood illumination. Source write access
alone does not admit additional properties, MIoT actions, composites, timers,
lock writes or arbitrary RPC. New provider execution remains experimental until
model-specific physical acceptance. No real write is authorized by this ADR.

## Decision

Add a separate fixed-origin modern control transport, independent of the read
transport. Construction is inert. Its one protected request uses only the modern
session, exact root identity and continuous reference codec. No legacy or community
history origin, codec or credentials are a fallback. Mark WriteProgress immediately
before possible network transmission, after local request admission/building.
After this boundary, HTTP/global errors, malformed responses, transport failure,
cancellation and timeout remain possibly sent; they cannot prove rejection.

Require one encrypted acknowledgement with code zero globally and exactly one
matching did/siid/piid result. Item code zero is accepted, never verified; a matching
nonzero item code is explicitly rejected. Plain JSON success, unrelated/duplicate
items and missing tracking evidence cannot establish acceptance. Raw and decoded
responses are bounded to 1 MiB, including gzip, with no trailing members. Never
retry, refresh credentials, choose another route or send companion properties.
A later state read retains independent stale/cloud evidence, not command causality.

The adapter requires explicit modern transport composition plus its existing
family opt-ins, exact Core grants and protected identity/model/spec/domain checks.
Default hosts remain read-only. Production native hosts register the transport
but do not thereby acquire any grant. Local option inspection may expose the
reviewed modern choices without networking, descriptor mutation or authority.
After trusted grant preparation, explicit account retrieval publishes supported
actions. A host without the modern transport still refuses stale grants/descriptors.

Execution independently re-derives protected risk and control family. Elevated
outlet power, recovery and physical-button settings require Core's trusted approval
marker regardless of public metadata, as in ADR-0091. Logical split children never
become independent command targets and no device suffix is stripped. Unknown or
removed membership, mismatched route/subject/model and unsupported arguments refuse
before send. Saved binding protocol selects legacy or modern before I/O; merely
adding a modern login slot never switches legacy controls.

Core receipt replay, lifecycle locks and immutable approval consumption remain
unchanged. Native API 1.37, ordinary schemas, exports, SQLite 5 and Mijia payload 3
are unchanged. Existing native/App control-option and grant contracts carry these
exact properties without provider branching in UI. Older binaries keep their
modern-control refusal. Packaging and physical acceptance are separate gates.

## Required evidence

Independent fictional continuous-stream request decoding/signature checks; all
acknowledgement outcomes, correlation errors, bounds, gzip, pre/post-send failures,
late cancellation and no retry. Local TLS checks must establish headers, fixed
origin, certificate/redirect refusal, the conservative send boundary and socket
release. Core tests must cover every admitted exact option, default/absent grants,
transport omission, family separation, tampered public risk, actual trusted outlet
approval, uncertain receipt replay, account/device leases and legacy coexistence.
Native Host option/plan/execute/approval and generic App regressions must preserve
existing lifecycle behavior. No provider account or appliance is used in tests.

## Executed verification

On October 30, 2026, the Rust 1.96 offline/locked workspace test run excluding
homeverb-midea passed 618 tests with one ignored. The excluded suite retains its
known unrelated macOS LAN-alias environment failures; it is not claimed passing.
The modern control additions include four protocol tests, six Core tests, two local
TLS tests and two Host tests. Core covers all 29 fields, all enumerated/Boolean
values and numeric boundaries. A native test helper initially supplied unequal
outer/command timeouts; it was corrected to honor the existing Host contract, and
both Host control tests then passed. Production timeout validation is unchanged.

Strict workspace/all-target Clippy and the rebuilt FFI library passed. The actual
C ABI verifier passed 36 ordinary methods/57 examples and its trusted dedicated
boundaries, including 76 modern-account cases. Existing generic Flutter control,
permission, approval, history and modern-device-flow regression passed 125 tests
across the nine files listed in docs/testing.md. This is a scoped pass; the known
eight macOS full-suite font/pixel/legacy-click failures are not claimed fixed.

No public schema or API-version change is required: only existing inspected
adapter options gain modern admission. Executable positive/negative fixtures live
in modern/reads/tests/writes.rs, adapter/tests/modern_accounts/controls.rs and
crates/host/tests/mijia_accounts/modern_host/controls.rs, with local TLS cases in
crates/mijia/src/transport/tests.rs. Existing ordinary schema fixtures remain valid.
No real provider account, appliance operation, phone test, package installation,
remote push or publication was performed. The dd51c73 APK does not contain this
phase; new packaging and physical acceptance remain separate work.

Final repository checks passed: workspace formatting, documentation links, source
hygiene, 25 schemas with 69 positive/1196 negative fixtures, digest-based coverage
regeneration and both reference-projection unit tests. These gates do not expand
the physical or provider evidence scope above.
