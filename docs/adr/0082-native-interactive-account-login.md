# ADR-0082: Additive native interactive account login

Status: accepted for implementation; C exports and Flutter work pending.
Date: October 25, 2026.

## Context and authorization

The owner requested completing HomeVerb's native implementation using the
community integration after the HA reuse assessment. ADR-0081's tested typed
Rust protocol, protected account and session registry now need a callable native
UI boundary. No permission to access real accounts, operate appliances, publish
packages or push branches follows from that implementation request.

## Decision

Add three dedicated trusted C functions in native API 1.28.0:

- `homeverb_begin_account_login`: existing ConnectRequest-shaped nonsecret metadata
  with explicit `xiaomi.mijia` provider, plus bounded writable account/password
  buffers. Input buffers are consumed and cleared using the existing convention.
- `homeverb_continue_account_login`: operation metadata and the session-owned
  challenge ID, plus a separately cleared answer buffer. A decoded request consumes
  its challenge before host admission. Answers cannot select another account.
- `homeverb_discard_account_login`: discard a session-owned challenge by handle.

The dedicated result is either a sanitized account-connection report or a bounded
PNG/JPEG captcha with its opaque local ID and remaining lifetime. No account URL,
service cookie, password, password digest or protected subject is returned.
The existing four-entry, three-minute, three-answer limits and lifecycle
invalidation from ADR-0081 apply. Ordinary JSON, CLI and MCP gain no login method.

## Backward compatibility

This is an additive minor version, not a breaking migration. All 19 existing C
functions, their signatures and result shapes stay unchanged; the new total is
22 C functions plus the Android initializer. All 34 ordinary JSON methods remain
unchanged. Existing clients continue to use `homeverb_connect_account`; only the
new interactive path requires API 1.28. Older binaries remain usable for old flows,
and new UI code must check 1.28 before resolving or invoking the added symbols.
SQLite schema 5, Mijia payload 1 and Meiju payload 3 remain unchanged. No persisted
account, credential, device, grant or receipt migration is performed.

## Exact contract synchronization scope

`packages/native-contract/api.json` supplies the compiled `get_api_info` version.
Its version changes to 1.28.0 without modifying the ordinary methods or limits.
The current contract manifest/example files share this version and must be kept
consistent: `approval.json`, `approval-examples.json`, `account-examples.json`,
`connection-link.json`, `connection-link-examples.json`, `identity-recovery.json`,
`identity-recovery-examples.json`, `lan-scan.json`, `lan-scan-examples.json` and
`examples.json`. Only their api_version fields change; examples.json's embedded
get_api_info data receives the same version. Historical documentation is retained.

New `account-login.json`, `account-login-examples.json` and
`schemas/native-account-login.schema.json` describe only the added functions.
The exact version assertion in `packages/ios/acceptance/main.m` follows the new
compiled metadata; editing that assertion does not claim an executed iOS test.
The C header, Android/iOS export verification, native bridge verifier and Flutter
version negotiation must be synchronized before delivery.

## Validation and delivery gates

Validate positive/negative schemas, native buffer clearing/refusal, fictional
Host challenge success/lifecycle, unchanged native regressions, and Flutter
captcha/reauthorization behavior. Build matching packages before offering a new
APK. An API version edit alone does not establish a working native feature.
No existing production consumer, remote branch or installed app is changed by
this local implementation phase.
