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
| POLICY_DENIED | No complete matching device/component/action/argument/route grant, or an elevated/high-risk action | Trusted host policy change; no model-supplied bypass |
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

The account-bound read path returns `DEVICE_IDENTITY_CONFLICT` before transport when the local device, selected connection or known model differs from the protected binding. Invalid components return `CAPABILITY_UNSUPPORTED`. Pending or unavailable credentials stop the request through the vault. Core rejects decoded but undeclared properties with `INVALID_OBSERVATION`; neither that rejection nor a protocol failure replaces the previous cache or triggers a retry.

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
`meiju_power_grants` (API 1.3) or `meiju_fan_grants` (API 1.9) with `INVALID_ARGUMENT`
before profile creation. A valid startup grants only the listed local devices and
exact states or reviewed fan codes over cloud. Other values/routes still return
`POLICY_DENIED`; no implicit grant expansion is allowed.

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
