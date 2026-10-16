# ADR-0057: Inspect current session grants independently of saved capabilities

Status: accepted for synthetic implementation within the native UI scope.

## Problem

Device descriptors can retain actions from an earlier granted session. Closing
that session and reopening read-only already revokes its execution authority,
but does not rewrite the saved descriptors. Rendering an action as authorized
solely because it is present can mislead the user until another discovery.

## Decision

Native API 1.17 adds `inspect_session_grants` with exact local `device_id` and
`connection_id`. Core validates the selected local descriptor/connection and
projects matching rules from its immutable trusted policy. The query requires
neither credentials nor an adapter call, and works for Mock and Meiju.

Return those local IDs, the selected `transport`, `source: session_policy`,
`provider_authorization: not_checked`, and `grants`. Each rule contains
`component_id`, `capability`, `action` and `argument_schema`; all apply to the
selected device and transport. Keep complete argument schemas intact: do not
merge grants into a broader range or combine independently authorized fields.
Ignore invalid schemas exactly as command authorization does. Exclude grants
for other devices/transports. Preserve source order; duplicates confer no extra
authority. The query does not export provider identities or account records.

These are configured local grants, not executable plans, supported capabilities,
live provider permission or trusted user approval. A rule may refer to an action
absent from the current descriptor; risk, protected model/profile, membership,
credentials, routing and per-invocation consent still gate execution. Elevated
actions remain denied by ordinary plan/execute; high-risk actions remain denied.
Never import this response as policy or use it to grant missing authority.

After reopening with a different trusted grant list, the result changes immediately
without discovery, credential access or descriptor mutation. Empty grants mean no
local write permission on that route. UI permission displays use this result;
device capability displays continue using the saved descriptor. API 1.16's
protected control options remain the separate source for proposed permissions.

The operation uses existing cancellation/deadline and foreground/close admission.
It changes no descriptors, cache, credentials, receipts or policy and sends no
network traffic. No new C exports, storage formats, CLI/MCP tools or permission
mutation methods are introduced. Negotiate API 1.17 and the advertised method.

## Verification

Test exact device/component/action/transport scopes and complete argument rules,
invalid schemas, unchanged elevated/high-risk denial, identity failures and
cancellation. Use fictional Meiju accounts to retain writable descriptors across
revocation while denying secret access and proving empty grants/unchanged receipts
without network calls. Exercise actual C queries for Mock and fictional Meiju,
closed request/response fields, lifecycle and reopen. Physical/mobile acceptance
and provider authorization remain unverified.
