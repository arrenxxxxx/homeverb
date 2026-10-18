# ADR-0073: Stable Meiju subjects and explicit legacy account recovery

Status: implemented for owner-requested same-account recovery; synthetic validation passed, live restoration pending.

## Evidence and decision

An owner-operated Android login passed provider authentication with the same
login-derived client identifier, but returned a different login ID. Exact subject
comparison then refused replacement. The pinned official APK uses `iotUserId`
for login state and separately uses `loginId` for password processing. These are
different fields; neither matching input spelling nor device membership proves
that two provider subjects identify the same person.

Fresh native sessions use a bounded, nonempty `iotUserId` from successful HTTPS
login when present, under `meiju.iot-user-id.v1:`. Protected session version 3
retains the login-ID alias for migration when that alias still exactly matches
the historical binding. Missing canonical identity retains legacy behavior;
it never permits a canonical binding to fall back to login-ID matching.
Ordinary reauthorization still rejects changed identity. New canonical sessions
can reauthorize despite changed login IDs if their IoT user IDs match exactly.

Legacy sessions with changed login IDs require a separate trusted native recovery
preview, confirmation and discard path. Preparation consumes protected credentials,
validates an intact legacy version-2 binding, authenticates once, requires a
canonical IoT user ID and the same login-derived client identifier, and retrieves
a complete inventory. Every saved appliance must still be present with the same
family, model and compatible protected product evidence. Empty, partial, missing,
extra or conflicting inventories refuse recovery. These checks constrain scope;
they are not automatic proof of account identity. The owner must confirm that this
is the same account in the trusted UI after seeing the retained account and device
count. A shared household could satisfy inventory matching, so ordinary JSON,
CLI, MCP and automatic retries cannot confirm this reconciliation.

The immutable preview owns protected old bytes, the fresh session and inventory,
expires after 60 seconds, and is bound to its native host. Background, close,
discard and cancellation invalidate it. Confirm reacquires registration/account
leases, checks exact original bytes, rejects a canonical subject already stored
in another account, and preserves all local account/device/connection identifiers
and preferences. It replaces the protected subject/session through the existing
pending-write protocol; uncertainty is not rolled back or replayed. The confirming
host closes and the App reopens read-only, so old in-memory command authority and
approvals do not survive the identity transition. No appliance command is sent.

## Compatibility and validation

Native API 1.25 adds three trusted C functions; general JSON methods and SQLite
schema 5 remain unchanged. The outer protected payload remains version 3. Older
native code refuses the new session encoding; it must not reset those accounts.
The legacy reconciliation is an explicit exception to ADR-0031's exact-subject
rule, requiring actual user-side confirmation and the evidence above.

Required checks cover canonical stability/conflict, legacy alias migration,
missing/malformed identity, full inventory scope, byte changes, duplicate account
subjects, expiration, one-shot/lifecycle consent, uncertain persistence, credential
erasure and absence of private values in public previews. Live authentication and
appliance acceptance must be recorded separately from synthetic tests.
