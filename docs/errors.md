# Errors and result contract

## Error envelope

```json
{
  "code": "WRITE_OUTCOME_UNKNOWN",
  "message": "The request may have executed; no reliable acknowledgement was received.",
  "retryable": false,
  "suggested_action": "refresh_state",
  "details": {"stage": "sent", "transport": "lan"}
}
```

`message` is human-readable English; `code` is machine-readable. `details` contains only redacted allowlisted fields, never raw responses, passwords, or device keys. The executor determines `retryable` from the operation context; not every network error is retryable.

## Error categories

| Code | Typical cause | Response |
|---|---|---|
| AUTH_REQUIRED | Login or user consent required | Start login through a trusted UI |
| AUTH_EXPIRED | Session cannot be refreshed safely | Request reauthorization |
| AUTH_FLOW_UNSUPPORTED | Authentication requires an excluded backend | Mark blocked; do not add a hidden backend |
| INPUT_UNAVAILABLE | The CLI cannot provide protected interactive terminal input | Use an interactive Linux terminal; never pass credentials as arguments or pipe input |
| ACCOUNT_NOT_FOUND | Unknown local account reference | Reload local account records; do not select another account implicitly |
| ACCOUNT_REMOVAL_PENDING | Account cleanup is incomplete; ordinary credential use or late publication is refused | Finish explicit local cleanup; never reconnect or restore the old account automatically |
| ACCOUNT_SCOPE_CHANGED | The device set differs from the reviewed removal scope | Reload local details and obtain a new user confirmation before removal |
| ACCOUNT_REMOVAL_UNSUPPORTED | A linked card has routes outside this account/provider | Preserve all data; shared-route detachment needs a separate supported identity flow |
| ACCOUNT_SESSION_UNCONFIRMED | Credential persistence was not finalized, recovery bytes differ, or protected account state is invalid | Offer saved-session recovery or explicit same-account reauthorization when protected identity is intact; preserve data on validation failure, never reset automatically |
| ACCOUNT_IDENTITY_CONFLICT | Reauthorization identifies a different provider account | Keep the original session and device bindings; use the original account or a separately reviewed new-account flow |
| SECRET_STORE_UNAVAILABLE | Secure credential storage unavailable | Fail explicitly; no plaintext fallback |
| SECRET_STORE_LOCKED | The default collection or matching item is locked before mutation | Unlock it through the operating system; HomeVerb does not call Unlock |
| SECRET_STORE_CONFLICT | Multiple entries match the exact profile and credential reference | Stop without reading, replacing, or deleting ambiguous entries |
| SECRET_STORE_WRITE_UNKNOWN | A credential mutation started but did not return a confirmed result | Inspect the exact reference before recovery; no automatic retry |
| DEVICE_NOT_FOUND | Unknown device ID | List local records or rediscover |
| DEVICE_IDENTITY_CONFLICT | Rediscovery changes an established device connection binding | Preserve the existing identity; do not copy grants, replace bindings, or retry automatically |
| AMBIGUOUS_DEVICE | Duplicate alias | Require an exact ID |
| ROOM_NOT_FOUND | Unknown local room ID | Reload rooms before assigning or editing |
| ROOM_NAME_CONFLICT | Room name already exists after lowercasing | Choose a distinct name |
| CAPABILITY_UNSUPPORTED | Unsupported action/component, invalid descriptor shape/version/references or capability scopes, invalid property freshness limits, contradictory field-decomposition declarations, or an action schema that cannot compile locally | Stop before adapter calls; rediscover valid metadata or use supported capabilities |
| INVALID_ARGUMENT | Invalid range/enum or caller command identifier that is blank or exceeds 256 UTF-8 bytes | Correct using the schema and runtime identifier limits |
| POLICY_DENIED | No complete matching grant/risk/route authority; ordinary calls deny elevated/high risk, and typed approval still denies high risk, automatic/ambiguous routes and decomposition | Trusted host policy change; no model-supplied bypass |
| APPROVAL_EXPIRED | The typed trusted preview reached its 60-second monotonic expiry before execution | Review a new preview and obtain fresh user consent; never auto-confirm |
| APPROVAL_INVALID | The prepared descriptor/Core/generation changed, or the native approval ID is missing, discarded, consumed, foreign or pruned | Review the current scope; no send occurred |
| LAN_UNREACHABLE | LAN unavailable and write proven unsent | Use a reachable network |
| CLOUD_UNREACHABLE | Cloud unavailable and write unsent | Check network/vendor availability |
| RATE_LIMITED | Vendor rate limit | Honor budget and return retry guidance |
| RESOURCE_BUSY | Local account/device operation lock conflict | Report conflict; bounded waiting |
| DEVICE_REJECTED | Explicit device rejection | Failed with the known reason |
| WRITE_OUTCOME_UNKNOWN | Possibly sent; acknowledgement missing | Unknown; no automatic replay |
| VERIFICATION_PENDING | Accepted but required state evidence is missing, stale, or different, or correlated completion evidence is missing or fails identity/time checks | Accepted; do not resend or substitute another verification method |
| PARTIAL_EXECUTION | Different sub-operation outcomes | Return per-item outcomes |
| CANCELLED | User cancellation | Report whether transmission may have occurred |

