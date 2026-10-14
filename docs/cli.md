# CLI contract

> All `homeverb` commands are proposed. This repository does not ship an executable. Proposed command syntax version: 1.0.

## Command namespace

```text
homeverb auth providers
homeverb auth login <provider> --profile <name>
homeverb auth status [--profile <name>]
homeverb auth logout <account-id>
homeverb device discover [--profile <name>] [--connection auto|lan|cloud]
homeverb device list [--json]
homeverb device describe <device> [--json]
homeverb device state <device> [--fresh|--cached] [--json]
homeverb device alias <device> <alias>
homeverb power set <device> --on|--off
homeverb climate set <device> [--temperature <celsius>] [--mode <value>]
homeverb device execute <device> --component <id> --capability <id> --action <id> --arguments <json>
homeverb doctor [--redact]
homeverb mcp --stdio [--profile <name>]
```

Standard capabilities get convenient commands. Generic `execute` still enforces schemas, permissions, and risk; it is not a bypass. Reject ambiguous aliases instead of guessing or acting on every matching device.

## Common options

| Option | Meaning |
|---|---|
| `--json` | Machine-readable output |
| `--profile` | Local configuration namespace |
| `--connection auto\|lan\|cloud` | Explicit routing policy |
| `--timeout 10s` | Total operation budget |
| `--wait verified\|accepted` | Required write-completion condition |
| `--non-interactive` | No prompts; return a structured auth error if needed |
| `--dry-run` | Local validation and planning only; no writes or automatic login |

Mark cached capability metadata as potentially stale during a dry run. State reads prefer live data; a normal read may return clearly warned cache fallback, but `--fresh` forbids presenting cache as a successful live read. `--cached` never networks. Writes default to waiting for `verified`; an unverified result remains `accepted` or `unknown` with a distinct exit status.

## Output rules

JSON stdout contains no logs, progress bars, ANSI escapes, or credentials. Diagnostics go to stderr. MCP stdout carries protocol messages only. Field IDs, baseline human messages, examples, and repository documentation are English. Missing sensor readings are not zero.

Queries return `schema_version`, `data`, `warnings`, and `error`. Writes use the [execution result envelope](../examples/command-result.verified.json). `--json` also returns structured errors where the process can still produce output; process crashes cannot guarantee a valid envelope.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Query succeeded, or write met the requested `--wait` condition |
| 2 | Invalid syntax, argument, or capability schema |
| 3 | User authentication action required |
| 4 | Unsupported device, capability, or platform |
| 5 | Unreachable route without an uncertain transmitted write |
| 6 | Permission or risk policy denied |
| 7 | Local write or refresh conflict |
| 10 | Explicit execution failure |
| 11 | Unknown write outcome |
| 12 | Accepted, but verification was requested |
| 13 | Partial batch/composite outcome |
| 130 | User interruption; a transmitted action may still execute |

Scripts must not parse a localized success string. Exit 0 with `--wait accepted` means acceptance only, not physical completion.

## Forbidden commands

No `daemon`, `service install`, `gateway start`, `serve --http`, `schedule`, or `automation run`. The `mcp --stdio` exception is limited to its host session; see [Process lifecycle](process-lifecycle.md).
