# ADR-0080: Exact Mijia reference readings

Status: accepted for implementation.
Date: October 25, 2026.

## Context

The October 22 owner-supplied Xiaomi Home archive contains 19 model records,
17 exact cached specifications and separately recorded read outcomes. The existing
adapter admits only selected fan, lock, doorbell and battery fields and rejects
valid eight-part MIoT URNs. Source definitions do not establish successful reads.

## Decision

Project only public model/specification facts from the digest-pinned archive.
Keep an exhaustive property/action/event coverage ledger with explicit exclusions
and evidence limits. Never copy account identifiers, observed values or private
file references into the compiled catalog. No archived program is executed.

Exact model and complete specification URN select reviewed scalar read mappings.
An absent reported URN may use the single pinned model specification; a conflicting
URN cannot use that fallback. Unknown models retain the existing public-spec path.
Recognize an optional eight-hex-digit URN extension without dropping it from
identity matching. Preserve every component and native scalar code separately.
Only equivalent existing semantics use common capabilities. Other fields use
`vendor.xiaomi.mijia.*` names, without guessed status labels or physical freshness.

Identity/network fields, opaque strings, pairing lists and streaming services are
excluded. Actions/events are not observations. Additional scalar fields are
read-only even if the source declares write access. Existing reviewed fan controls
retain their independent protected domains and exact grants. No additional write,
subscription, timer, login, retry or route is introduced.

Saved legacy mappings remain valid when they exactly match the prior derivation;
an explicit account refresh upgrades them. Compiled facts do not establish a new
firmware response or current membership. Missing/error readings remain absent.
The modern `mijia` session and `core.api.mijia.tech` evidence cannot authorize
sending legacy `xiaomiio` credentials to a different origin.

## Evidence boundary

Offline parser, Core, host and App tests establish software behavior only.
Cloud/cache inventory disagreement, modern standalone authentication/renewal,
specialized BLE/history/media/event protocols and physical acceptance remain
explicit gaps until independent evidence is available. No complete official-App
parity claim follows from the static catalog.
