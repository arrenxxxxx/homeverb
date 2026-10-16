# ADR-0055: Control reviewed dishwasher cycles through trusted confirmation

Status: accepted for offline implementation; physical acceptance remains deferred.

## Evidence and scope

The pinned E1 mapping at `ce0c4b88224def0f6b53714312c19f71aa9c2a50` explicitly maps
pause to `operator: pause`, resume to `operator: start`, and cancel to
`work_status: cancel`. Extend the existing experimental exact-model and protected
product admission, using only products 7600020L, 76006481, 7600649Q and 7600V1E7.
Each has an empty centralized-field list. Exclude 760Y0026 and 7600016L because
their commands additionally preserve settings whose values this operation does
not receive. Never select the family default or copy cached state into controls.

These mappings are community protocol evidence, not physical support or proof
that the owner's dishwasher has one of these product codes. The independent
source comparison checks all twelve admitted product/action payloads and the
excluded centralized profiles without account or appliance access.

API 1.21 extends only product 760Y0026 through explicit options under
[ADR-0063](0063-explicit-dishwasher-cycle-options.md); the original contract below
remains the baseline for the other four profiles.

## Contract

Native API 1.15.0 adds `meiju_dishwasher_cycle_grants`: at most 64 distinct exact
local UUID-v4/action pairs. Actions are `pause_cycle`, `resume_cycle` and
`cancel_cycle`, all on `main / vendor.midea.dishwasher` over cloud with exactly
empty arguments. Existing program-start grants neither grant these actions nor
receive authority from them. Adapter opt-in is also independent of start opt-in.

All three actions require trusted single-command confirmation under ADR-0043.
Treat them as elevated and non-idempotent: resume may consume water/heat, cancel
can terminate a program, and no replay guarantee is established for pause. Display
the exact action/device and explain its effect. Do not present protocol pause or
cancel as a safety interlock, physical rollback, completed drain or safe-door
guarantee. Ordinary JSON/CLI/MCP cannot perform the confirmation.

The adapter independently checks the approval marker before secret access, then
rechecks protected identity, inventory membership, exact model/family/profile,
host admission and empty arguments. Send one reviewed cloud payload with no
pre/post read, automatic retry, fallback, program selection, settings defaults or
restoration. Preserve existing accepted/failed/unknown receipts and replay guards;
cloud acceptance never proves the current cycle state.

Dishwasher capability revision 3 also exposes optional read-only raw `operator`
alongside `work_status`, retaining cloud staleness and vendor semantics. Never
derive a running/paused boolean or optimistic state from a submitted action.
Unknown or absent observations remain unknown. Both locales supply action labels,
confirmation copy and the new property label.

There are still 31 ordinary methods and eleven C/JNI exports. JSON request schema
1.0, SQLite schema 4 and protected payload 3 remain unchanged. The unreleased Rust
grant bundle gains a `dishwasher_cycle` slice; callers may use its Default value.
Older native libraries reject the new startup field; negotiate the API version.

## Acceptance

Exercise exact source payloads, independent grant scope, excluded/missing/conflicting
profiles, forged public metadata/risk, no automatic read/cache mutation, retained
receipts, pre-send refusal, accepted/rejected/unknown outcomes, cancellation and
deadline cleanup. Validate startup schema/runtime parity and actual C approval
admission with fictional descriptors and no account. Run workspace/runtime,
presentation and Android package checks. Flutter consent UI, physical behavior,
off-LAN acceptance and the selected owner's product match remain unverified.
