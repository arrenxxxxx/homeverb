# ADR-0072: Protected Mijia account identity and Core reads

Status: accepted for implementation. Date: October 18, 2026.

## Context

The owner selected mainland-China Mijia, all returned devices including named
unsupported cards, and fan/doorbell/door-lock adaptation. The native protocol in
[ADR-0070](0070-mijia-native-cloud.md) is insufficient by itself: Core needs stable
local identity, protected account ownership and reviewed model/property evidence.

## Decision

Add a compile-time `MijiaAdapter` and explicit read-only Rust host composition.
Startup is local. Trusted connection consumes transient credentials and uses the
existing AccountVault registration/account leases and system secret store. Saved
Mijia payload version 1 binds the exact local account/credential references,
provider `xiaomi.mijia`, region `CN`, provider-validated subject, user-scoped
session and random local device/connection/identity IDs. Public cards contain no
provider device IDs, tokens or credential references. SQLite remains schema 5;
the separate Meiju protected payload stays version 3.

Reauthentication must match the original protected subject. An unselected exact
subject reuses its account; ambiguous subjects and cross-account physical IDs
fail without automatic merge or authority transfer. Pending-account reauthorization
requires intact protected identity and exact unchanged bytes. Explicit saved-session
recovery uses a bounded authenticated read and exact-byte confirmation, without
another password login or blind secret-store retry. Local rename/removal reuse
the vault's existing scope and cleanup rules, including unavailable/unreadable
credential handling. Removal never revokes the provider account.

Each positive inventory member retains a binding, including unsupported models.
Public MIoT metadata is optional for a card: failure preserves a named placeholder.
Mapping attempts share a shorter child deadline that reserves half the remaining
operation (at most five seconds) for metadata; it never extends the parent budget.
Saved source specifications and their derived mappings must revalidate together.
Existing mapped devices need no spec fetch on ordinary state reads.

Current community enumeration does not prove complete account membership.
Publish positive results only through the explicit account refresh/connection
result, with partial coverage retained. Never remove missing historical bindings
or write a complete-inventory timestamp. Generic discovery lacks a partial-result
envelope and refuses populated Mijia accounts before credential/network access,
directing hosts to explicit account refresh. Empty startup discovery remains local.

Core reads recheck exact protected device/model/component/connection membership,
then request only reviewed readable SIID/PIID pairs. Return typed scalar values,
cloud source, local receipt time, absent source time and `stale: true`. Neither
online hints nor inventory membership becomes power, lock state, freshness or write
verification. Core validates the full snapshot before replacing cache.
All adapter actions remain unavailable, independently of public descriptors and
Core grants; later fan controls require a separate implementation phase. Door-lock
writes remain prohibited. No subscriptions, polling, automatic login or LAN route
is introduced.

## Consequences

The Rust library can connect, refresh, recover, publish cards and read through
Core. Native C/JSON selection, App presentation and physical acceptance remain
separate work until their versioned contracts and tests are implemented. This
does not claim any real model or account is verified. Explicit regional enumeration
and shared-home completeness remain owner-acceptance items.
