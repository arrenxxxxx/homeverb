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
| DEVICE_NOT_FOUND | Unknown device ID | List local records or rediscover |
| AMBIGUOUS_DEVICE | Duplicate alias | Require an exact ID |
| CAPABILITY_UNSUPPORTED | Unsupported action or component | Return actual capabilities |
| INVALID_ARGUMENT | Invalid range or enum | Correct using the schema |
| POLICY_DENIED | Permission or risk restriction | Trusted user action; no model bypass |
| LAN_UNREACHABLE | LAN unavailable and write proven unsent | Use a reachable network |
| CLOUD_UNREACHABLE | Cloud unavailable and write unsent | Check network/vendor availability |
| RATE_LIMITED | Vendor rate limit | Honor budget and return retry guidance |
| RESOURCE_BUSY | Local account/device operation lock conflict | Report conflict; bounded waiting |
| DEVICE_REJECTED | Explicit device rejection | Failed with the known reason |
| WRITE_OUTCOME_UNKNOWN | Possibly sent; acknowledgement missing | Unknown; no automatic replay |
| VERIFICATION_PENDING | Accepted but not verified | Accepted; do not resend |
| PARTIAL_EXECUTION | Different sub-operation outcomes | Return per-item outcomes |
| CANCELLED | User cancellation | Report whether transmission may have occurred |

After possible transmission, prefer `WRITE_OUTCOME_UNKNOWN` over a misleading `LAN_UNREACHABLE`. See [CLI exit codes](cli.md).

## Invariants

`verified` includes verification evidence. `failed` includes an explicit error. `unknown` explains uncertainty and must not suggest blind retries. Batch aggregation preserves every item; `partial` never hides successes or failures. Contract version changes require explicit coordinated documentation and migration; do not silently treat an incompatible envelope as valid.
