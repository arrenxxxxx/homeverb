# ADR-0036: Inspect saved account inventory without contacting appliances

Status: accepted for implementation; provider and phone acceptance remain separate.

## Decision

Native API 1.10.0 adds `inspect_account_inventory(account_id)` so a foreground UI
can explain ADR-0035 membership before attempting a live read or control. Read the
selected account's protected payload under its account lease. Return its existing
public account summary and sorted local device/connection IDs with `membership`:
`listed`, `not_listed`, or `unconfirmed`.

This is explicit inspection of saved evidence. It performs no login, cloud request,
discovery, credential mutation, descriptor publication or policy change. It cannot
prove that an appliance is online or that a command will be authorized. The
summary retains `authorization=not_checked`; no current-cloud timestamp is invented.
Live execution must repeat protected admission under its own lease.

Return all saved bindings for that one account, including unsupported and historical
devices. A partial retrieval may have retained a new unconfirmed binding without
publishing its card. The UI only decorates an existing card when both local device
and connection IDs match; it must not create cards from this result or merge devices.
Do not expose appliance IDs, identity references, subjects, session bytes or credentials.

The existing `get_account` remains a metadata-only query available without secret
access. Inspection can fail when secrets are locked/unavailable, account updates
are pending, removal has started, or the account lease is busy. Return an error
instead of an empty list or an invented membership state. Legacy payloads return
unconfirmed entries without migrating or refreshing anything. Provider/region
scope, total deadlines, cancellation and foreground/closed-session rules apply.

## Compatibility and UI behavior

Require the new method in `get_api_info` before calling it. The C ABI remains seven
functions, SQLite schema 4 and protected payload version 2 are unchanged, and all
existing requests remain valid. Older libraries reject the method. Default Mock
hosts return `AUTH_FLOW_UNSUPPORTED`; no account or network work is started.

Use the localized membership labels on explicit account/device inspection. Keep
observed state, membership, capability support and write permission separate.
Offer explicit account refresh for absent/unconfirmed membership. Do not start
polling or automatically retry a previous write after inspection or refresh.
Discard stale UI callbacks after account selection changes or cancellation.

## Verification

Synthetic host tests cover complete/empty/partial inventories, legacy membership,
unpublished bindings, account isolation, redaction, reopening, unavailable/corrupt
secrets, pending/removing/busy accounts, unchanged bytes/cards and lifecycle cleanup.
Validate actual responses and closed requests against the native schemas. The
Linux ABI suite checks method negotiation, unsupported-host and missing-account
admission. No real credential access, appliance control or Flutter execution is
part of this phase.
