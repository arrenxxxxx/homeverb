# ADR-0081: Bounded interactive Mijia community login

Status: accepted for staged implementation; native bridge and App work follow the protocol phase.
Date: October 25, 2026.

## Context

The owner authorized supplementing HomeVerb using the Apache-2.0 community
`hass-xiaomi-miot` integration. The existing native `xiaomiio` flow stops on every
captcha. Community revision `f78158257e19a21902adaf6bcc8440c3ba172a91` documents
captcha image/cookie acquisition, protected `captCode` submission and user-driven
relogin. Official HA OAuth identity and restricted HA code are not reused.

## Decision

Add a typed native login outcome that either contains a validated session or an
owned captcha continuation. Keep the existing one-shot login behavior unchanged.
A continuation retains only zeroizing account/password-digest/sign/cookie material,
not the plaintext password. It cannot be cloned or serialized. Its image can be
displayed only through a dedicated native flow; URLs, cookies and account identity
never enter ordinary JSON, logs or SQLite. No account is published before a fully
validated STS result and protected identity reconciliation.

Only a provider-returned HTTPS URL on `account.xiaomi.com`, with no userinfo,
fragment, non-default port, control characters or network-path reference, can
supply a captcha. The dedicated fetch uses a fresh TLS client, no redirects,
proxy override or retries, and no cloud service token. Admit bounded PNG/JPEG
images with bounded dimensions and exactly one valid `ick` cookie. Unsupported
images or malformed replies fail closed without following another URL.

Each answer consumes its continuation; rejected answers may produce a replacement
image/cookie within the original three-minute lifetime and three-answer limit.
Each request remains bounded by its own Operation. Cancellation of the original
operation invalidates later continuation. Host integration must keep a bounded,
session-owned registry and clear it on background/cancel/close. Waiting for input
holds no sockets, account lease, registration lock or worker. No automatic
password resubmission, challenge solving or write replay is introduced.

The first phase covers graphically displayed captchas only. Phone/email identity
verification, protected native bridge, Flutter completion and real-account
acceptance are separately tracked; none is implied by protocol tests. Existing
SQLite/protected payload formats and ordinary CLI/MCP login refusal are unchanged.
