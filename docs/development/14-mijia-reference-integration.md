# 14. Mijia reference integration and remaining coverage

The October 25 worktree uses API 1.27 and the owner-supplied October 22 Xiaomi Home
11.7.709 analysis. The full official-App feature request remains **incomplete**.
The source archive explicitly does not supply all authentication, protocol or
runtime control contracts. An implemented decoder is not physical acceptance.

## Implemented software

- Eight-part MIoT URNs retain their exact extension; malformed/foreign URNs refuse.
- Seventeen exact model/specification catalogs project 264 reviewed scalar read
  properties. Nineteen model records are tracked; scales and watch remain named
  placeholders without a reviewed specification. A fitting specification without
  readable fields also stays a placeholder.
- Fan, air-purifier, light, outlet/switch, gateway, lock/video-doorbell, camera,
  temperature/humidity, water-leak, remote battery and hood scalar settings retain
  separate services and exact wire types. A runtime property error remains absent;
  offline mapping cannot repair a provider's failed response.
- Only equivalent fan/purifier/switch/light on-state uses `power.state`. Indicator,
  camera and lock-adjacent switches retain vendor scope. No online flag becomes
  power, lock state or physical connectivity.
- Additional fields, including fan vertical swing/angle/anion/timing/quiet, remain
  read-only. Existing exact-granted fan setters retain their protected checks.
  Source write access alone cannot create any new action.
- Public pinned specifications can supply exact mappings during explicit account
  retrieval without public-spec networking. Conflicting reported URNs do not use
  this fallback. Old protected derivations remain valid and explicit refresh
  upgrades them while preserving local account/device/connection references.
- Shared English/Chinese labels cover added scalar fields and component selectors.
  The generic App panel keeps raw codes, missing values, units and cloud staleness;
  the initial component prioritizes the actual appliance service over auxiliaries.

The [coverage ledger](mijia-interface-coverage.json) enumerates every source
property/action/event, including exclusions. The [projection script](../../scripts/extract_mijia_reference_facts.py)
checks the archive digest and copies only public specification facts. It never
executes an archived script or copies private observations/identifiers. Verify it:

```sh
python3 scripts/extract_mijia_reference_facts.py --check
cargo +1.96.0 test -p homeverb-mijia --offline --locked
```

## Coverage boundaries

| Source item | Current disposition |
|---|---|
| 264 scalar readable properties | Exact mapping implemented; synthetic Core reads; provider/physical acceptance remains per property |
| 97 identity/pairing/media properties | Excluded from ordinary observations; no identity leakage or streaming inference |
| 78 non-readable properties | No read operation; write/event-only definitions are not state |
| 28 opaque readable values | Separate typed/business/privacy contracts required |
| 3 network identity values | Excluded from state and exports |
| 40 actions | No new action admission; runtime/firmware/composite constraints and risk review missing |
| 78 events | No event-history/subscription body, paging or lifecycle contract; not implemented |

The archived successful reads used **SID `mijia`** and
`https://core.api.mijia.tech/app`, with a continuous RC4 stream across encrypted
form fields. The existing native login uses **SID `xiaomiio`**, its reviewed
`api.io.mi.com` routes and legacy field encryption. These are separate session
contracts. Do not move a saved legacy token to the modern origin, silently change
its signing mode, or call modern success evidence proof of independent login.
The archive excludes the existing App session and reports standalone login,
renewal and expiry recovery as unverified.

The modern device-page endpoint requests logical split children. The analysis
records two split switch objects and a cloud/cache mismatch (21 shared IDs out
of 22 in each source). These do not prove 23 physical appliances. The current
legacy enumerator retains its bounded account/home union and partial status;
complete inventory, logical-parent reconciliation and missing-device removal are
not claimed. No private identifiers enter this ledger.

Blocked pending additional evidence: modern native authentication/renewal;
specialized paths for failed cloud properties and BLE devices; history/statistics,
event pagination and foreground subscriptions; health/media/streams; plugin-specific
composite conversion and additional write constraints. Door unlocking stays denied.
No background service, arbitrary RPC, remote MCP or scheduled automation is added.

## Verification

Executed results are appended after the final checks. API 1.27, SQLite 5 and the
protected payload envelopes are unchanged by read mappings. New reference-derived
payload content requires this updated mapping implementation; keep matching native
and App builds. No real account/device operation or remote publication is part of
this phase.

Final source checks on October 25: 321 Mijia/Host Rust tests and compile-fail tests
passed, including all 264 fictional scalar readings through Core. The standalone
Mijia run passed 62 unit and three compile-fail tests. Strict Mijia/Host Clippy,
workspace build and formatting passed. Flutter analysis and the final six-file
account/Mijia/reference/localization/recovery subset passed all 32 cases. Shared
presentation tests passed seven cases; Python tools passed 72 tests. Documentation,
21 schemas with 56 positive/365 negative fixtures, source hygiene and digest-based
reference regeneration passed.

