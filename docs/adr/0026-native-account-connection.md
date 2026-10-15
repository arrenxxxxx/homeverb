# ADR-0026: Connect native accounts with separate secret inputs

Status: accepted for implementation under the owner-approved ADR-0022 scope.

## Decision

Add foreground account listing and explicit inventory refresh to the general native
request channel. Add a separate `homeverb_connect_account` C function for the
trusted application to supply bounded account/password buffers alongside nonsecret
operation metadata. Passwords, tokens and provider subjects never enter general
JSON requests, returned DTOs, SQLite, logs or diagnostics. The first implementation
uses only the selected mainland Meiju host; Mock rejects account connection.

The C caller supplies disjoint writable byte buffers, owns their allocation and
clears/frees them on every path. Native code copies valid bounded buffers into
zeroizing storage and clears those caller buffers before parsing metadata or
starting network work. Lengths are explicit, not NUL-terminated. Invalid pointer
or oversized input is not dereferenced; caller cleanup remains mandatory. This
reduces retention, not a promise to erase copies made by the OS, Flutter or a
managed string runtime. UI credentials must not enter analytics or state persistence.

Reuse the host's eight-operation registry, shared deadline, explicit cancellation,
foreground and close gates for connection as well as ordinary calls. Successful
authentication stores the session and automatically retrieves devices. Then reuse
Core's whole-batch descriptor/identity validation to publish cards without a second
inventory request or a separate user Find Devices step. Exact model read admission,
historical cross-account ambiguity checks and existing preference preservation apply.

Report successful authentication separately from retrieval/publication failure.
A stored session is not proof of current authorization. Local account summaries
contain only random IDs, provider/region, user label and credential-storage state;
authorization is `not_checked`. They do not read credentials, mask a guessed login,
invent a provider permission scope or claim a last-check time. An explicit refresh
uses saved credentials without login. Partial inventory publishes no new batch;
Core storage failure may leave earlier saved cards and must not imply rollback.

## Compatibility and limits

The additive native API is 1.2.0. Existing C symbols and requests retain behavior;
the new C symbol is separate from the general method catalog. The account calls do
not enable controls, removal, revocation, sharing, mobile credential stores,
background renewal or HTTP APIs. Older libraries reject new JSON methods and lack
the new symbol. Check API version/method support before resolving it.

Verification must cover synthetic successful login-to-card publication, reconnect
identity/preferences, failed/partial retrieval, saved-session refresh, unavailable
storage, wrong identity, cross-account ambiguity, cancellation/background/close,
shared operation IDs, schema refusal and caller-buffer clearing. Real saved-account
refresh can be exercised without passwords; real reauthorization still needs owner
input through a trusted local UI and remains separately recorded.
