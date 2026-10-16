# ADR-0052: Link an additional connection through a trusted native preview

Status: accepted for synthetic implementation within the additional-LAN scope.

## Decision

Keep ordinary discovery's exact connection-set guard. Add a separate typed Rust
Core/Session operation to append one connection to an exact existing device ID.
The caller selects an existing anchor connection. The proposed connection must
have the same adapter, account reference and stable identity reference, a new
connection ID and a transport absent from the device. Cross-account/provider
merges, replacement, removal and duplicate cards are outside this operation.

The registered adapter must explicitly implement `validate_connection_link`.
Its default refuses linking. An implementation must validate current protected
identity/model/route evidence locally, without networking or credential mutation,
and guarantee that subsequent discovery includes the established connection set.
The asynchronous hook inherits the Operation so local protected-state lookups
can be cancelled without a blocking credential-store call.
User consent cannot replace that evidence. Mock and Meiju keep the default;
only a synthetic test adapter opts in during this phase.

`prepare_connection_link` returns a non-cloneable, nonserializable immutable
`PendingConnectionLink`, bound to its Core instance and approval generation for
60 monotonic seconds. Show the descriptor, anchor, new connection and existing
grants for the new transport before requesting trusted user consent. Those
grants are potential authority, not proof that every action is currently valid.
Adding a route can activate existing grants and changes automatic read routing;
cloud-only grants remain cloud-only. Linking does not create grants or establish
physical support. Dismissal drops the preview.

`confirm_connection_link` consumes the preview on every attempt. After actual
trusted UI consent, lock the device, revalidate the preview, descriptor and
adapter evidence, and pass an unforgeable `ValidatedConnectionLink` to storage.
The Store default refuses the operation. SQLite compares the current effective
descriptor inside an immediate transaction, checks account-removal admission and
appends only the connection to the raw stored descriptor. Local alias overlays
participate in the comparison without overwriting the provider label. Rooms,
favorites, ordering, cache, history, capabilities and policy remain intact.

Use the inherited Operation, checking cancellation/deadline and preview validity
again immediately before storage admission. Confirmation caps the operation at
preview expiry; an in-flight wait reaching that cap returns `DEADLINE_EXCEEDED`.
An already expired preview returns `CONNECTION_LINK_EXPIRED`. Background/close invalidates pending
previews and cancels active tokens. Cancellation after transaction admission is
not a rollback; a completed local commit returns its descriptor. Do not retry or
infer a missing commit from an interrupted callback; inspect local state first.

## Compatibility and verification

New fixed errors distinguish unsupported linking, stale scope and expiry from
command approval failures. No ordinary JSON, C/JNI, CLI or MCP link method or
`confirmed` field is added. Native JSON API 1.14.1 and all storage versions remain
unchanged. Additive trait defaults preserve existing adapter/store implementors.

Test immutable previews, exact selection, invalid/cross-identity proposals,
default refusal, adapter revalidation, expiry, foreign Core, lifecycle changes,
cancellation, deadlines, locks and concurrent metadata/account removal. Verify
transactional comparison, raw/effective alias handling, preserved organization,
cache/history, repeat refusal and restart. Ordinary refresh must still reject
connection edits, while an opted-in adapter can refresh the full linked set.
Test that existing LAN grants become usable and cloud-only grants cannot send
over LAN. No real account, appliance or physical compatibility test is included.
