# CLI contract

Status: Mock commands, local organization, diagnostics and the experimental mainland
Meiju account/read flow and scoped power, fan, mode activation and profile-scoped temperature controls are implemented. Select `--mock` or `--adapter midea.meiju`
explicitly. Meiju uses the same native Host/Core as the app bridge; no example
executable, HA or service is required. Real CLI login/read/control acceptance remains
separate from the earlier [developer probe](development/06-meiju-probe.md).

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

Global options: `--profile` (default `mock` or `meiju` for the selected adapter),
`--data-dir`, `--connection auto|lan|cloud`, `--timeout-ms` (1-180000, default 10000;
account connect defaults to 180000 after input), `--wait accepted|verified`
(default verified), `--request-id`, `--dry-run`, `--scenario`, `--meiju-grants`, and `--json`.
All operation responses are JSON even without `--json`. Help/version output is text.
`--mock` conflicts with `--adapter`. Explicit `--scenario` remains Mock-only.
MCP accepts either adapter with the startup policy described in the
[local MCP contract](mcp.md); command-only global options are restricted there.
Existing Mock command forms and profile paths are unchanged.

## Meiju account and read commands

The default Meiju profile is `$XDG_DATA_HOME/homeverb/meiju`, or
`$HOME/.local/share/homeverb/meiju` when XDG_DATA_HOME is unset. An explicit
`--data-dir` is the parent directory; `--profile` is always appended. To reuse the
existing developer-probe profile, pass `--data-dir "$HOME/.local/share/homeverb"
--profile meiju-pilot`; do not append `meiju-pilot` twice or copy credential files.

```text
homeverb --adapter midea.meiju account connect --label "Home appliances"
homeverb --adapter midea.meiju account connect --label "Home appliances" --account-id <local-account-id>
homeverb --adapter midea.meiju account list
homeverb --adapter midea.meiju account show <local-account-id>
homeverb --adapter midea.meiju account inventory <local-account-id>
homeverb --adapter midea.meiju account refresh <local-account-id>
homeverb --adapter midea.meiju account recover <local-account-id>
homeverb --adapter midea.meiju account rename <local-account-id> "Home appliances"
homeverb --adapter midea.meiju account remove <local-account-id> --expected-device-id <local-device-id>
homeverb --adapter midea.meiju device discover
homeverb --adapter midea.meiju device cards
homeverb --adapter midea.meiju device describe <local-device-id>
homeverb --adapter midea.meiju --connection cloud device state <local-device-id> --fresh --component main
homeverb --adapter midea.meiju device state <local-device-id> --cached --component main
```

`connect` is Linux-only and requires interactive stdin/stderr plus an unlocked
Secret Service collection. It checks storage before prompting. Account and password
are read from the controlling terminal with echo disabled, bounded to 256 bytes
each, then passed as native secret buffers. There are no credential arguments,
environment inputs, pipe inputs or public credential JSON. Ctrl+C cancels input
and normal unwinding restores terminal settings; forced process termination cannot
promise cleanup. Temporary application buffers are cleared; kernel/terminal copies
and forensic erasure cannot be guaranteed. Unsupported credential characters or
provider challenges fail through the existing native flow.

Human input is outside the network budget. A separate 10-second preflight checks
secure storage; after input, `--timeout-ms` bounds the shared login/inventory work.
Other account methods begin their budget at native dispatch after local host startup.
No background worker, automatic login, refresh or retry is started. Ctrl+C propagates
to the native session and waits for its result, preserving known/uncertain storage
outcomes instead of discarding them. Successful connection automatically publishes
supported and unsupported cards through the same Core validation as the app.

