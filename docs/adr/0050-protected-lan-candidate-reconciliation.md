# ADR-0050: Reconcile LAN hints against one protected account binding

Status: accepted for synthetic implementation within the additional-LAN scope.

## Decision

Add native `MeijuAccount::reconcile_lan_candidate(selected, network, report,
operation)`. Load the selected account's protected state under its existing
lease. Require the exact local device, connection and identity references,
known listed membership, mainland region, AC/E1/CA family and a six-byte
provider appliance ID. Do not accept a caller-selected provider identifier.
This uses saved membership, not fresh cloud authorization or peer authentication.

Check the report's existing collector bounds and counters before considering
matches. All endpoints must be usable peers in the explicit network, with
nonzero ports and six-byte IDs. A native caller can construct a report, so
these checks establish input consistency only, never collection provenance.

Return a native, nonserializable `LanCandidateMatch`:

- `Incomplete` when collection reached a limit, even if one matching hint exists.
- `NotObserved` when an unlimited report contains no hint for the selected ID;
  this is not absence from the account or proof of an offline device.
- `Ambiguous` for a conflicting family, endpoint or protocol for the selected
  ID, or another ID claiming the selected endpoint. Never choose by arrival order.
- `UniqueCandidate` borrowing the matching hint when all selected-ID hints are
  identical, have the protected family and have no endpoint conflict. Exact
  duplicates do not create ambiguity. Unrelated conflicts do not admit or block
  the selected device. V2 and V3 remain hints, not supported routes.

## Boundaries

No sockets, cloud calls, credential lookup, new IDs, persistence, card publication,
route registration, model admission, observation or write outcome are added.
An unlimited five-second report cannot establish global uniqueness or freshness.
The result borrows a report hint and has no serialization implementation;
trusted native callers must still avoid exporting private values through other means.
Matching public identifiers does not authorize sending a token to that endpoint.
Model/profile evidence, explicit network consent and authenticated endpoint
admission remain separate requirements. Cloud-only model evidence is not LAN
compatibility evidence. Existing descriptors keep their cloud connections.

The API is additive within the Rust adapter. Native JSON API 1.14.1, C/JNI
exports, SQLite schema 4 and protected account/session formats remain unchanged.
There is no new public error code: invalid reports use `INVALID_ARGUMENT`,
incorrect local bindings use `DEVICE_IDENTITY_CONFLICT`, and the existing
membership, credential, region, family and lifecycle errors remain applicable.

## Verification

Use fictional protected accounts and discovery hints. Cover all result variants,
exact duplicate and reordered hints, different identities at one endpoint,
multiple endpoints/protocols/families for one identity, unrelated conflicts,
invalid scopes and bounds, membership changes, cross-account local references,
unsupported IDs/families, cancellation and deadline. Compare protected bytes,
store writes and existing cards before and after reconciliation. Check redacted
Debug and a compile-fail serialization example. No household network is accessed.
