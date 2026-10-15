# Fictional contract examples

Every device, account, timestamp, and result here is fictional data, not physical-device evidence. The implemented `homeverb.mock` adapter reuses capability fixtures and substitutes its own fictional device/connection identifiers.

| Fixture | Purpose |
|---|---|
| [device.json](device.json) | One appliance with candidate LAN and cloud connections |
| [capability.climate.json](capability.climate.json) | Example climate attributes/actions; 16-30 Celsius and 0.5 steps are not universal model limits |
| [device.refrigerator.json](device.refrigerator.json) | One fictional device with fridge and freezer components |
| [capability.fridge.json](capability.fridge.json) / [capability.freezer.json](capability.freezer.json) | Distinct component-scoped definitions of `climate`; illustrative ranges, steps, and freshness only |
| [capability.meiju-mode.json](capability.meiju-mode.json) / [command.meiju-mode.json](command.meiju-mode.json) | Experimental combined AC power-on and vendor mode selection; separate exact-mode authority and no physical verification |
| [capability.meiju-temperature.json](capability.meiju-temperature.json) / [command.meiju-temperature.json](command.meiju-temperature.json) | Experimental protected-profile AC setpoint; illustrative 17–30 Celsius half-degree action limits, independent exact-target authority and no physical verification |
| [capability.approved-cycle.json](capability.approved-cycle.json) / [command.approved-cycle.json](command.approved-cycle.json) / [command-result.approval-expired.json](command-result.approval-expired.json) | Fictional elevated cycle for the typed trusted-user approval tests; ordinary JSON cannot approve it, and expiry has no send attempts |
| [command.json](command.json) | Brand-independent command envelope |
| [cli-meiju-grants.json](cli-meiju-grants.json) | Fictional exact-device power-off and vendor fan-code permissions for explicit CLI startup; never an ordinary command |
| [batch-command.json](batch-command.json) | Explicit commands for two fictional devices; not a runnable two-device Mock setup |
| [batch-result.json](batch-result.json) | Mixed per-device outcomes with preserved evidence and uncertainty |
| [command-result.verified.json](command-result.verified.json) | Result with state-match evidence |
| [command-result.completion.json](command-result.completion.json) | Successful fictional correlated completion without a current-state claim |
| [command-result.accepted.json](command-result.accepted.json) | Accepted but not verified |
| [command-result.unknown.json](command-result.unknown.json) | Uncertain write; no blind retry |
| [command-result.composite.json](command-result.composite.json) | Verified non-atomic write with separate item evidence |
| [command-result.partial.json](command-result.partial.json) | Mixed sub-operation outcomes |
| [command-result.storage-failure.json](command-result.storage-failure.json) | Preserved verification after a storage error; remaining item not sent |
| [command-result.retired.json](command-result.retired.json) | Retired request ID rejected without another send |
| [command-result.policy-denied.json](command-result.policy-denied.json) | Unauthorised command rejected before transmission |
| [command-result.meiju-not-listed.json](command-result.meiju-not-listed.json) | Retained device refused before transmission after complete inventory omission |
| [room.json](room.json) | Local room with a stable ID and position |
| [room-assignment.json](room-assignment.json) | Fictional selection moved to one local room atomically |
| [device-preferences.json](device-preferences.json) | Local alias, room assignment, favorite, and position |
| [diagnostics.json](diagnostics.json) | Bounded local counts and redacted receipt summary without private record fields |
| [state.json](state.json) | Explicitly cached and stale observation |
| [state.live.json](state.live.json) | Fictional live response with fresh target evidence and an older unrelated property |
| [adapter-manifest.json](adapter-manifest.json) | Candidate metadata without a daemon or project backend |

These show contract arguments, not vendor wire packets. No real passwords, serial numbers, or IP addresses are supplied. Human-readable fixture messages are English.
