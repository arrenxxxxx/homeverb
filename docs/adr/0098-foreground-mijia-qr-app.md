# ADR-0098: Foreground Mijia QR sign-in in the App

Status: Implemented with fictional Flutter and C-ABI tests; provider/phone acceptance pending.
Date: October 30, 2026.

## Decision

Consume the API 1.36 protected modern-account bridge directly from Flutter FFI.
A separate typed client interface owns QR challenge objects, protected byte arrays,
remaining lifetime and explicit modern-account results. Negotiate API 1.36 and a
registered Mijia provider before symbol lookup. Older libraries keep the existing
password/captcha/verification flow. No CLI shelling or ordinary JSON method is used.

Worker isolates receive addresses and nonsecret metadata only. Allocate and clear
exactly 8192 native scan bytes on every path, copy only the validated response
length, and retain scan bytes only in the foreground challenge object. Reject
unknown result fields, invalid metadata, malformed account/directory summaries,
wrong providers and changed lifecycle generations. A continuation consumes only a
challenge owned by the same client, clears its scan data and performs one explicit
bounded call. No retries, automatic polling or QR regeneration occur.

The App adds an explicit QR choice beside the existing Mijia password flow after
its credential-storage readiness check. Render locally with pinned BSD-3-Clause
`qr` 3.0.2 from byte input, not a URL/string widget, remote image service or browser.
Retain only a disposable module bitmap for painting; clear source bytes after
encoding and clear the bitmap on dismissal, expiry, navigation or background.
The encoder may allocate transient managed intermediates; this is bounded lifetime
and explicit clearing of owned buffers, not a guarantee of whole-heap erasure.
Use a white quiet zone, dark modules and bounded layout with accessible instruction
text, without exposing the URL in semantics or clipboard/export actions.

Because HomeVerb must remain foreground, instructions ask the user to scan with
Xiaomi Home on another device, approve there and explicitly continue. Do not invite
switching away and imply that background completion will survive. Returning from
background requires a new explicit attempt. Waiting/cancellation stays tied to
existing controller/navigation epochs and native operation IDs. A local expiry
timer performs no networking; leaving the foreground cancels networking and drops
protected presentation. Storage completed before cancellation is not rolled back.

Modern result state is separate from legacy device retrieval. Show saved account
and conservative logical-directory counts without claiming imported physical
cards or complete membership. Explicit modern refresh/recover/renewal actions use
the corresponding native operation; never renew automatically after failed reads
or silently substitute the legacy route. Native logical-parent/Core reconciliation
remains separately tracked. Source examples and fictional App tests do not prove
that newly issued QR sessions work with the provider or physical appliances.

New user-facing text uses the canonical English/Chinese catalogs and generated
localization. Keep third-party license attribution with the pinned dependency.
No native API/schema, stored authority, device write or background policy changes.
Android packaging and owner-run real-account/phone acceptance remain separate.
