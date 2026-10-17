# Source publication audit

Date: October 17, 2026. Audited source: `84db14deb3c0acd3de2401413a97664b2c7658ea`.
The owner requested history-sensitive-data and dependency/asset license review.
This record contains no secret values, personal email addresses or raw scanner
reports. Review artifacts stayed in a private temporary directory outside Git.

The later owner-authorized [history cleanup](publication-status.md) removes the
identified historical image from the candidate branch. The observations below
refer to the original audit scope and private backup; they are retained as evidence.

## Result and remaining decisions

No real credential was identified in the scans and reviewed candidates below.
Known source license declarations have been inventoried, and a missing Flutter
scaffold notice has been added. Publication still requires these concrete decisions:

| Finding | Evidence | Required disposition |
|---|---|---|
| Historical third-party marketing screenshot | Deleted `output/imagegen/homeverb-visual-directions-v2/references/mijia-app-store-home-browser.png`, blob `c96a25524b`; its accompanying historical observations record identifies an official Mi Home App Store promotional image | No redistribution permission was found. Remove it from the history intended for publication, obtain appropriate permission, or publish a reviewed source snapshot without that history. An ordinary deletion commit has not removed the blob. |
| Commit email privacy | Two distinct author/committer addresses across reachable history: one GitHub noreply, one non-reserved address | The owner identified it as their own address and was informed of Git-history visibility. It is not a credential or an automatic publication blocker. Values are omitted here; no authorship rewrite is requested. |
| Meiju public compatibility material | All four constants in `crates/midea/src/material.rs` match the pinned public upstream `cloud.py`; two are encoded as integer-to-hex byte conversions upstream | Included in the owner-requested project-wide source preparation; no duplicate Meiju approval is required. Provenance does not establish provider endorsement. |
| Private vulnerability reporting | No verified reporter intake in SECURITY.md | Configure and verify the reporting route before source publication. |

No branch was rewritten, commit removed, source pushed or repository made public.
A future history rewrite must operate on a separate reviewed copy first, preserve
a private backup and re-scan the resulting public refs. Removing all obsolete
`output/imagegen/` history would also remove the identified marketing screenshot;
retained application/prototype assets are separate paths. The commit-email choice
can be handled in the same reviewed operation if the owner requests it.

## Secret and identifier review

Remote refs were fetched before the audit. `git ls-remote --heads --tags origin`
reported only `main` at `82fd26a7e2904fd367fcfaef89977f6e827bc8e1`; no remote tags
were advertised. Local `main` was the audited source above. There were 180
reachable commits, including four merge commits. No hidden GitHub pull-request
refs, server-side unreachable objects, local reflogs or other clones are covered.

Gitleaks **8.30.1** was downloaded from its official GitHub release into private
temporary storage. Its Darwin arm64 archive matched the published SHA-256:
`b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5`.
No repository or environment configuration override was present. Default rules
were used, inline allow comments were disabled and no project ignore/baseline
suppressed findings. Reports used `--redact=100`.

| Input | Executed coverage | Result |
|---|---|---|
| Git history | `gitleaks git . --log-opts='--all --full-history'`; 176 non-merge commits reported, approximately 8.07 MB of patch text | Exit 1, five findings; all manually classified below |
| Tracked source snapshot | `git archive HEAD` extracted privately, then `gitleaks dir`; approximately 6.48 MB of scanned text | Exit 1, the same five findings |
| Commit metadata | All reachable author/committer fields and full commit messages exported privately, then `gitleaks dir` | Exit 0, no credential finding; separate email privacy observation above |
| Additional identifier patterns | 2,879 unique text blobs checked for email-like strings, mainland mobile-number patterns and colon-separated MAC addresses | 29 candidates: 26 icon filename/Git SSH syntax matches and three fictional phone-test matches; no real identifier identified in these candidates |

The five `generic-api-key` findings were not silently suppressed:

| File and line | Classification and evidence |
|---|---|
| `crates/midea/tests/fixtures/login.json`, lines 9, 10 and 13 | Three fictional cryptographic known-answer outputs. Fixture provenance and the independent-vector codec test establish their use; the test passed during this audit. |
| `crates/midea/tests/fixtures/lan-v3.json`, line 4 | Fictional sequential token bytes, with declared independent Python/OpenSSL vector provenance; all four LAN V3 tests passed during this audit. |
| `docs/adr/0036-local-account-inventory-inspection.md`, line 49 | Ordinary verification prose, not a credential. The scanner matched wording spanning the synthetic-test description. |

The phone candidates belong to `account_identity_test.dart`'s fictional masking
and local-persistence fixtures. They are not evidence of a live account. Public
Meiju compatibility constants were reviewed separately even though the default
scanner did not flag them. Nothing was validated against a provider, device or
external credential-checking service.

A pattern scan cannot establish absence of every secret or private identifier.
Binary content is outside the text scanner. All 522 reachable PNG/JPEG/WebP image
blobs were visually triaged on nine private contact sheets; the marketing image
was inspected at full size and traced to its historical provenance record. No
obvious live-account capture was identified. Contact-sheet triage is not OCR or
a pixel-by-pixel privacy proof. None of those images had an EXIF GPS tag.

## Dependency license evidence

