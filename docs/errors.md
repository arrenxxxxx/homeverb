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

## Meiju inventory errors

The native inventory library uses `MIDEA_RESPONSE_INVALID` for malformed/oversized or wrong-home responses, `MIDEA_REQUEST_REJECTED` for explicit nonzero provider codes other than recognized expired-session codes, and `MIDEA_REQUEST_FAILED` for transport failures. Provider messages, URLs and identifiers are never copied into these errors. Recognized session expiry returns `AUTH_REQUIRED` and stops further home requests without automatic login/retry. Individual home failures remain explicit in a partial inventory; a failed top-level listing never becomes an empty account. Cancellation/deadline stop the whole read flow. These errors do not establish any device-write outcome or real-platform support.
