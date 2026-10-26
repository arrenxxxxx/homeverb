# ADR-0085: Protected Mijia phone and email verification tickets

Status: typed protocol implemented and synthetic-tested; account/Host/native UI integration pending.
Date: October 25, 2026.

## Evidence and scope

The owner authorized native completion based on the Apache-2.0 community
integration. Revision `f78158257e19a21902adaf6bcc8440c3ba172a91` supplies
`identity/list`, `identity/auth/verifyPhone`, `identity/auth/verifyEmail` and the
post-verification service-login sequence. It does not provide a native endpoint
for sending SMS/email. A ticket comes from explicit user interaction with the
provider verification page; it is not necessarily a numeric one-time code.

## Decision

Add an opt-in typed Rust login outcome with a protected verification challenge,
while preserving existing one-shot and captcha-only behavior. A challenge keeps
only the bounded, zeroizing session material needed for verification, the
provider page, explicit available phone/email choices and its original lifetime.
It is neither cloneable nor serializable. Drop releases it; waiting holds no
socket, account lease, registration lock or background worker. The original
cancellation token, three-minute expiry and explicit single-use answers apply.

Only HTTPS `account.xiaomi.com/fe/service/identity/authStart` can initiate this
flow. The options request replaces that exact path with `/identity/list`, retaining
the query. Only explicit provider options 4 and 8 become phone/email choices.
One selected method receives one form post with `_flag`, `ticket`, `trust=false`
and `_json=true`; there is no fallback to another method or automatic answer retry.

After a code-zero answer, follow only bounded GET continuations at the exact
account `/identity/result/check` path or legacy `sts.api.io.mi.com/sts` path.
A fresh `serviceLogin` for SID `xiaomiio` must provide a valid signing key and
consistent numeric subject before the service token forms a session. Token-only,
foreign/ambiguous identity, redirect loops or incomplete responses do not connect.
No password is resubmitted. Extra `/fe/` confirmation pages remain an explicit
provider interaction requirement; do not parse HTML or follow `skipUrl` to confirm
account settings automatically.

Use a separate one-send verification transport with no implicit redirects,
retries, proxy or cookie jar. Bounded cookies remain in memory, honor origin/path
and expiry, and do not move account credentials to STS. Service tokens are never
sent in verification requests. Validate every returned location before issuing
its request. Credentials and URLs do not enter ordinary JSON, logs, configuration
or SQLite. Existing cloud session storage remains unchanged.

## Rollout and acceptance

The first phase is typed protocol and HTTPS transport, tested with fictional
in-process peers and local TLS. Existing account/Host/C/App paths remain captcha-only
until a dedicated trusted browser/ticket boundary preserves account snapshots,
lifecycle and protected buffers. This phase does not publish an account or change
native API 1.30, 34 ordinary methods, C exports or persisted payload schemas.
The browser must complete provider interaction without background networking or
plaintext credential persistence in HomeVerb. Native/UI integration must not be
reported complete from protocol tests.

Test both methods, absent/unsupported options, malformed/expired cookies, subject
correlation, the no-location signing-key path, bounded redirects, rejected tickets,
no password replay, cancellation/expiry/drop, buffer bounds and redacted debug.
Additional provider confirmation, modern SID login and physical acceptance remain
separately tracked until concrete evidence and implementation cover them.
