# ADR-0097: Native modern Mijia account bridge

Status: implemented in native API 1.36; Flutter integration pending.
Date: October 30, 2026.

## Decision

Native API 1.36 adds four dedicated trusted-UI C functions for ADR-0096:

- `homeverb_begin_modern_account_login`: closed ConnectRequest metadata and a
  caller-owned protected QR output buffer of exactly 8192 bytes.
- `homeverb_continue_modern_account_login`: one closed continuation request.
- `homeverb_discard_modern_account_login`: local idempotent handle disposal.
- `homeverb_modern_account_session`: closed account metadata with exactly one
  explicit operation, `refresh`, `recover` or `renew`.

The begin buffer is writable, exclusively owned, disjoint from metadata and
cleared before host/metadata admission. Invalid lengths and NULL pointers are not
dereferenced. The caller clears/frees every allocation on every path. The response
contains only a local challenge ID, byte length, remaining milliseconds and
`kind=qr`; the sensitive UTF-8 scan URL stays outside JSON. Only trusted local
QR rendering may consume it, with no logging, browser navigation, persistence,
clipboard export or remote image generation. Oversized/empty projection discards
the pending handle instead of leaking registry capacity or truncating scan data.
The adapter already limits reviewed scan URLs to 8192 bytes.

Continuation reaches the existing single-use Host registry and original lifetime.
Syntactically valid typed metadata consumes the handle before operation admission;
malformed JSON or unknown fields fail parsing without interpreting a handle.
The bridge does not retry, regenerate QR codes or poll implicitly. UI must negotiate
API 1.36 before resolving these symbols and explicitly request continuation while
foreground. Shared login limits and cancel/background/close behavior are unchanged.

Connected/directory responses expose only the public account summary and a bounded
logical-entry count, partial/failed status, sanitized issues and `complete=false`.
They contain no provider or logical-device identifiers, credential reference,
entry names, routes or Device IDs. A successful empty directory remains partial;
failed retrieval has zero counted entries and an error. Connected authentication
is `succeeded`; explicit refresh/recovery use `not_attempted`. This phase does not
publish physical cards or reconcile logical children. A renewal response contains
only `kind=renewed` and the public account summary, without a directory operation.
Stored credentials remain saved if a subsequent directory read fails.

The ordinary 36 JSON methods and previous 26 C exports remain unchanged; four
new exports make 30, plus the Android initializer makes 31. No ordinary JSON/CLI/MCP
entry point gains authentication authority. SQLite 5 and Mijia payload 1/2 remain
unchanged. The existing ABI and errors are additive; older libraries lack the new
symbols and must retain their prior behavior. Mock/Meiju-only native hosts refuse
these flows without network access.

## Required evidence

Closed positive/negative metadata/result fixtures, actual C symbol and refusal
checks, valid-buffer clearing on invalid host/metadata, invalid-length/null refusal,
protected projection and overflow cleanup, no private values in public output,
ordinary dispatch refusal, lifecycle/cancel forwarding and the existing fictional
Host success/race tests. Rebuild and run the full native verifier with API 1.36;
update Android/iOS export checks and manifests. These checks do not establish a
rebuilt mobile package, Flutter QR rendering, real login or physical compatibility.

## Executed evidence

The real native library passed 62 focused refusal/buffer/lifecycle cases for all
four new symbols. Protected QR projection and bound/expiry cleanup have unit
coverage; successful fictional Host report projections match the same closed
schemas. Workspace, full ABI, mobile package-unit and static-check evidence is
recorded in the [Mijia guide](../development/11-mijia.md). These checks establish
no QR scan, provider authentication, new APK or physical-device acceptance.