The [machine-readable inventory](audits/dependency-licenses-2026-10-17.json)
records versions, source declarations, license summaries, legal-file paths and
SHA-256 hashes, plus input lockfile hashes. It deliberately omits workstation
paths and credentials. It covers **320 external Rust packages** in the complete
locked resolution and **74 Dart/Flutter SDK packages** in the mobile lockfile.
Development/build and target-specific dependencies are included; this is not a
claim that all of them ship in a particular binary. The presentation package has
no external runtime dependency declarations.

`cargo metadata --locked --format-version 1` completed after downloading missing
locked target dependencies. The initial offline attempt could not resolve an
uncached Windows crate. The Flutter package configuration matched all 74 locked
external entries; SDK license inheritance was checked against Flutter 3.47.7 at
`abaf9c523780a608bd46686fd5e53740a07077f8`.

All 320 Rust packages declare licenses. No mandatory GPL/AGPL-only dependency was
identified in those declarations. Most provide MIT/Apache-2.0 alternatives; other
permissive terms and combined obligations must still be retained:

| Component | Review conclusion |
|---|---|
| `ring` 0.17.14 | Apache-2.0 AND ISC, with BoringSSL, once_cell and fiat legal files; do not discard one side of an AND expression. |
| ICU4X and related Unicode crates; `unicode-ident` | Preserve Unicode-3.0 terms, including the extra Unicode obligation alongside MIT/Apache choices in `unicode-ident`. |
| `webpki-roots` 1.0.9 | CDLA-Permissive-2.0 certificate data; redistribution of the data requires providing the agreement text. |
| `r-efi` 5.3.0 and 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later permits choosing a permissive branch; the presence of LGPL in this OR expression does not make LGPL mandatory. AUTHORS and source provenance remain relevant. |
| `libsqlite3-sys` 0.35.0 | MIT wrapper includes SQLite and optional SQLCipher material in the registry source. Account for the selected features and actual bundled files when assembling binary notices. |
| Dart `characters` 1.4.1 | BSD-3-Clause library plus Unicode-3.0 generated data. Wikipedia samples under CC-BY-SA-3.0 are in test/source data, not imported by the HomeVerb application; preserve their terms if redistributing the full dependency source. |
| Dart `file_selector_android` | Its LICENSE includes both Flutter BSD-3-Clause and aFileChooser Apache-2.0 notices. |
| Flutter engine (`sky_engine`) | Composite bundled notices; not accurately described by one top-level BSD label. Validate the actual target engine/SDK notice bundle for binary releases. |

Six Cargo archives declare a permissive license but omit a standalone license
text: `asn1-rs-impl`, `jni-sys-macros`, `uuid-simd`, `vsimd`,
`winapi-i686-pc-windows-gnu` and `winapi-x86_64-pc-windows-gnu`. They are downloaded
dependencies, not vendored HomeVerb source. Before bundling them in a binary or
vendor archive, recover the appropriate pinned upstream notices rather than
fabricating copyright holders. The inventory flags each case.

This source review is not a completed Android/iOS application-license bundle.
Gradle/Maven transitives, SDK/NDK/runtime notices, Python development-tool
resolutions and the exact target's packaged artifacts were not exhaustively
resolved here. They are separate binary/build-tool review work, not a reason to
claim the current source has no license. No compatibility conclusion authorizes
use of a vendor service or replaces source-specific obligations.

## Assets and notice correction

The retained application images were traced to the fictional Window Light
prototype, project raster generators and the refrigerator/dishwasher generation
record. Launcher icons are generated by `apps/mobile/tool/generate_icons.py` from
the project's home glyph. The Inter font license is already retained. This
provenance review establishes sources, not universal ownership of generated imagery.

The Android/iOS/Linux/web scaffolds derive from Flutter templates. Six retained
files match local template bytes exactly, including three placeholder iOS launch
images, the bridging header and two Linux runner files. Other scaffold files are
modified. The repository previously lacked Flutter's BSD-3-Clause license text;
[Flutter attribution](../third-party/flutter/NOTICE.md) and its complete license
have now been added and linked from the third-party index.

The historical Mi Home marketing screenshot remains a separate rights issue.
It is neither generated HomeVerb artwork nor covered by Flutter's license.
The existing `cefcef0` deletion removes it from the current tree only.

## Verification and follow-up

Executed during this audit:

- Gitleaks history, tracked-snapshot and commit-metadata scans described above.
- Complete locked Cargo metadata and local Dart/SDK license inventory; lockfile
  hashes, full Rust lock coverage, counts, path redaction and Flutter license
  byte integrity were checked.
- `python3 scripts/validate_docs.py`, `python3 scripts/validate_repository.py`,
  `python3 scripts/validate_commits.py --all`,
  `.venv/bin/python scripts/validate_contracts.py` and `git diff --check`: passed.
- A final current-source snapshot including the audit documents was scanned again:
  the same five classified findings, with no additional finding.
- `cargo test -p homeverb-midea --offline --locked --lib codec::tests::matches_independent_fictional_known_answers`: one test passed.
- `cargo test -p homeverb-midea --offline --locked --test lan_v3`: four tests passed.

Re-run source/history scans after new commits or any history rewrite. Review the
final public GitHub metadata and exact commit's remote CI separately. Source and
binary publication remain separate actions. This audit makes no appliance call,
performs no real write, and changes no runtime/API/permission behavior.
