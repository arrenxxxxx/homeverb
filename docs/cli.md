# CLI contract

Status: the listed Mock, local organization, and redacted diagnostic commands are implemented. `--mock` is required. Real authentication is pending; do not infer it from the design baseline.

## Implemented commands

```text
homeverb --mock device discover
homeverb --mock device list
homeverb --mock device cards
homeverb --mock device preferences <device> --settings <json>
homeverb --mock device reorder <all-device-ids...>
homeverb --mock room list
homeverb --mock room create <name>
homeverb --mock room rename <room-id> <name>
homeverb --mock room delete <room-id>
homeverb --mock room reorder <all-room-ids...>
homeverb --mock room assign <device-ids...> --room <room-id>
homeverb --mock room assign <device-ids...> --unassigned
homeverb --mock device describe <device>
homeverb --mock device state <device> [--fresh|--cached] [--component main]
homeverb --mock power set <device> --on|--off
homeverb --mock climate set <device> [--temperature <celsius>] [--mode <value>]
homeverb --mock device execute <device> --component <id> --capability <id> --action <id> --arguments <json>
homeverb --mock [--dry-run] batch --file <local-json-path>
homeverb --mock executions [--device <device>]
homeverb --mock execution <request-id>
homeverb --mock diagnostics
homeverb --mock mcp --stdio
```

Global options: `--profile` (default `mock`), `--data-dir`, `--connection auto|lan|cloud`, `--timeout-ms` (1-180000, default 10000), `--wait accepted|verified` (default verified), `--request-id`, `--dry-run`, `--scenario`, and `--json`. All operation responses are JSON even without `--json`. Help/version output is text.

`batch --file` reads a [BatchCommand](../schemas/batch-command.schema.json): `schema_version: "1.0"` and 1-32 full `commands` for distinct device IDs. Each command supplies its route, wait condition, timeout, and request ID. Global nondefault `--connection`/`--wait` and any global `--request-id` are rejected for batches. Global `--timeout-ms` limits the entire batch, including preflight and queued work. The fictional CLI host still grants only its known Mock device. Do not put credentials in batch files.

Whole-list preflight must pass before writes. `--dry-run` returns per-command plans without journaling. Successful execution returns [BatchExecution](../schemas/batch-result.schema.json) with ordered items; preflight failure returns the query error envelope. Mixed outcomes exit 13, uniform uncertainty exits 11, and uniform acceptance exits 12 if any item requested verification. Uniform failures use the first item's existing error exit code. Interruption exits 130 after returning available item outcomes. Inspect individual request IDs after interruption; there is no aggregate receipt, durable queue, or automatic batch retry.

An explicit request ID must be nonblank and at most 256 UTF-8 bytes, including for composite commands. Generated child receipt IDs append a step suffix and can be longer; inspect either parent or child with `execution <request-id>`. Never submit a derived child ID as a new write. Reusing the original ID with different arguments returns `REQUEST_CONFLICT`.

Queries use `{schema_version,data,warnings,error}`. Writes use the execution envelope. Logs do not share JSON stdout. Cached reads never contact an adapter. Live reads, including `--fresh`, do not silently fall back to cache. Dry run validates local metadata and policy only; its warning identifies potentially stale capabilities. Ambiguous aliases require an exact ID.

Local organization mutations return the query envelope, not appliance execution receipts. `--settings` requires all three editable fields, for example `'{"alias":"Quiet AC","room_id":null,"favorite":true}'`. Null restores the original alias or removes a room assignment. Reorder arguments must contain every current ID exactly once; invalid lists make no changes. Deleting a room only unassigns its devices. Names, favorites, and ordering survive rediscovery and process exit. These commands do not contact an appliance or grant control permissions.

`room assign` accepts 1-256 distinct exact device IDs and requires exactly one destination option. It atomically changes only room membership; missing references or storage errors leave every device unchanged. Results follow the requested ID order. Aliases, favorites, and card positions are preserved. Use `--unassigned` to clear membership explicitly.

`executions --device` accepts an exact ID or unambiguous alias and filters retained history before applying the 100-row limit. Unknown devices fail. A known device with no retained history returns an empty array. This does not contact the adapter, preserve additional records, or recover retired receipt details.

`diagnostics` returns a local report in the query envelope. Preview it on stdout, or save that same output through ordinary shell redirection to a private location outside the repository. It performs no discovery, appliance read, login, keyring access, or upload. Normal host startup still opens the profile and applies any required migration/recovery. The report contains counts and at most 100 retained receipt summaries, with at most 32 attempts per receipt and an explicit omitted-attempt count. Device/account identifiers, aliases, room names, state values, arguments, operation timestamps, and raw messages are omitted; unknown error codes become `OTHER`. Counts describe local metadata, not live reachability. Composite parent and child rows may both appear. See [diagnostic privacy](security-and-privacy.md#local-diagnostic-reports).

For multi-field writes explicitly declared as independent setters, `--dry-run` includes `items` containing each single-field command and its candidate transports. Other actions keep their arguments together. The preflight validates both whole-command and per-step permissions before reporting a plan. It does not contact the appliance or prove reachability. This CLI's Mock grants cover only `main` power/climate actions; there is no model-controlled grant-editing option.

`execution <request-id>` reads one exact retained receipt in the query envelope. Query success exits 0 regardless of the nested command status. It makes no device call, needs no current write grant or valid device descriptor, and does not rewrite the receipt after a state refresh. Missing IDs return `REQUEST_NOT_FOUND`; retired IDs return `REQUEST_RETIRED`. Neither permits automatic replay. Host startup still performs ordinary local recovery; the lookup itself does not mutate the journal. See [ADR-0014](adr/0014-local-receipt-lookup.md).

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Query succeeded or write met its requested wait condition |
| 2 | Invalid syntax, argument, capability schema, contract version, or conflicting room name |
| 3 | Authentication or secure credential storage required |
| 4 | Unknown room/device/receipt, ambiguous device, conflicting discovery identity, or unsupported capability |
| 5 | No reachable permitted route or unsent deadline expiry |
| 6 | Policy denied |
| 7 | Device/migration lock, request-ID conflict, or retired request ID |
| 10 | Explicit device failure or local infrastructure error |
| 11 | Unknown write outcome |
| 12 | Accepted, but requested verification is unavailable |
| 13 | Different non-atomic sub-operation outcomes |
| 130 | User interruption; inspect the result because transmission may have occurred |

`--wait accepted` permits exit 0 for acceptance without physical verification. An identical request ID returns its recorded result; a changed command under that ID is rejected. Once detailed history is retired, any reuse returns `REQUEST_RETIRED` without sending. Refresh state instead of automatically replacing that ID. No daemon, service installer, HTTP server, automation, or scheduled execution command exists.

See [the runnable walkthrough](development/01-run-mock.md) and [execution semantics](routing-and-execution.md).
