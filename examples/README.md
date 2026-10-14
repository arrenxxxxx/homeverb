# Fictional contract examples

Every device, account, timestamp, and result here is fictional design data, not physical-device evidence. There is no implemented `example.mock` driver yet.

| Fixture | Purpose |
|---|---|
| [device.json](device.json) | One appliance with candidate LAN and cloud connections |
| [capability.climate.json](capability.climate.json) | Example climate attributes/actions; 16-30 Celsius and 0.5 steps are not universal model limits |
| [command.json](command.json) | Brand-independent command envelope |
| [command-result.verified.json](command-result.verified.json) | Result with state-match evidence |
| [command-result.accepted.json](command-result.accepted.json) | Accepted but not verified |
| [command-result.unknown.json](command-result.unknown.json) | Uncertain write; no blind retry |
| [state.json](state.json) | Explicitly cached and stale observation |
| [adapter-manifest.json](adapter-manifest.json) | Candidate metadata without a daemon or project backend |

These show contract arguments, not vendor wire packets. No real passwords, serial numbers, or IP addresses are supplied. Human-readable fixture messages are English.
