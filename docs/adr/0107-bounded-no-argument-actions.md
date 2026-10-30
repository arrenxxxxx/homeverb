# ADR-0107: Bounded no-argument MIoT actions

Status: Typed Rust protocol implemented with fictional tests; Core/native/App admission remains pending.
Date: October 30, 2026.

## Evidence

The pinned Apache-2.0 community revision f78158257e19a21902adaf6bcc8440c3ba172a91
[cloud client](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/xiaomi_cloud.py)
posts /miotspec/action with params as an object. Its
[device client](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/device.py)
supplies did, siid, aiid and in. Its
[specification helper](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/miot_spec.py)
retains action input/output property references and distinguishes argument failures,
action execution errors and operation timeouts. The October 22 archive independently
lists the modern /miotspec/action candidate at the reference origin and supplies
exact action definitions. Neither source establishes real HomeVerb action acceptance.
The modern profile composes that route evidence with the independently implemented
SID=mijia continuous-stream transport; its runtime acceptance remains unverified.

## Initial exact scope

Project only these six public action definitions into a separate pinned catalog,
without changing existing property catalogs or protected payloads:

| Exact model | SIID/AIID | Source action |
|---|---|---|
| abhome.light.wy0a01 | 2/1 | toggle |
| abhome.light.wy0a01 | 2/2 | brightness-up |
| abhome.light.wy0a01 | 2/3 | brightness-down |
| abhome.light.wy0a01 | 2/4 | toggle-light-mode |
| jdjz.fan.2072fc | 2/1 | toggle |
| xiaomi.hood.cyp10 | 6/1 | illumination toggle |

All six declare empty in and out arrays. Bind model, complete device/service/action
URN and exact SIID/AIID before sending. Require a matching exact specification;
never infer an action from its name, a source write flag or an arbitrary caller key.
The hood target is illumination, not ventilation. Brightness increments and mode
cycles have no invented step, wrap behavior or physical-completion claim. All six
are non-idempotent: repeating one can reverse or compound its effect.

## Tracked protocol decision

Use the protected selected protocol only: legacy SID=xiaomiio at api.io.mi.com/app
with per-field encoding, or modern SID=mijia at core.api.mijia.tech/app with continuous
reference encoding. Preserve the complete DID and modern subject/kind checks.
Send one params object with in=[]; accept no caller parameters, output fields,
endpoint or conversion. Use the independent tracked control transport, never the
read path. Existing TLS, redirect, body-size, gzip and operation bounds apply.
There is no retry, renewal, route switch or protocol fallback.

Only a decrypted code-zero envelope with a single matching result object and
matching DID/SIID/AIID can acknowledge acceptance. Empty/omitted out is permitted;
nonempty outputs are not exposed or interpreted. Code zero is accepted, not verified.
Code 1 (not completed), execution errors, return-value errors, timeouts and unknown
codes remain possibly-sent/unknown. Only reviewed argument/identity/missing-action
pre-execution rejection codes may produce rejected: -4003, -4006, -4007, and the
-70xxxxxxx family suffixes 005, 008, 025 and 035. Other codes must not be guessed
into proven rejection. Authentication errors can request reauthorization while
preserving uncertain progress; they never trigger automatic renewal or replay.

Cancellation, deadline and a dropped future release transport work. After-send
cancellation is not a rollback. A late response cannot reinstate acceptance after
operation cancellation. One consumed WriteProgress cannot be sent again. The
low-level primitive does not replace Core's persistent request-ID/replay guard.

## Delivery and compatibility

This first phase exposes typed native-library protocol methods only. It adds no
Core action, host method, startup grant, CLI/MCP tool or Flutter control. Native
API 1.39, ordinary schemas, C exports, SQLite 5 and protected payload 3 stay unchanged.
Extending the public Rust CloudEndpoint enum requires unpublished exhaustive
consumers to handle the new variant; existing property requests keep their profile.
A later phase must independently admit exact action bindings and grants, declare
non-idempotency, preserve request-ID receipts across reopen/protocol adoption and
version native/App support before claiming user-callable actions.

Remote deletion, resets, camera/media operations, outlet or wall-switch toggles,
parameterized actions and all unreviewed actions remain denied. No real account,
appliance write, installation or publication is authorized by protocol tests.

## Required evidence

Test every admitted tuple on both selected profiles, exact empty-input encoding,
complete root identity, wrong model/spec/kind/subject refusal before send and no
read-path substitution. Exercise correlated acceptance and proven rejection,
ambiguous codes, plaintext/foreign/duplicate/missing replies, size/compression
bounds, cancellation/deadline/drop/late response and consumed progress. Local TLS
checks must prove one connection, no redirect retry, certificate refusal and
resource release on both action endpoints. Existing property controls must regress
unchanged. Record typed-only scope and remaining entry-point work explicitly.

## Executed evidence

The implementation and its software verification are recorded in the
[phase record](../development/14-mijia-reference-integration.md#typed-no-argument-actions--october-30-2026).
This is typed protocol evidence only; Core/native/App action admission and
real provider/phone/model acceptance remain incomplete.
