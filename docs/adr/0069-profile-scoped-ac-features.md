# ADR-0069: Admit reported profile-scoped AC features

Status: accepted for the owner's October 17 request to integrate the selected
air conditioners' available functions. Physical acceptance remains separate.

## Decision

Extend the pinned Meiju mapping beyond the four existing basic controls. Select
the optional feature catalog using protected product evidence, subtype before
product before explicit wall category, and retain the four reviewed model gate.
Do not use the generic family default or fall through from excluded product
overrides. The catalog records protocol facts from the source revision cited in
the AC parity guide; source Python is never loaded or executed by HomeVerb.

Each setting has its own scalar action, exact argument domain and wire mapping.
Eco can require two fields; no-wind sensing and anti-draft remain distinct;
direction codes are discrete positions, never percentages. Only reported,
recognized optional states expose Flutter controls. This is experimental mapping
and field-availability evidence, not model-wide physical certification.
An optional boolean property descriptor, `control_requires_observation`, expresses
this presentation rule without a provider branch in the UI. Earlier strict
descriptor consumers must be upgraded before reading revision-4 AC definitions.

Read-only optional queries share the existing Core operation and deadline.
Native startup limits optional observations and supplemental queries to the devices
in the exact feature grants. Basic-only saved descriptors continue to read until
explicit selected-device setup refreshes their metadata. Skip queries whose relevant fields were already returned; do not query
unmapped energy or environmental groups merely because upstream lists them. The
mandatory default query must succeed. Optional query failures retain the default
result and add a fixed warning; malformed observations are never published.
There are no retries or independent background workers. All observations retain
cloud provenance and unproven freshness. A feature write rechecks protected
identity, membership, model and profile, then performs bounded prerequisite reads
before its one control request. Missing required state refuses before send.
Other supported swing axes must be reported in that operation's reads. Preserve
a reported valid buzzer preference; omit an absent preference, as basic native AC
commands already do. Never copy the upstream client's default or stale Core cache
into a command. A malformed reported preference refuses before send. Concurrent
changes in another vendor client can still race these reads.

API 1.23 adds startup-only `meiju_ac_feature_grants`, at most 1024 exact canonical
local-device/feature/value entries. The adapter opt-in grants no authority by
itself. Public descriptor weakening cannot bypass its private profile contract.
Ordinary requests, CLI and MCP cannot mutate permissions. Default native startup
remains read-only. Existing method and C symbol counts, SQLite schema 5 and
protected payload 3 remain unchanged; older native libraries reject the new field.

Keep accepted/unknown/rejected outcomes, receipt replay guards and no write retry
or route fallback. A code-zero response is not physical verification. Later App
state reads remain separate. Missing optional values are not invented from the
requested command or from source defaults.

## Limits and verification

Do not enable unreviewed conditional/coupled functions, disinfection, self-clean
or device scheduling using a low-risk generic setter. Ordinary wall sleep presets
are commented out in the pinned source and remain absent; explicit product
profiles with an active `energy_save` sleep pair retain that separate mapping.
Kitchen cooking helpers and the unidentified `remove_arofene` function are not
admitted by this review. Never infer them from a raw response field.
The exact official App feature inventory and physical firmware behavior remain
acceptance inputs; absent integration must not be labelled absent hardware.

Use fictional profiles for selector precedence, wire values, omitted-feature
handling, prerequisite failures, tampered descriptors, exact grants, cancellation,
uncertain transmission and replay. Verify Flutter hiding/fallback/labels and the
native APK independently. No new physical write is required by offline tests.