After possible transmission, prefer `WRITE_OUTCOME_UNKNOWN` over a misleading `LAN_UNREACHABLE`. See [CLI exit codes](cli.md).

## Invariants

`verified` includes verification evidence. `failed` includes an explicit error. `unknown` explains uncertainty and must not suggest blind retries. Batch aggregation preserves every item; `partial` never hides successes or failures. Contract version changes require explicit coordinated documentation and migration; do not silently treat an incompatible envelope as valid.

`APPROVAL_EXPIRED` and `APPROVAL_INVALID` are pre-send failures with no attempts
or verification. They belong to the typed Rust path and separate trusted C
functions in [ADR-0043](adr/0043-native-trusted-approval-bridge.md), never ordinary
JSON/CLI/MCP. Native missing/dismissed/consumed/foreign/pruned IDs return a query
error before execution admission; a retained expired preview returns an Execution.
Preparation at eight pending previews returns `RESOURCE_BUSY` until dismissal or
expiry pruning. Decoded confirmation consumes a matching preview even on schema
or operation-admission failure; malformed undecodable input does not. A
cancelled/expired in-flight send still uses existing uncertainty semantics,
not an approval-refusal claim. See [ADR-0042](adr/0042-trusted-single-command-approval.md).

The separate BatchExecution envelope carries status, timestamps, and per-device Execution items; error and verification fields belong to those items. Invalid batch shape, duplicate device/caller IDs, missing permissions, unavailable routes, or an existing conflicting request fail whole-list preflight without sending. Collisions involving derived composite IDs use `REQUEST_CONFLICT`. Failures after admission remain per item, and independent devices continue within the shared deadline. A mixed batch returns `partial` even when different items met different requested wait conditions. See [batch semantics](routing-and-execution.md#composite-and-batch-operations).

## Additional implemented host and storage errors

| Code | Meaning |
|---|---|
| DEADLINE_EXCEEDED | The shared budget expired before a confirmed send |
| CACHE_MISS | No cached observation exists |
| INVALID_OBSERVATION | A live/cached snapshot has an unsupported version, wrong identity, invalid warning/chronology, or a value that violates its component's readable property contract; live reads also reject cache flags, foreign transports, and future receipt times |
| REQUEST_CONFLICT | A request ID already belongs to another command |
| REQUEST_NOT_FOUND | No receipt or retired-ID marker exists in this local profile at lookup time; this does not prove non-execution |
| REQUEST_RETIRED | This ID was recorded, but detailed history was retired; no new send occurred and the earlier physical outcome is unavailable |
| NOT_EXECUTED | An earlier composite step or storage error stopped further writes; this item has no transmission attempts |
| STORAGE_ERROR / MOCK_STORAGE_ERROR | Local persistence could not complete; no raw database details are exposed |
| STORAGE_VERSION_UNSUPPORTED | A newer format or unversioned foreign database must not be opened for writing |
| HOST_BACKGROUND / HOST_CLOSED | The native host rejects new work in its current lifecycle state |
| INTERNAL_ERROR / MCP_IO_ERROR | Host execution or stdio transport failed |

Presentation resources translate stable codes; Core messages remain English. Unknown codes use a localized generic message and preserve the code for diagnosis.

If persistence fails after verification or while the result is already unknown or partial, optional English `warnings` preserve the infrastructure failure without replacing the stronger outcome. A verified result still has `error: null` and its original evidence, even if saving the observation failed. Existing rejection/cancellation errors are retained too. Composite items remain present and later unsent items keep `NOT_EXECUTED`, even if finalizing their receipt fails. The UI must display these warnings. The saved journal may still contain the earlier conservative receipt.

Receipt lookup reports query failure for `REQUEST_NOT_FOUND` or `REQUEST_RETIRED`; neither establishes the physical outcome. A found receipt is query success even when the nested command status is failed or unknown.

`REQUEST_RETIRED` is a failed new invocation, not proof that the original command failed. Display a refresh action and never automatically assign a new request ID and resend. `RESOURCE_BUSY` also covers exclusive migration access; close other profile users before upgrading.

Secret-store errors contain no raw backend error text. Before a credential mutation starts, cancellation and deadlines retain `CANCELLED` / `DEADLINE_EXCEEDED`. After it starts, any unconfirmed failure is conservatively `SECRET_STORE_WRITE_UNKNOWN`. Missing entries return `None` on read; deleting an absent reference succeeds. Local credential deletion never establishes vendor revocation. See [native credentials](storage.md#native-credential-storage).

## Meiju native authentication errors

`MIDEA_LOGIN_INPUT_UNSUPPORTED` rejects nonempty/length/ASCII credential violations
before networking. The admitted community codec cannot claim untested Unicode
password support. Login uses the fixed read-protocol error codes below for invalid
responses, explicit rejection and transport failure. Unknown login challenges are
rejected without bypass or retry. No provider message, account spelling, password
hash, URL, token or session key is included. Invalid protected session encodings
return `ACCOUNT_SESSION_UNCONFIRMED`; no session is fabricated or reset.

## Meiju cloud read errors

`MIDEA_INVENTORY_PARTIAL` means the Core adapter could not retrieve every accessible
home. A recognized expired-session failure instead preserves `AUTH_REQUIRED`. Discovery publishes none of that adapter's current batch; existing cards,
preferences and caches remain. An explicit later retry may succeed. Private
bindings already saved during retrieval remain stable. Duplicate appliance bindings
across accounts, including historical bindings missing from today's inventory,
return `DEVICE_IDENTITY_CONFLICT`; no implicit merge or new duplicate card is allowed.
An unsupported or unreviewed exact model returns `CAPABILITY_UNSUPPORTED` before a
state request. Public model admission that disagrees with the protected model/family
returns `DEVICE_IDENTITY_CONFLICT` before a request.

The native read protocols use `MIDEA_RESPONSE_INVALID` for malformed/oversized, wrong-home/appliance responses or malformed recognized state fields, `MIDEA_REQUEST_REJECTED` for explicit nonzero provider codes other than recognized expired-session codes, and `MIDEA_REQUEST_FAILED` for transport failures. Provider messages, URLs and identifiers are never copied into these errors. Recognized session expiry returns `AUTH_REQUIRED` without automatic login/retry. It stops subsequent inventory requests; state queries return the error directly. Individual home failures remain explicit in a partial inventory; a failed top-level listing never becomes an empty account. Unsupported state families, query combinations, components or explicit temperature units return `CAPABILITY_UNSUPPORTED`. Cancellation/deadline stop the read flow. These errors do not establish any device-write outcome or real-platform support.

The pure [LAN discovery decoder](adr/0045-bounded-lan-discovery-decoding.md) also
returns fixed `MIDEA_RESPONSE_INVALID` errors for unsupported envelopes, invalid
source addresses, lengths, checksums, padding, IDs, ports or family fields. Errors
contain no datagram bytes or identifiers. This is local parsing without network
I/O, account access, retries or device admission; it establishes no write outcome.

The separate bounded LAN collector rejects invalid network scopes with
`INVALID_ARGUMENT`. Socket setup/send/receive failures return fixed
`MIDEA_REQUEST_FAILED` without OS messages or addresses; cancellation and inherited
budget expiry retain `CANCELLED` and `DEADLINE_EXCEEDED`. Received invalid packets
are ignored within the datagram limit. A completed five-second window is an
observation report, not proof that every device responded; `limit_reached` marks
early bounded completion. Errors publish no partial candidates or device changes.

The [V3 wire codec](adr/0046-bounded-v3-lan-codec.md) uses `INVALID_ARGUMENT` for
unsupported credential lengths or outgoing payload sizes, `AUTH_REQUIRED` for
an explicit handshake rejection or failed key confirmation, and fixed
`MIDEA_RESPONSE_INVALID` for malformed frames or failed response digests. Its
`AUTH_REQUIRED` does not establish cloud session expiry or authorize automatic
reauthentication. OS randomness failure is `MIDEA_REQUEST_FAILED`; request-counter
exhaustion is `RESOURCE_BUSY`, with no wrap or automatic reconnection. None of
these codec-only results establishes a device-write outcome or permits a retry.

The account-bound read path returns `DEVICE_IDENTITY_CONFLICT` before transport when the local device, selected connection or known model differs from the protected binding. Invalid components return `CAPABILITY_UNSUPPORTED`. Pending or unavailable credentials stop the request through the vault. Core rejects decoded but undeclared properties with `INVALID_OBSERVATION`; neither that rejection nor a protocol failure replaces the previous cache or triggers a retry.

Protected inventory membership is independent of historical identity. API 1.9.1
returns `MIDEA_DEVICE_NOT_LISTED` when the last complete retrieval omitted the
device, or `MIDEA_INVENTORY_REQUIRED` when no complete inventory has established
membership (including legacy payloads). Both stop bound reads and controls before
network work, with `retryable=false`. Offer explicit account refresh; do not infer
revocation, delete the card or resend a write automatically. Controls retain
`failed` / `not_sent`. Local/cached access remains available. See
[ADR-0035](adr/0035-meiju-inventory-membership.md).

## Meiju product mapping evidence

Malformed `sn8`, `modelNumber` or `category` in inventory returns a sanitized
`MIDEA_RESPONSE_INVALID` for that home; retrieval is partial and cannot publish
new cards. Conflicting nonempty evidence for one appliance, within retrieval or
against protected bindings, returns `DEVICE_IDENTITY_CONFLICT` before saving the
binding update. No value is included in the error. Missing values do not erase
known evidence. Corrupt or unsupported protected payloads return
`ACCOUNT_SESSION_UNCONFIRMED`; do not recreate bindings. See
[ADR-0040](adr/0040-protected-meiju-product-profiles.md).

## Meiju cloud control errors

The explicit experimental AC control path uses existing execution codes and schema
1.0. Unsupported model/action/host, invalid arguments, private binding conflicts and
credential/preflight failures stop before transmission. HTTP 401/403 or recognized
authorization codes 7400/40002 return a rejected `AUTH_REQUIRED`, with no automatic
login or resend. Unlike read errors, arbitrary nonzero control codes do not prove
rejection: these, invalid replies and failures after possible transmission become
`WRITE_OUTCOME_UNKNOWN`. Provider text and transport details are never exported.
An explicit code-zero acknowledgement remains accepted with `VERIFICATION_PENDING`
when verification was requested; current cloud observations cannot verify it.
See the [outcome table](development/07-meiju-control.md#outcome-contract).

Native startup rejects malformed, duplicate, excessive, noncanonical or wrong-adapter
`meiju_power_grants` (API 1.3), `meiju_fan_grants` (API 1.9),
`meiju_mode_grants` (API 1.11) or `meiju_temperature_grants` (API 1.12) with `INVALID_ARGUMENT` before profile creation.
A valid startup grants only the listed local devices and exact power states, fan
codes, combined power-on/mode actions or exact temperature targets over cloud. Activation accepts only
`cool`, `heat`, `auto`, `dry` and `fan`; `off`, aliases and extra fields are invalid.
Power/fan grants cannot authorize activation or vice versa. Other values/routes still return
`POLICY_DENIED`; no implicit grant expansion is allowed.
Temperature startup values must be finite numbers from 16 to 30 in half-degree
increments; this admission union does not override the device action's narrower
bounds/step. Invalid targets, extra fields or duplicate numeric targets such as
26 and 26.0 return `INVALID_ARGUMENT`. Missing or unsupported protected mappings
return `CAPABILITY_UNSUPPORTED`; invalid action values return `INVALID_ARGUMENT`
before transmission. Protected identity/membership checks still apply. Neither
grant admission nor a successful plan proves physical temperature support.

The Linux CLI applies the same validation to an explicit `--meiju-grants` startup
file. Relative paths and malformed contents return `INVALID_ARGUMENT`. Unreadable,
nonregular, foreign-owned, group/other-writable or oversized files and final
symlinks return `POLICY_DENIED`, without exposing their path or contents. Missing
or empty grants deny appliance writes before profile startup. Single Meiju writes
and previews without an explicit request ID return `INVALID_ARGUMENT` when grants
are present. See [CLI admission](cli.md#scoped-meiju-controls).

Meiju MCP applies the same startup-file checks. Without nonempty grants, command
tools are unlisted and direct calls return `POLICY_DENIED`. Forged/invalid command
inputs return the query error envelope. Once decoded and validated, execution
returns its existing receipt, including Core preflight failures. MCP `isError`
is true for failed/unknown/partial outcomes and for acceptance when verification
was requested; receipt lookup itself remains successful regardless of stored
status. `MCP_IO_ERROR` ends the session after cancelling and draining active work;
a later lookup is required when the response could not be delivered. See
[MCP results](mcp.md#results-and-shutdown).

For the reviewed scalar AC temperature profile, malformed/nonfinite `temperature`
returns `MIDEA_RESPONSE_INVALID` and leaves the previous snapshot intact. Missing
or null target fields remain absent; `small_temperature` is not a recognized
field in that selected mapping. Compound profiles retain their two-field checks.
Neither path applies write bounds to observations or establishes freshness.

## Native account connection results

`homeverb_connect_account` is separate from ordinary JSON and rejects invalid
metadata with `INVALID_ARGUMENT`, unsupported hosts with `AUTH_FLOW_UNSUPPORTED`,
and invalid native buffers with `MIDEA_LOGIN_INPUT_UNSUPPORTED`. Host foreground,
closed, duplicate-ID and concurrency errors are shared with normal calls.

After confirmed authentication/session storage, retrieval errors remain inside
`data.retrieval.error` while `data.authentication` is `succeeded`; inspect both layers.
`refresh_account` returns `not_attempted` authentication and never logs in.
Partial retrieval preserves old cards. Cancellation or storage failure after some
local publications does not roll them back. `SECRET_STORE_WRITE_UNKNOWN` can leave
pending credentials; reload local metadata and require trusted recovery rather
than recreating/resetting the account. General account listing never validates
live authorization. See [the account contract](native-ui-contract.md#account-connection-and-refresh).

`remove_account` uses an outer query error when refusal occurs before cleanup
intent. Once intent is durable, cleanup errors are inside `data.error` with
`data.status: pending`; query success alone is not removal success. Keep the
account visible as `credential_state: removing` and offer explicit local-cleanup
retry. In API 1.8.1 the backend inspects the exact entry by reference before any
deletion, including explicit retries, without requiring readable credential bytes.
Locked/unavailable/ambiguous storage remains a refusal, not proof of absence.
`removed` requires completed local cleanup and a null inner error. Late descriptor
saves for a completed removal return `ACCOUNT_NOT_FOUND`; the old local ID cannot
be reused. Neither status
establishes provider revocation; see [ADR-0028](adr/0028-local-account-removal.md).

`clear_execution_history` returns `RESOURCE_BUSY` without deleting any receipt if
an execution journal is active. Explicit cleanup otherwise returns a query result
with a removed row count; a storage error rolls back that transaction. Cleared
history uses the existing `REQUEST_RETIRED` outcome, while cleared cached reads use
`CACHE_MISS` without a network fallback. Interrupted callers reload local views and
must not automatically repeat cleanup or appliance commands.

`recover_account` uses a query result with the selected account's current credential
state and nested retrieval error. `AUTH_REQUIRED`, `MIDEA_INVENTORY_PARTIAL`, malformed
protected material or changed confirmation bytes leave pending material unconfirmed.
A stored result with a publication error is distinct from a pending recovery failure.
Expired pending credentials may use explicit same-account reauthorization in API
1.8 when protected identity/bindings remain intact. Missing/corrupt material cannot
be repaired by login; preserve it. Changed protected bytes during login return
`ACCOUNT_SESSION_UNCONFIRMED` without replacement; a different login subject returns
`ACCOUNT_IDENTITY_CONFLICT`. Uncertain persistence uses the existing storage errors.
Reload local metadata after interruption. No error permits automatic login or an appliance retry.

## Saved inventory inspection errors

`inspect_account_inventory` uses existing account/vault errors: `ACCOUNT_NOT_FOUND`
for missing/foreign scope, `RESOURCE_BUSY` for a held account lease,
`ACCOUNT_SESSION_UNCONFIRMED` for pending/corrupt material, and
`ACCOUNT_REMOVAL_PENDING` during removal. Secure-store errors, cancellation,
deadline and host foreground/closed-session gates retain their existing meanings.
No error causes cloud fallback or mutation. Unconfirmed legacy membership is a
successful saved-evidence result; it does not make a live read/control admissible.

Profile-scoped dishwasher startup (API 1.14) rejects malformed, duplicate, oversized
or non-Meiju `meiju_dishwasher_grants` with `INVALID_ARGUMENT` before profile creation.
A grant without trusted confirmation still returns `POLICY_DENIED`. Missing product
mappings or host opt-in remain `CAPABILITY_UNSUPPORTED`; a changed protected family,
model or unsupported profile cannot be replaced by public descriptor edits.
Argument, identity, membership and receipt errors keep their existing semantics.
An accepted start is not verified; an uncertain start must never be auto-replayed.
