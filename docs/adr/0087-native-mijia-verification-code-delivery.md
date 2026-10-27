# ADR-0087: Explicit native Mijia verification code delivery

Status: typed protocol, account/Host, native C and App implementation synthetic-tested; Android package checked; real-account acceptance pending.
Date: October 26, 2026.

## Evidence

Anonymous read-only inspection of Xiaomi's public identity page and static modules
identifies code-delivery operations that the pinned community client leaves to a
browser. No account, token, protected notification URL or send-code endpoint was
used for research. Independent protocol facts and asset hashes are recorded in
the Mijia reference integration guide; proprietary JavaScript is not copied into
HomeVerb or executed by it.

## Decision

Extend the protected verification challenge with an explicit selected-method
code request. First GET the selected `/identity/auth/verifyPhone` or `verifyEmail`
with `_flag=4` or `8` and `_json=true`. A code-zero response may supply `contentType`;
the inspected frontend otherwise uses `160010`. Do not return or persist the
provider's masked address. POST `/identity/pass/sms/userQuota` with
`addressType=PH` or `EM`, `contentType` and `_json=true`; require code zero,
result zero and a positive bounded numeric `info` before sending.

One POST sends `retry`, `icode` (empty for the unchallenged request) and `_json=true`
to `/identity/auth/sendPhoneTicket` or `/identity/auth/sendEmailTicket`. Email
requests retain the frontend's `_dc` epoch-millisecond cache-buster. The same
origin/path-scoped memory cookies and original challenge lifetime apply. No
caller-provided phone number, email address, callback or provider URL is admitted.
Code requests require explicit UI action; beginning login or displaying a prompt
must not send a code automatically. Ordinary JSON/CLI/MCP cannot call them.

Track per-method attempt ordinals internally, allow at most three sends across
methods within the original three-minute challenge, and require a monotonic
cooldown of at least 60 seconds after a successful request. Honor a longer valid
provider `data.wt` delay. These are client bounds, not claims about provider quotas.
No timer performs networking. Never automatically resend on errors, cancellation,
timeouts or failed validation; the consumed continuation is unavailable. A
code-zero reply means the provider accepted a request, not that a message arrived.

Provider codes 87001/20031 require human risk verification. Do not generate risk
tokens, solve a challenge automatically, scrape HTML or replay the send request.
Additional account confirmation and multi-stage identity remain separately
unimplemented. This adds the ordinary code-delivery path, not universal account
acceptance. Existing ticket answering and legacy captcha APIs remain compatible.

## Integration and verification

Typed account/Host continuation preserves immutable account snapshots, rechecks
before each operation and returns a new one-use pending handle. Expose selected
method, successful request count and remaining cooldown as nonsecret metadata;
provider URLs/addresses remain protected. Add a dedicated native function before
App adoption; do not overload a ticket buffer with an instruction to send SMS.
No real SMS/email or account access is permitted by development tests.

Use fictional protocol peers and local TLS to verify GET/POST separation, exact
forms, explicit methods, quota failure, provider risk refusal, expired cookies,
subject/lifetime preservation, monotonic cooldown, capacity, cancellation and
no retry. Keep contract/version, lifecycle and compatibility evidence synchronized
when later exposing the typed implementation through C and Flutter.

## Native code-request bridge (API 1.32)

Add `homeverb_request_account_verification_code(handle, metadata, prompt,
131072)` as one dedicated trusted C operation. Closed metadata contains only
schema/operation/deadline, the session-owned challenge ID and `method` (`phone`
or `email`). No ticket buffer, provider address, cookie or URL is accepted.
The existing protected prompt buffer rules apply, including clearing on admission
failure. The result retains the verification-prompt metadata and additionally
reports `code_requests` (1..3) and `cooldown_ms` (0..86400000). A new local handle
replaces the consumed old handle. These fields describe acknowledgement and local
bounds, not message arrival. Existing begin/answer result shapes stay unchanged.

Require API 1.32 before resolving the new symbol. The total becomes 26 C functions
plus Android's initializer; all 34 ordinary methods and saved formats remain
unchanged. Synthetic ABI tests must cover buffer clearing, metadata/method
refusal, closed/background hosts and ordinary JSON isolation. App adoption and
mobile packages remain separately verified phases.