`list`, `show` and `rename` use local metadata without reading credentials.
`inventory` reads saved protected membership without network access. `refresh`
retrieves the selected account's devices with saved credentials. `recover` explicitly
validates interrupted saved authorization; it never substitutes another login.
See [account recovery and reauthorization](native-ui-contract.md#recover-interrupted-account-storage).
`remove` deletes only local authorization and the reviewed linked-card set. Repeat
`--expected-device-id` for every ID returned by `show`; omit it only for an empty
set. A changed set refuses cleanup. A pending cleanup retains its nested error
and can be retried explicitly with the same reviewed set. No provider revocation
or appliance command follows removal.

Account responses retain the [native response shapes](../schemas/native-response.schema.json).
Exit status inspects both the top-level error and nested retrieval/removal errors.
Stored authorization with partial retrieval exits 13; failed retrieval uses its
error's exit code. Pending removal is never exit 0. Do not discard the account
result merely because the command exits nonzero. `--dry-run`, command request IDs,
nondefault route/wait options and Mock scenarios are invalid for account operations.

Local room/card/preferences, cache, receipt and diagnostic commands also work
with the Meiju profile. Live discovery/reads use the selected adapter only; cached
reads retain their recorded evidence. Missing/unsupported models stay unsupported,
cloud readings remain stale, and no LAN route is added. Without nonempty explicit
startup grants, appliance writes and their dry runs return `POLICY_DENIED` before
opening the profile. MCP can reuse this profile through explicit
`--adapter midea.meiju mcp --stdio` startup.

## Scoped Meiju controls

On Linux, `--adapter midea.meiju --meiju-grants <absolute-path>` loads a trusted
startup file for experimental AC power, vendor fan, combined power-on/mode and
profile-scoped temperature actions. The
owner must review each local device ID and allowed value. The file is never loaded
automatically, saved into the profile or accepted inside ordinary command JSON.
It has the closed [CLI grant schema](../schemas/cli-meiju-grants.schema.json); see
the [fictional example](../examples/cli-meiju-grants.json) and
[ADR-0037](adr/0037-cli-meiju-control-grants.md).

The same file may be selected by a trusted [MCP host](mcp.md). MCP captures its
policy for the entire session; stop/restart that host to change grants. One-shot
CLI commands reread the explicitly supplied file on each invocation.

`schema_version` is `"1.0"`. The independent `meiju_power_grants`,
`meiju_fan_grants` and `meiju_mode_grants` lists each permit at most 64 distinct
entries. `meiju_temperature_grants` permits 256 entries starting with native API
1.14.1; earlier binaries reject more than 64. The entire file still has a 32 KiB
limit. Keep exact reviewed pairs and never silently truncate a list.
Power entries name one local `device_id` and boolean `state`; fan entries name
one local `device_id` and one `wind_speed` code: 20, 40, 60, 80, 100 or 102. These
are vendor codes, not percentages. Mode entries name one local `device_id` and
one `mode`: `cool`, `heat`, `auto`, `dry` or `fan`. Mode activation **turns on the
AC and selects that mode**; it does not authorize independent power or fan changes.
Power/fan grants do not authorize activation. Older binaries reject the new mode
list; see [ADR-0039](adr/0039-explicit-meiju-mode-activation.md).
Omitted/empty lists grant nothing. No wildcard, provider identifier,
arbitrary action, credential or approval flag is accepted. Only the existing
exact-model AC mappings and cloud route are admitted; dishwasher and refrigerator
controls remain unsupported.

Temperature entries name one local `device_id` and exact Celsius
`target_temperature`. The startup union is 16–30 in half-degree increments, but
the selected protected product mapping and action schema must also admit the
value. There is no family-wide range or fallback for missing product evidence.
Temperature grants never authorize power, mode, fan or another target. Older
binaries reject this list; see [ADR-0041](adr/0041-profile-scoped-ac-temperature.md).

Store the file outside the repository at an absolute path, owned by the current
OS user and not writable by group/others; mode `0600` is recommended. The loader
checks the opened regular file, refuses final symlinks and limits content to
32 KiB. Invalid content or a relative path returns `INVALID_ARGUMENT`; an unsafe,
unavailable or oversized file returns `POLICY_DENIED`. The complete policy is
validated before profile creation. This does not defend against a compromised
owning process or validate every parent path component.

After connecting read-only and obtaining local IDs, prepare the reviewed file and
explicitly refresh the selected account with it to publish admitted descriptors.
There is no hidden discovery or refresh before a write. These commands illustrate
the workflow; replace placeholders and use the same profile throughout:

```text
homeverb --adapter midea.meiju --meiju-grants <absolute-grants-path> account refresh <local-account-id>
homeverb --adapter midea.meiju --meiju-grants <absolute-grants-path> --request-id <unique-request-id> --dry-run power set <local-device-id> --off
```

A single write or preview requires an explicit `--request-id`; batch files retain
their per-command IDs and whole-list preflight. Dry run checks local descriptors
and permissions without contacting the provider or creating a receipt. It cannot
prove saved inventory membership, reachability or physical behavior. Only after
separate owner authorization for the real operation, remove `--dry-run` to send
with the reviewed request ID. For a fan preview, use the same options with
`device execute <local-device-id> --capability vendor.midea.ac --action set_fan_speed
--arguments '{"wind_speed":40}'`. For a combined power-on and mode preview, use
`device execute <local-device-id> --capability vendor.midea.ac --action activate_mode
--arguments '{"mode":"cool"}'` with a matching mode grant. Review both effects
before sending; this is not a mode-only setter. For a temperature-only preview,
use `climate set <local-device-id> --temperature 26.5` with an exact matching
`meiju_temperature_grants` entry and the same startup, request ID and dry-run
options. Read the described action's bounds and step before choosing a value;
no implicit power or mode change is included.

Execution repeats Core and protected-account checks. Cloud acceptance remains
`accepted`, never physical verification; the default verified wait exits 12 when
verification is unavailable. A possibly sent request remains `unknown`. Inspect
`homeverb --adapter midea.meiju execution <request-id>` without a grant file after
interruption or permission removal. Reusing an identical authorized request
returns its receipt; changing its command conflicts. Never create a fresh ID to
automatically retry an accepted/unknown request. See the
[control evidence limits](development/07-meiju-control.md#outcome-contract).

## Shared command behavior

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
| 3 | Authentication, confirmed saved credentials, secure storage or interactive input required |
| 4 | Unknown room/device/account/receipt, identity conflict, missing inventory membership, or unsupported capability/flow |
| 5 | No reachable permitted route or unsent deadline expiry |
| 6 | Policy denied |
| 7 | Lock, request-ID/credential conflict, changed removal scope, pending cleanup, or retired request ID |
| 10 | Explicit device failure or local infrastructure error |
| 11 | Unknown write outcome |
| 12 | Accepted, but requested verification is unavailable |
| 13 | Different non-atomic sub-operation outcomes or partial account inventory |
| 130 | User interruption; inspect the result because transmission may have occurred |

`--wait accepted` permits exit 0 for acceptance without physical verification. An identical request ID returns its recorded result; a changed command under that ID is rejected. Once detailed history is retired, any reuse returns `REQUEST_RETIRED` without sending. Refresh state instead of automatically replacing that ID. No daemon, service installer, HTTP server, automation, or scheduled execution command exists.

See [the runnable walkthrough](development/01-run-mock.md) and [execution semantics](routing-and-execution.md).
