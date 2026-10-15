# ADR-0047: Resolve LAN credentials from a protected Meiju account binding

Status: accepted for synthetic implementation within the additional-LAN scope.

## Evidence and decision

The pinned MIT `midea-local` revision
`ef28169926914d15f40fe521b9cc3d3c2f553c59`, `cloud.py` and `security.py`, uses
`/v2/iot/secure/getToken` for Meiju. Its request includes a string `homegroupId`,
a one-element string array `applianceCodes`, and `udpid`, plus the upstream
general fields: `src`/`appId` 900, `format` 2, `platformId`/`clientType` 1,
`language` en_US, null `uid`, request ID, UTC stamp and the login client `deviceId`.
Preserve the exact client ID sent by HomeVerb's existing login codec; do not
switch that login algorithm to the LAN source's different client-ID hash.
Derive the UDP ID by
SHA-256 over the nonzero six-byte appliance ID, XOR the digest halves, and encode
lowercase hex. The trusted native caller explicitly selects big or little endian
(upstream methods 1 and 2). Do not copy upstream's v1 fallback, default keys,
credential cache, automatic method switching or token logging.

Add a native `MeijuAccount::lan_v3_handshake` operation taking the exact local
DeviceBinding and UDP-ID method. Check the protected account session, mainland
region, all three local binding IDs and last complete inventory membership before
network access. Admit only the selected AC/E1/CA families and six-byte IDs.
Neither a caller-provided provider ID nor a discovery candidate selects keys.

The protected payload does not retain home IDs. Resolve the selected appliance's
home through fresh authenticated home and appliance listings within the same
Operation. Cap this lookup at 32 homes, fail on any incomplete listing, require
exactly one occurrence, and compare family, model and retained product evidence
against the protected binding. Reject ambiguity even when duplicate entries
otherwise agree. Never pick the first matching home. Do not update account
inventory or replace historical evidence as a side effect of key retrieval.

Only then issue one v2 credential request. Require exactly one matching `udpId`
entry, a 64-byte token and 32-byte key encoded as hex. Unknown provider fields are
ignored; malformed, duplicate or oversized responses fail with fixed errors.
Decode secret strings through zeroizing ownership and return only an opaque
`V3Handshake`. Do not persist the credentials, expose them in JSON, or send them
to a LAN endpoint. This explicit operation requires cloud connectivity; it does
not implement offline onboarding or a reusable LAN credential cache.

## Protected session compatibility

New successful logins save protected inner session version 2 with the original
16-character lowercase hex `client_device_id`. This is a client identifier,
not an appliance ID, account subject or authorization proof. Keep it in protected
memory and the secret store only. Continue reading and encoding legacy version-1
sessions without inventing an ID or silently rewriting them. Existing cloud reads
and controls work with both formats. Legacy sessions cannot use the new credential
lookup until explicit same-account reauthorization supplies the missing client
ID; fail before network access with `AUTH_REQUIRED`. Never prompt or log in
automatically. Validate version/field combinations and reject unknown versions.
Older libraries refuse version 2; downgrade requires an explicit supported
reauthorization flow, never deletion or reset of the protected binding map.

## Boundaries and verification

Native JSON/C/JNI contracts, SQLite schema 4 and outer account payload version 3
are unchanged. Existing account refresh and cloud controls do not call this helper.
The lookup performs no scan, TCP connection, appliance state query, control,
login, retry or background work. One inherited deadline/cancellation covers every read and the
credential request. Transport errors and provider messages remain sanitized.

Account-scoped credentials and V3 key confirmation do not establish the identity
of a discovered endpoint or grant appliance permissions. Protected endpoint
reconciliation, model-specific inner packets, route registration and physical
acceptance remain gates before a real LAN route can use these credentials.

Use scripted fictional account transport with the real account vault and SQLite
to check request shapes, both UDP methods, subject/binding/membership preflight,
unique home resolution, conflicting and partial inventory, token selection,
lengths/hex, failure/cancellation/deadline, unchanged storage and redacted output.
Test version-1 cloud compatibility and no rewrite, version-2 round trips and
client-ID validation, same-account reauthorization with preserved local bindings,
and absence of the new private identifier from SQLite and public responses.
No real account or household appliance is used.