An initial complete Flutter run passed 193 and failed nine. One newly exposed Mock
identity-recovery capability assertion was corrected while preserving Mijia-only
refusal; its actual FFI regression passed in the 32-case rerun. Eight existing
macOS failures remain: button ink, field ink, two control-glyph/pixel tests and
four legacy app/outcome click cases. A final full run is recorded with delivery.
No full-suite pass or physical/device compatibility is claimed.

The Android native AAR cross-build passed arm64-v8a and x86_64, 20 exports per ABI
and 16 KiB ELF alignment using the already-installed NDK 28.2.13676358 and JDK 21.
APK assembly and archive verification are a separate delivery phase; no package
installation or account/device request has occurred.


## Combined Mijia reference APK -- October 25, 2026

Clean application source `87ff5b12a246f46525d7cf50ab154a3bfe154a87` built the
combined `homeverb.platforms` release APK with the FVM-pinned Flutter 3.47.7.
The build used the existing SDK/NDK and persistent development signing location,
with temporary Flutter SDK/JDK configuration. No license or SDK was installed.

Artifact: `apps/mobile/build/delivery/mijia/homeverb-mijia-0.2.0-87ff5b1.apk`.
Size: 74,466,398 bytes. SHA-256:
`e271a2cabd20d5eae87232e0355a084987840608e65d95c30e246fb70ae112e4`.
The adjacent `verification.json` records clean source, AAR/native digests, APK v2
development signature, exactly arm64-v8a/x86_64, API 28 minimum, disabled backup,
no integration-test plugin, 20 Core exports per ABI and 16 KiB ZIP/ELF alignment.
Both native payloads exactly match the current AAR after normal NDK stripping.

Final full Flutter regression: **194 passed, eight failed**. All eight match the
previously recorded macOS glyph/pixel and legacy app/outcome-click cases. The
introduced Mock recovery assertion is fixed and passes. Final workspace strict
Clippy and Flutter analysis passed. This is not a full Flutter test-suite pass.

The recovered worktree moved from `/private/tmp/homeverb-mijia-current` to the
persistent `/Users/zhao/develop/codes/any/homeverb-mijia` directory after packaging.
The old damaged temporary directory's residual files remain untouched; only its
stale Git registration was pruned. Offline Flutter dependency resolution and
analysis passed again after relocation. Mainline remains unchanged. No merge
commit, remote push, installation, account access, real appliance operation,
physical-phone test, telemetry upload or publication was performed.

A full `cargo +1.96.0 test --workspace --offline --locked` attempt on October 25
stopped in the existing Meiju LAN suite: 56 tests passed and 14 failed while
binding synthetic interface addresses (`AddrNotAvailable`, macOS error 49).
The required local interface aliases are absent; host network configuration was
not changed. This run is not a passing workspace test. The separately executed
Mijia/Host suites above remain the scoped evidence.

## Distinct component selectors -- October 25, 2026

Repeated family labels now use their exact localized component labels, so the
reference lock's `battery-4` and `battery-10` remain independently selectable.
Singleton families retain their short label. Unknown component IDs keep a stable
ID fallback. No provider/model branching or change to component identity is added.
The selector also uses the existing control-text layout, fixing the Flutter
`ChoiceChip` dry-layout assertion caused by ordinary `Copy` text.

Executed `fvm flutter analyze` passed. All 18 tests in `test/mijia_test.dart`,
`test/mijia_reference_test.dart` and `test/mijia_controls_test.dart` passed.
The two new regression cases first reproduced the selector failures and then
passed in English/Chinese at 320 logical pixels and double text scale. They
select both batteries, verify independent snapshots and confirm no writes occur.
The previously recorded full-suite failures remain unresolved; this scoped run
is not a new full-suite pass. Native API 1.27, data schemas and grants are unchanged.
The APK from `87ff5b1` predates this UI fix; its earlier archive verification is
valid for that source revision only.

## Component selector APK -- October 25, 2026

Clean source `28cf192c8c7879ba357ce22d7dc83451fb3e742c` built the combined-platform release
APK with the component selector correction:
`apps/mobile/build/delivery/mijia/28cf192/homeverb-mijia-0.2.0-28cf192.apk`.
Size: 74,466,398 bytes. SHA-256:
`07022839234c89cfabaec002c13007a5934388bb1326eacf32865d7e155b88ac`. The adjacent `verification.json` records both ABIs,
20 exports each, current AAR matching, development signing and 16 KiB alignment.
The release archive has no integration-test plugin and application backup is off.

Executed `fvm flutter build apk --release --target-platform
android-arm64,android-x64 --dart-define=HOMEVERB_ADAPTER=homeverb.platforms` and
`scripts/verify_flutter_apk.py` passed. An initial `--no-pub` attempt retained
an integration-test registration generated by the test workflow and failed Java
compilation; normal release dependency/plugin regeneration resolved it. Installed
SDK/NDK/JDK and existing development signing material were reused. No installation,
provider request, real-device operation, upload, push or publication occurred.
This APK supersedes the `87ff5b1` package for local evaluation; it does not close
the protocol gaps or the previously recorded full-suite/physical acceptance limits.
