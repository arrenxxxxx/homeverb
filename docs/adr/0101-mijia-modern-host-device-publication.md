# ADR-0101: Foreground modern Mijia device publication

Status: Typed adapter/Host publication implemented and synthetic-tested; native/App integration pending.
Date: October 30, 2026.

## Decision

Add explicit typed adapter and Host continuations that publish ADR-0100 device
bindings after modern QR authentication. The existing logical continuation and
directory methods retain their behavior. The new adapter continuation returns
AccountDiscovery; the Host returns the existing sanitized AccountConnection with
authentication succeeded and retrieval partial or failed. Explicit typed modern
device refresh/recovery publish through the same Core path with authentication
not_attempted. Directory entries are never counted as physical cards.

Reuse the single directory response obtained during QR continuation. Registration
and all account leases cover subject checks, protected authentication persistence,
directory reconciliation and binding persistence. Do not release them between
login and device adoption, issue another directory read, or extend the original
QR deadline. Both the original and continuation cancellation tokens cover adapter
work; Host publication is clamped to the original QR expiry and checked against
the foreground lifecycle generation before and after publication.

Only persisted, reconciled descriptors reach Core. Unknown models remain named
unsupported cards, logical children do not receive separate cards, unresolved
relationships remain partial and identity conflicts refuse publication. Missing
historical cards stay retained. Account storage may succeed even when directory
retrieval or publication fails; report those outcomes independently. Failed or
uncertain binding storage publishes no new descriptors. Recovery validates exact
saved bytes without authentication or rewriting unchanged credentials. A late
cancellation does not roll back completed protected or Core storage; callers must
reload local accounts/cards after interruption.

Published devices use protected modern credentials and the selected modern codec
for ordinary Core reads. No fallback, automatic login/renewal, control/history
authority, background work or complete-inventory timestamp is introduced. Existing
legacy account methods remain explicit legacy operations in this typed phase.

Native API 1.36 and its logical-only C/App result schemas remain unchanged. A
subsequent versioned native/App integration must opt into device publication and
preserve old callers. It must also define generic account refresh selection for
modern-only and mixed accounts before exposing the new cards in Flutter. SQLite 5,
Meiju payload 3 and Mijia payload 3 are unchanged. Older implementations still
cannot open Mijia payload 3. No APK or provider/device acceptance is implied.

## Required evidence

Fictional QR-to-card-to-state reads through the actual Host/Core; exactly one
directory read during login; logical-child and unknown-model behavior; partial
and failed retrieval reports; stable IDs/organization across refresh/reconnect;
uncertain authentication/binding storage and explicit recovery; foreign/consumed
handles, original deadline, cancellation/background/close, dropped work, lease
release and late-generation refusal; secret exclusion from reports and SQLite;
existing logical C/Host behavior and legacy account regressions. New Rust reports
must satisfy the existing account connection schema, without changing C exports
or claiming App availability. Real provider/phone/model acceptance stays pending.

## Executed evidence

The Rust workspace regression excluding the documented unrelated Meiju/macOS
LAN-alias failures passed 591 tests with one ignored, including all 12 new Host
publication cases. Strict workspace/all-target Clippy and the native library build
passed. The rebuilt ABI passed 36 ordinary methods/57 examples and all dedicated
export checks, including 62 modern-account refusal/buffer/lifecycle cases. Contracts
passed 25 schemas, 69 primary positives and 1,092 negatives; documentation, repository
hygiene, the generated coverage check and both reference-projection tests passed.
These are fictional software checks, not provider, phone or physical-device
acceptance. No Flutter build, APK, provider account, appliance write, remote CI,
push or publication was performed.
