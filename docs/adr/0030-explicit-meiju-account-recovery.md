# ADR-0030: Recover interrupted Meiju credential persistence explicitly

Status: accepted for native implementation with synthetic account evidence.

## Decision

Native API 1.7 adds `recover_account(account_id)`, an explicit foreground operation
for the admitted mainland Meiju provider. The existing account/vault can be left
`pending` after interrupted or uncertain secure-store writes. Ordinary reads,
refresh and reconnect remain unable to use that unconfirmed material automatically.

Recovery holds the registration and selected account leases. It reads the exact
protected reference, validates the bounded payload's account/provider/region/reference
and established bindings, restores the native session, and matches its subject to
the protected account subject. Then it performs one complete inventory retrieval
using that session. Authentication rejection, malformed data, partial retrieval,
cancellation or deadline expiry leaves the account pending. Missing credentials
cannot be recovered by this method; never initialize a replacement
map or choose another account automatically.

A successful inventory validates that the saved session is currently accepted for
those read endpoints. It is not a new password login, an independent proof of the
owner's identity, evidence of physical compatibility or a control permission. The
subject association comes from the earlier admitted login and protected payload;
inventory does not invent a new subject from device names or ownership guesses.

Before confirmation, reread and compare the exact protected bytes while retaining
the lease. Mark them stored only if unchanged, without repeating their secret write.
Then bind the retrieved inventory through the ordinary protected mapping path and
publish via the existing cross-account identity and Core descriptor guards, without
a second inventory request. A newly seen device may require a distinct binding-map
update; that normal mutation can itself become pending and must be reported honestly.
Known IDs, aliases, rooms, favorites, credentials and grants are never reset.

The query result reuses the account refresh shape: account summary,
`authentication: not_attempted`, and separate retrieval/publication outcome. A
failure after selecting the account includes its current credential state with
the nested retrieval error. Admission failures use the outer query error. A stored
account follows ordinary explicit refresh, allowing a user-reviewed retry after
an interrupted response. A removing account is refused. No startup, page opening,
read, reconnect or background task initiates recovery.

## Compatibility and verification

The ordinary catalog grows to 30 methods. Seven C symbols, SQLite schema 4, protected
payload version 1 and appliance command/receipt schemas stay unchanged. Older
hosts reject this method; Flutter checks the catalog before enabling it. MCP stays
read-only and exposes no account mutation method.

Synthetic host tests must cover interrupted initial connection and mapping updates,
stable identities/preferences, no repeated secret write for unchanged bindings,
partial/expired retrieval, missing/corrupt/foreign material, changed bytes before
confirmation, SQLite confirmation failure, cancellation/deadline/background/close,
lease contention/release, and failed publication after confirmation. Actual C ABI
admission uses fictional profiles without reading the owner's keyring. No live
account recovery, physical control or mobile acceptance is claimed.
