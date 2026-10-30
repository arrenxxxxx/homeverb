# ADR-0114: Exact Mijia switch and outlet settings

Status: Native and Flutter implementation verified with fictional evidence; physical acceptance pending.
Date: October 30, 2026.

## Decision and evidence

The digest-pinned October 22 reference specifications and the existing reviewed
legacy/modern property-set envelope admit seven additional exact scalar controls.
Every source must match the protected full device/service/property URNs, readable
and writable access, format and domain before admission. No brand-wide switch
support or wire selector is exposed to upper layers.

| Model | Component | Property | SIID/PIID | Domain | Risk |
|---|---|---|---|---|---|
| opple.switch.0010d5 | switch-3 | power.state | 3/1 | Boolean | elevated |
| opple.switch.0010d6 | switch-11 | power.state | 11/1 | Boolean | elevated |
| opple.switch.0010d6 | switch-12 | power.state | 12/1 | Boolean | elevated |
| yutai.plug.jdls1 | switch-2 | power.state | 2/1 | Boolean | elevated |
| yutai.plug.jdls1 | switch-2 | vendor.xiaomi.mijia.switch.default_power_on_state_2 | 2/2 | 0, 1, 2 | elevated |
| yutai.plug.jdls1 | indicator_light-4 | vendor.xiaomi.mijia.indicator_light.on_1 | 4/1 | Boolean | low |
| yutai.plug.jdls1 | custom-7 | vendor.xiaomi.mijia.custom.child_lock_7 | 7/7 | Boolean | elevated |

The custom child-lock setting controls the outlet's local input, not a door lock.
It does not inherit purifier lock semantics. Raw startup-state codes remain exact
provider values, without a common brand-wide interpretation. Setting power sends
one absolute Boolean target; it never sends the existing toggle action. Indicator
writes do not send power, and input-lock writes do not imply power or timer changes.
Timers, voltage/power limits, charging protection, pairing, schedules and opaque
custom fields remain separate missing coverage.

## Authority, execution and compatibility

Use the independent outlet adapter family and exact per-device/component/value
Core grants. Power, startup behavior and local-input locking require the existing
immutable trusted preview and actual user consent for every attempt. The adapter
checks protected risk independently of editable public metadata. The indicator
alone is low risk. Fan/reference grants cannot authorize these targets.

All setters use set_value idempotency with unavailable verification. Correlated
acceptance does not confirm physical power, startup behavior or input-lock state.
Timeout, cancellation, late/foreign acknowledgement and lost replies retain the
existing unknown receipt without retry, credential renewal, protocol fallback or
companion writes. Reopening and history retirement do not authorize replay.

API 1.43 extends existing mijia_outlet_grants with switch-3/11/12 state,
indicator_light-4 on_1 and custom-7 child_lock_7. Existing switch-2 tuples are shared
only where complete protected evidence proves equivalent parameter domains.
The shared 64-entry limit, canonical UUIDs, closed fields and Mijia/combined-only
startup remain. No method, export, command outcome, SQLite 5 or protected payload
3 changes. Read-only old mappings remain valid; explicit refresh upgrades them.
Older libraries reject new tuples; Flutter must negotiate before supplying them.
The API 1.42 APK remains unchanged until a later clean-source build.

## Verification requirements

Exercise every target and source mutation on both protocol selections with
fictional transports. Test independent grants, actual trusted approval, forged
public risk, incorrect arguments, protected identity changes, read-only defaults,
local inspection, exact one-property requests and uncertainty without replay.
Validate exact startup/schema/C ABI admission and rejection. Flutter and package
adoption are recorded separately. No real account or appliance request is needed.

Executed native verification: Rust 1.96 offline/locked workspace tests passed
697 tests with one ignored, excluding homeverb-midea's known macOS LAN-alias
failures and using --test-threads=4. The Mijia suite passed 271 unit and 14
compile-fail tests; focused Host account/grant suites passed 68 tests. Strict
all-target workspace Clippy and FFI build passed. Actual C ABI checks passed
36 methods and 84 live examples, including both startup modes and invalid new
tuples; modern-account export checks retained 76 boundary cases. Contracts passed
25 schemas, 83 positive and 1,855 negative fixtures. Documentation, source hygiene,
reference projection, both projection unit tests and diff whitespace checks passed.
An initial default-parallel run exposed a stale supported-model assertion, which
was corrected, and two existing 100 ms deadline tests failed under CPU contention;
both passed in the bounded-concurrency package and workspace reruns. No physical
or provider acceptance follows from these fictional tests.

## Flutter adoption

The controller and native client both negotiate API 1.43 before new-tuple startup.
Existing switch-2 permissions retain their earlier minimum version and do not
infer a model. The generic control panel and immutable preview present the exact
selected component and target; scoped power and local-input lock copy are
localized. Older libraries reject before any granted reopen. The existing outlet
fixture preserves its original four-option scope, so the new schema does not
silently widen old test fixtures. No new ordinary method or saved format is added.

Executed Flutter verification: 175 tests across 18 affected control, permission,
component, native-contract, localization, approval and foreground-refresh files
passed under Flutter 3.47.7. The new suite has 20 tests, including all seven fields
in both languages at 320 logical pixels with double text size, actual API 1.42
refusal, current native startup in both modes, explicit targets/routes,
single confirmation and background discard. Flutter analysis reported no issues;
seven shared presentation tests passed. These focused regressions do not resolve
the eight previously recorded unrelated macOS visual/legacy-click full-suite
failures. No real account, appliance write, phone acceptance or remote CI is claimed.
