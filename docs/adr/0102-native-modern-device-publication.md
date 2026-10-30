# ADR-0102: Native modern Mijia device publication

Status: Native API 1.37 and Flutter adoption implemented with fictional evidence; packaging/provider acceptance pending.
Date: October 30, 2026.

## Decision

Native API 1.37 adds opt-in device publication to the existing dedicated modern
account exports. Continuation metadata accepts optional Boolean publish_devices,
default false. True uses ADR-0101 and returns kind connected_devices with the
existing AccountConnection report. False/absent retains API 1.36 logical connected
behavior. Saved-session operations add refresh_devices and recover_devices,
returning kind devices with AccountConnection and authentication not_attempted.
Existing refresh, recover and renew keep their logical/authentication-only behavior.
No C symbol or ordinary JSON method is added. The 36 methods, 30 exports (31 with
the Android initializer), protected QR buffer rules and single-use registry remain.

Device reports contain sanitized local account/device IDs and partial or failed
retrieval only, with at most 4096 unique IDs. Failed retrieval returns no IDs; it
never implies completed local storage was rolled back. Logical counts do not become
card counts. Successful login with failed directory/publication still reports
successful authentication. Pending authentication storage remains an error; pending
binding storage reports failed retrieval and requires explicit recovery. Native
projection uses the same Host lifecycle and original QR deadline as ADR-0101.

Ordinary account refresh/recovery must work for modern-only accounts. Select the
inventory protocol from protected bytes under the same registration/account leases
as retrieval: use modern when there is no legacy slot or at least one binding is
currently modern-selected, otherwise legacy. This selection occurs before I/O and
never retries/falls back after failure. A mixed account with a modern-selected
binding refreshes the modern directory; returned roots select modern reads, while
absent historical bindings retain their protocol and IDs. Modern-selected roots
have no modern controls/history yet, so their old control options disappear rather
than retaining unsupported authority. Merely adding a modern authentication slot
to an otherwise legacy account does not switch its ordinary refresh. Explicit typed
legacy and modern refresh methods remain available with their exact semantics.
No preference bit, credential conversion, new payload version or automatic renewal
is introduced. Local listings and startup perform no secret or network work.

Flutter must negotiate API >=1.37 before requesting device publication, validate
the distinct closed result shapes, then reload local accounts/cards and route
ordinary state reads through Core. API 1.36 remains supported with logical-only
reports. UI adoption, packaging and real-provider acceptance are tracked separately
until implemented and tested. No App readiness follows from native contracts alone.
SQLite 5, Mijia payload 3 and Meiju payload 3 are unchanged. Older implementations
still cannot open protected Mijia version 3. No write authority, background service,
complete-inventory time, new origin selection or remote action is admitted.

## Required evidence

Positive/negative schemas for optional publication, new operations and distinct
reports, including wrong types/providers/authority fields, duplicates, limits,
complete claims and failed results with IDs. Actual C metadata/refusal/buffer/
lifecycle tests plus injected Host success reports use the same schemas. Generic
refresh/recovery tests cover modern-only and mixed accounts, stored protocol
selection, failure without fallback, pending recovery, retained identities, and
legacy behavior before adoption. Re-run old logical account/native regressions,
workspace tests, strict Clippy and version-manifest checks. All tests use fictional
accounts and transports; phone/provider/model acceptance remains pending.

## Executed evidence

The workspace regression excluding the known unrelated Meiju/macOS LAN-alias
failures passed 595 tests with one ignored. It includes four new ordinary-account
selection/refusal cases and the previous Host publication cases now validated
against the API 1.37 result schema. Strict workspace/all-target Clippy and the
native library build passed. The rebuilt ABI passed all 36 methods/57 examples
and dedicated bridges, including 76 modern-account refusal/buffer/lifecycle cases.
Contract validation passed 25 schemas, 69 primary positives and 1,196 negatives.
Documentation, repository hygiene and both reference-projection tests passed.
These are fictional software checks. Flutter opt-in, a new APK, signed iOS runtime,
provider authentication and physical-device acceptance were not performed. No
provider account, appliance write, remote CI, push or publication was used.

## Flutter adoption contract

The Flutter client negotiates API 1.37 before adding publication metadata; API
1.36 keeps logical results and API 1.35 still refuses QR before symbol lookup.
Distinct typed result kinds retain either a logical directory or a sanitized
account device report, never both. Validate exact report fields, expected account,
provider/authentication, partial/failed status, error shape, unique bounded UUID
IDs and no complete claim before controller adoption. Owned result maps/lists are
immutable. No new credential or scan material enters ordinary JSON.

After a current foreground result the controller reloads local accounts/cards,
then retains the device report for existing partial/failure presentation. Account,
client, navigation and lifecycle changes invalidate late reports; interrupted work
keeps the local reload requirement. Publication does not imply online devices or
physical support. Card opening uses existing descriptor-driven Core reads and
never obtains modern control authority. Explicit QR refresh/recovery requests
publish devices on 1.37, while renewal remains authentication-only. QR connection
labels work with both native versions; legacy logical results retain their warning.

Required App evidence includes actual fictional C ABI negotiation for 1.35/1.36/
1.37, closed new result parsing, cancellation after ready replies, QR-to-card clicks
and state reads, partial/failed/empty results, local reload failure, bilingual
narrow layouts and unchanged legacy login/account regressions. Flutter implementation now has the evidence below; packaging and provider/phone
acceptance remain separate unfinished gates.

## Flutter implementation evidence

The affected 134-test Flutter account/native/QR/localization/foreground-read run
passed, including 20 new device-publication cases. Fictional C libraries verify
actual API negotiation, new metadata, immutable parsed results, closed response
refusal and cancellation after ready replies. Widget/controller tests follow QR
submission to new cards, click a card and request state through the Core client
interface; separate Host tests above establish native Core behavior. They also
cover empty/failed retrieval, both locales at 320 pixels/doubled text, navigation/
account/background races and local reload failure without repeated login. The
shared home title was made flexible after this test exposed English overflow.
This is software evidence, not a real QR login, phone or appliance test.

Both shared reflow tests passed: all screens at 320 pixels/200 percent text and
all states in landscape/150 percent text. Flutter analysis reported no issues,
and all seven shared presentation tests passed. No full Flutter-suite pass or
new mobile package is claimed; the known unrelated macOS pixel/legacy-click
failures remain outside this affected passing regression.
