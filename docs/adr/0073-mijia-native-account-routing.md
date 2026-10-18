# ADR-0073: Mijia native account routing

- Status: Accepted
- Date: 2026-10-18

## Context

The owner requested mainland-China Mijia accounts and named device cards alongside
HomeVerb's existing integration. The protected Core implementation in ADR-0072
needs a foreground native host entry point and an honest partial inventory result.

## Decision

Native API 1.24 adds explicit `xiaomi.mijia` and `homeverb.platforms` startup modes.
The latter registers both reviewed Meiju and Mijia adapters in one embedded Core
and shared account vault. Omitted startup still selects Mock. Existing Meiju mode
and exact Meiju grants retain their meaning; Mijia starts read-only. Construction
never authenticates, reads credentials or discovers appliances.

The existing protected connection function accepts optional nonsecret `provider`
metadata (`midea.meiju` or `xiaomi.mijia`). Omission selects Mijia in Mijia-only
sessions, and Meiju in existing or combined sessions. A provider can only select an
already registered adapter, not a different principal or authority. Explicit local
account reauthorization must match that provider and its protected subject before
binding updates. Passwords still use separately bounded, erased native buffers.

Ordinary account operations route by saved local account metadata, never a supplied
provider or private vendor identifier. Local lists include the registered providers
only. Account-specific refresh/recovery publishes positive descriptors through the
matching adapter. Partial Mijia retrieval can return positive `device_ids` with a
sanitized error; failed publication returns no new IDs. Complete coverage is not
inferred. Meiju partial behavior is unchanged. Strict older response consumers
must negotiate API 1.24 before enabling the new startup modes.

Read-only inventory inspection reports Mijia membership as `unconfirmed`, because
this field means membership in a complete saved inventory and Mijia has none.
Positive bindings still admit exact device reads; they never prove absence.
Meiju-specific control-option/conflict/scan features remain unavailable for Mijia
until independently implemented. No lock control, background work, relay, new C
export, SQLite migration or protected-payload change is introduced.

## Consequences

The existing account/card/read methods serve Mijia and combined sessions without
shelling out or introducing a service. Flutter connection and capability presentation
remain a later phase until implemented and checked. Real authentication challenges,
regional coverage and physical behavior remain owner acceptance work.
