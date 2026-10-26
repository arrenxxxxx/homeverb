# ADR-0086: Native protected account verification prompts

Status: C bridge implemented and synthetic-tested; foreground browser integration pending.
Date: October 26, 2026.

## Context

The owner authorized community-based native Mijia completion. ADR-0085 now has
synthetic-tested protocol and protected account/Host continuation. Community
revision `f78158257e19a21902adaf6bcc8440c3ba172a91` opens a provider page and accepts
an explicitly entered ticket; it does not implement native SMS/email sending.
The existing captcha-only C result has no URL field and must keep its shape.

## Decision

Native API 1.31 adds three trusted functions: `homeverb_begin_account_verification`,
`homeverb_continue_account_verification` and `homeverb_discard_account_verification`.
They use the existing protected Host registry. Ordinary JSON, CLI and MCP gain no
method. Existing 22 C functions, 34 ordinary methods and saved formats stay intact.
The total becomes 25 C exports plus the Android credential initializer.

Begin consumes separately writable login/password buffers of 1..256 bytes.
Continue consumes a 1..2048-byte answer buffer and closed nonsecret metadata that
selects captcha, phone or email. The adapter validates the actual answer and
provider-offered method. No request may choose another account or provider URL.
Valid input buffers are copied and zeroed even on metadata/host/output failure.
Oversized or null inputs are not dereferenced; caller cleanup remains mandatory.

Both functions also require an exclusively owned, nonoverlapping writable output
buffer of exactly 131072 bytes. This capacity covers the existing captcha limit.
Clear it before admission. Connected/error responses leave it zero; pending
responses copy only the bounded captcha bytes or provider-page UTF-8 bytes.
Returned JSON contains only kind, local challenge ID, remaining lifetime,
prompt length, media type or offered method names. Provider URLs, tickets and
cookies never enter JSON. Callers must clear/free the entire output buffer on all
paths, never log/cache/serialize its content, and discard unused challenges.
Malformed output capacity fails before networking, persistence or registration.
The returned JSON string follows the existing `homeverb_string_free` convention.

Use explicit API negotiation before resolving symbols. The inherited three-minute
lifetime, shared four-entry registry, one-use answers and background/cancel/close
invalidation apply. The existing C/App captcha path remains available unchanged.
A decoded valid continuation consumes its pending state before Host admission;
malformed C metadata is rejected before dispatch, as in the existing login bridge.

## Browser boundary and limits

This phase supplies a protected native bridge, not a browser. A later trusted
foreground UI must preserve lifecycle cancellation and avoid plaintext cookie or
credential persistence. Do not exempt external-browser handoff from background
cancellation, auto-confirm account settings, manufacture ticket extraction from
HTML, or claim SMS delivery. Android/iOS packaging and real account/phone acceptance
remain separately recorded. API changes alone do not update an existing APK.

## Verification

Add closed schemas and positive/negative examples, synthetic projection checks,
actual C input/output clearing tests, invalid-capacity/metadata/host/lifecycle
admission and ordinary-method refusal. Retain existing login/native regressions,
update Android export checks and the iOS version assertion, and record actual
package builds separately. SQLite 5, Mijia payload 1 and Meiju payload 3 remain
unchanged. No publication or real-device operation is authorized by this phase.
