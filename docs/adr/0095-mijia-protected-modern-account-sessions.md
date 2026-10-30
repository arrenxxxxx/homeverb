# ADR-0095: Protected modern Mijia account sessions

Status: accepted for implementation; native Host/App integration pending.
Date: October 30, 2026.

## Decision

Persist independently authenticated SID=mijia material through the existing
system-secret-backed AccountVault, using a version-2 Mijia protected payload.
A single local provider/account identity may retain both its existing legacy
session and a separate tagged modern reference session. The modern tag fixes
SID, mainland origin and continuous-stream read profile; no arbitrary origin,
codec or SID is stored. Modern-only accounts have no legacy slot or bindings.
Both slots must have exactly the same protected numeric subject when present.

Version-1 payload encoding remains unchanged, with legacy material mandatory.
Version 2 requires the modern slot and permits an optional legacy slot. All
unknown fields/profile tags, malformed subjects/client identities and copied
account/credential references fail closed. Older implementations cannot read
version 2; use a matching implementation rather than rewriting it as version 1.
SQLite schema 5 and ordinary account metadata remain unchanged. Neither private
subject, client identity, token nor signing key enters plaintext SQLite or JSON.

An explicit typed modern-transport composition adds QR begin/finish, saved modern
directory inspection, exact-property reads, renewal and uncertain-save recovery.
Construction still performs no I/O. Existing native hosts and ordinary JSON do
not enable these entry points in this phase. The default adapter still refuses
modern protocol operations before networking when it has no modern transport.

QR preparation checks secure storage and captures the protected account set
under the registration lock. No account lease is retained while a user scans.
The single-use continuation belongs to its adapter instance and selected local
account. It revalidates the original snapshot before completion and after the
provider exchange. Removal, relogin or protected-byte changes cannot be undone
by a delayed response. Original cancellation and deadline cover final persistence
as well as the provider poll/exchange.

A successful modern login matches the protected subject across existing accounts.
An explicit selected account must match; an unselected login reuses its unique
matching account rather than duplicating it. Adding/replacing a modern slot
preserves existing legacy bytes, device bindings, labels, grants and local IDs.
Legacy reauthorization may likewise add/replace its own slot without erasing
modern evidence. No token crosses a protocol boundary. Existing Core device
reads, controls and history continue to require the legacy slot; no modern
fallback or new control authority is introduced.

After protected authentication storage, an explicit connection retrieves the
modern directory once and returns opaque account-bound entries. They retain
logical children and do not publish physical Device cards. Entry reads require
the same local account and credential reference plus the protocol's subject
checks. Native Core reconciliation of these entries remains separate work.

Renewal is explicit on an active protected modern session, holds its account
lease throughout, preserves both identity and legacy evidence, and saves only
the successfully renewed modern slot. It does not enumerate or replay reads or
writes. An uncertain protected write leaves the account pending. Explicit
recovery reads the exact saved modern session, validates it with the bounded
directory operation, then confirms those same bytes; it does not log in, renew,
rewrite unchanged credentials or infer complete inventory. Failed recovery keeps
pending metadata and prior bytes intact. Cancel/drop releases account locks.

No C/JSON API, App flow or new appliance write is added in this phase. Foreground
Host lifecycle ownership, QR rendering, protected entry adoption, logical-parent
reconciliation and actual provider/device acceptance remain pending.

## Required evidence

Version-1 byte stability and version-2 round trips; restart and explicit renewal;
subject/profile/client identity tamper refusal; same-subject adoption without new
account/device IDs; legacy and modern slot isolation; pre-network unavailable
storage; owner/snapshot/lifecycle refusal for QR continuation; uncertain save and
exact-byte recovery without replay; concurrency, cancellation, deadline and drop
lease release; absence of private markers from SQLite/public debug; old read,
control and account regression suites remain passing.
