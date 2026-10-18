# ADR-0070: Independent native Mijia cloud integration

Status: accepted for implementation; account and physical acceptance pending.
Date: October 18, 2026.

## Context

The owner requested Mijia integration and will perform live acceptance. HomeVerb
must remain an independent native client without HA, a project backend, shared
confidential application credentials, or background workers. Official Xiaomi Home
HA OAuth configuration does not establish a HomeVerb client registration.

## Decision

Implement a separate `homeverb-mijia` adapter with provider ID `xiaomi.mijia`.
Use the community native `xiaomiio` account flow and region-specific cloud
requests. The protocol uses the user's password transiently and the resulting
user-scoped session, with no shared application secret. Do not borrow the HA OAuth
client identity. The reviewed MIT source and pinned protocol evidence belong in
the [integration guide](../development/11-mijia.md).

All requests share their caller's bounded Operation. Transport permits only
reviewed HTTPS origins and paths, no redirects, retries, proxy override, retained
listeners or workers. Challenge responses require further user-side authentication;
do not bypass them or print private challenge URLs. Passwords, provider identity,
tokens and device bindings remain protected native data and system-store payloads.
Login cannot enter ordinary JSON, CLI arguments or MCP tools.

Stable random local IDs and protected account/region membership precede Core
publication. Inventory inclusion is neither live state nor proof of support.
Device descriptors must derive from actual MIoT service/property evidence;
legacy devices without reviewed evidence remain visible as unsupported. Do not
infer semantics or write permission from brand, category, name or an online flag.
Cloud readings without trustworthy source time remain stale and cannot verify a
write. Device writes need independent reviewed semantics and exact grants; initial
protocol/read work does not admit arbitrary MIoT setters or actions.

## Consequences

This is experimental community compatibility, not official endorsement. Account
challenges, regional service changes and incomplete inventory must fail explicitly.
Real account, exact model, phone, off-LAN and physical acceptance are owner work.
Existing Mock/Meiju defaults, storage schema and write semantics remain unchanged.
Native API and App additions require synchronized versioned contracts and tests
before being advertised as available. A protocol library alone is not App support.
