# ADR-0096: Modern Mijia Host lifecycle

Status: implemented in the typed Host; dedicated C/App integration pending.
Date: October 30, 2026.

## Decision

Add typed foreground Session entry points for the ADR-0095 modern account flow.
They reuse the existing protected ConnectRequest and continuation metadata but
accept no login/password buffers. The provider must explicitly select the
registered Mijia flow. No ordinary Request, CLI or MCP method is added.

A successful begin returns a nonserializable, redacted QR presentation object
containing an opaque local challenge ID, protected scan bytes and remaining
milliseconds. Only trusted local UI may render those bytes. The pending adapter
challenge remains in its owning Session, sharing the existing four-entry limit
with captcha and verification challenges. Prune expired entries during admission;
never retain an account lease while a user scans. The begin timeout also bounds
the entire challenge lifetime (at most 180 seconds); callers that need the full
scan window must explicitly request it. Continuation cannot extend that lifetime.

Continuation consumes its handle before validating remaining metadata or registering
work. Invalid input, timeout, failure and cancellation cannot replay it. A foreign
Session cannot consume another Session's handle. Discard is local and idempotent.
Background, cancel and close clear all pending sign-in families and cancel relevant active
registered work. A lifecycle generation check rejects late presentation/results
before returning to a caller. No timer, idle renewal, retry or background task is
introduced. Credentials successfully saved before cancellation remain saved;
uncertain writes retain the adapter's pending/recovery semantics.

Explicit typed modern-directory refresh/recovery and renewal use the same operation
registration, foreground requirement and lifecycle completion check. Refresh and
recovery return the adapter's opaque logical entries and conservative completeness;
renewal returns only the public account summary and does not enumerate. These
entries are not public Device cards, contain no general serialization implementation,
and do not add routes, authority or protocol fallback. Core reconciliation remains
a separate step. No provider identifier, scan payload or credential enters ordinary
JSON, logging or SQLite.

Mijia-only and combined native Session constructors register the reviewed HTTPS
modern transports without networking at startup. Core-only CLI/MCP constructors
retain their current configuration. Default Mock and Meiju-only sessions refuse
modern requests. Existing read/write grants, ordinary dispatch, native API 1.35,
C exports, SQLite 5 and payload versions are unchanged by this Host-only phase.

Dedicated C buffers, API negotiation, QR rendering and App navigation/lifecycle
integration remain pending. No authenticated provider or physical acceptance is
implied by fictional Host tests.

## Required evidence

Fictional QR begin/finish and saved-session refresh/renewal; preserved legacy cards;
shared pending limit across all sign-in families; expiry/pruning; foreign owner,
duplicate handle and invalid-metadata consumption; cancellation/background/close
while begin, poll and directory/renewal are in flight; dropped futures release
registration and leases; pending-secret recovery without reauthentication; no
ordinary JSON methods or protected values in public account output; empty native
startup remains silent. Run affected Host and Mijia suites and strict Clippy,
then rebuild/check the native ABI and validate repository contracts/documentation.

## Executed evidence

Fourteen fictional modern Host tests passed. They cover the shared registry,
provider/ordinary-dispatch refusal, explicit renewal/recovery, legacy-card
preservation and pending/in-flight/ready-result lifecycle races. The broader
workspace, ABI and contract results are recorded in the
[Mijia guide](../development/11-mijia.md). Dedicated C/App and authenticated
provider/physical acceptance remain pending.
