# ADR-0039: Admit explicit AC power-on and mode activation

Status: accepted for offline implementation; physical acceptance remains deferred.

## Evidence and decision

Pinned Midea Auto Cloud revision `ce0c4b88224def0f6b53714312c19f71aa9c2a50`
maps its default AC mode selections to one control object containing `power: on`
and `mode: cool|heat|auto|dry|fan`. `climate.py` sends that complete mapping when
setting HVAC mode. This is not a mode-only setter. The separate `off` selection
maps only to power-off and is already covered by HomeVerb's power action.

Expose `main / vendor.midea.ac.activate_mode {mode}` for those five exact vendor
values, explicitly meaning **turn on and activate mode**. Keep it vendor-specific
and require separate host opt-in for the four exact experimental AC read models.
Source evidence admits a candidate mapping, not model/firmware verification. Do
not guess target-temperature bounds/steps or add dishwasher/refrigerator controls.

Send exactly `command.control: {power: "on", mode: <mode>}` in one request. Neither
copy stale `command.status` nor change fan/temperature nor send another power
command. `atomic` and `independent_fields` are false: one vendor request does not
prove atomic physical changes and must never be split into separate commands.
Preserve the existing accepted/unknown/rejected, cancellation and replay rules.
Current cloud timestamps cannot verify either effect. Physical acceptance must
check both effects and preservation of unrelated settings, including the existing
missing-status-context limitation in ADR-0034.

## Authority and compatibility

Native API 1.11.0 adds optional `meiju_mode_grants`: at most 64 unique
`{device_id, mode}` entries, with canonical local UUID v4 and one reviewed mode.
Each grant covers the full power-on-and-mode action over cloud. Power-only and
fan-only grants do not permit activation; activation does not grant independent
power, fan, temperature or other modes. UI labels and consent must name both
effects. Ordinary commands and model inputs cannot change startup authority.

The same closed list is available in explicit CLI/MCP startup files. Missing or
empty lists preserve earlier behavior; older binaries reject the new field.
Negotiate native API 1.11 before supplying it. Keep the 31 JSON methods, seven C
functions, SQLite schema 4, protected payload version 2 and existing Rust host
helpers compatible. New scoped host helpers accept the additional list.

## Verification

Use fictional adapters/accounts to verify all admitted models/modes, exact combined
wire bodies without unrelated values, metadata tampering, protected model/binding
and inventory refusal, independent permissions, unchanged cache, receipts/replay,
and foreground cancellation. Validate native schema/ABI admission, CLI/MCP plans
and pre-send failures, and English/Chinese power-on labels. No real credential
read, provider request or appliance control is authorized by this implementation.
