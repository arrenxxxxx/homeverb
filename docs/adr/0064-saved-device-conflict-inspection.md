# ADR-0064: Inspect saved cross-account device conflicts before reconciliation

Status: accepted for offline implementation; reconciliation remains separate.

## Problem and scope

Meiju discovery rejects equal provider appliance IDs across accounts, including
historical bindings, before publishing any descriptors. The error preserves
existing cards but does not identify the other local account/device references.
A device in the newly connected account may have a protected binding without a
published card. Ordinary card lookup cannot diagnose that conflict.

## Contract

Native API 1.22.0 adds the read-only ordinary method `inspect_device_conflicts`
with an exact local `account_id` and `device_id` (UUID-v4). The result contains
`source: saved_inventory`, `authorization: not_checked`,
`physical_identity: unverified`, one `selected` binding and a sorted `matches`
list from other mainland Meiju accounts. Each binding contains only local
account/device/connection IDs and membership (`listed`, `not_listed` or
`unconfirmed`). No provider IDs, subjects, credentials, product selectors or
endpoints are returned. A binding need not have a published card.

Equal saved provider IDs identify candidates only: neither matching labels nor
a match result proves physical identity, compatible model/profile evidence,
current account authorization or permission to transfer control. Empty matches
means no duplicate in the inspected saved account maps, not global uniqueness.
Do not infer a selected winner from account order, membership or publication.

## Consistency and failure behavior

Hold the registration lock and all relevant account leases until inspection
finishes, using a deterministic account-ID order. Inspect at most 64 mainland
Meiju accounts; reject a larger scope with `MIDEA_CONFLICT_LIMIT` before secret
reads. Hold no network connection. Every operation uses the existing foreground
cancellation and deadline budget and releases leases on any exit.

Validate the selected account and protected device binding before returning any
result. Include historical and unconfirmed memberships. Any unavailable, pending,
removing, corrupt or busy relevant account fails the entire inspection with its
existing fixed error, without partial success or an empty-list fallback. Preserve
all protected entries, descriptors, local organization, cache, history and grants.
No authentication, refresh, credential update, implicit recovery or deletion runs.

## Integration and acceptance

CLI `account conflicts <account-id> <device-id>` calls the same Session method
and returns the same projection; it does not read credentials from arguments.
The native JSON envelope and storage formats remain unchanged. There are 34
ordinary methods; C/JNI signatures are unchanged. Older libraries reject the new
method, so negotiate the API catalog. Both locales provide inspection labels and
copy explaining that a candidate is not confirmation or a merge.

Test duplicate/current/historical/unconfirmed and unpublished bindings, unrelated
accounts, stable ordering and reopening, zero provider calls and secret writes,
no private output, bounded scope, malformed selections, busy/corrupt/unavailable
accounts, cancellation/background/close and lock release. Exercise actual C ABI
admission and response schemas with fictional data. This inspection is the input
to a future explicit reconciliation flow; it does not implement account choice,
route transfer, device merge, real-account verification or physical support.
