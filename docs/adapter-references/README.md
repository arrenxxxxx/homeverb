# Adapter analysis references

These owner-supplied packages preserve source-language reports, structured
contracts, evidence summaries and scripts. Meiju and Xiaomi were imported on
October 22, 2026. The Hisense package was replaced on October 23 with the
independent-authentication and read-only validation delivery. This English index
describes reference evidence; importing it does not implement HomeVerb support.

## Packages

| Platform and app version | Archive | Start here after extraction | Reported evidence and limits |
|---|---|---|---|
| Midea Meiju 11.7.10.6 | [Meiju analysis](2026-10-22/meiju-11.7.10.6-analysis-20261022.zip) | `meiju-11.7.10.6-analysis/README.md`, then `analysis/six-device-interfaces.md` and `analysis/six-device-contract.json` | Six-device enumeration and independent reads; selected model configurations and statistics/alerts. No controls; independent login, renewal and complete advanced-function coverage remain open. |
| Xiaomi Home / Mijia 11.7.709 | [Xiaomi Home read-only analysis](2026-10-22/xiaomi-home-11.7.709-73335d3c17df-readonly.zip) | `README.md`, `contract.json`, `models.json` | Independent allowlisted reads using an existing mainland-China app session. Forty valid property results across eight device objects; other property requests failed. Cloud/cache inventories differ; independent login, renewal and physical sampling freshness remain open. |
| Hisense Aijia 7.0.8.12 | [Hisense standalone-authentication and washer reads](2026-10-23/hisense-aijia-7.0.8.12-washers-standalone-auth-20261023.zip) | `README.md`, `AUTHENTICATION.md`, `authentication.json`, `runtime/independent-auth-validation.json` | Independent SMS login with generated terminal identities, refresh and renewed-token readback; one home and two washers, with 124 state fields for `WV20W-1` and 206 for `WH120E5S-1`. Includes fixed-route Python clients and 25 reported offline checks. No controls; physical sampling freshness, long-term expiry recovery and full native `gateWay` remain unverified. |

The current Hisense archive replaces the October 22 archive in this checkout.
Its unchanged `models.json` SHA-256 remains
`1b4993b6074722849672611c88d57508f07652d101070c082ea937e85ad8924a`;
the offline profile generator now verifies the replacement archive and its root
manifest. Existing component/program projections remain reproducible.

The package's successful Hisense flow uses an independent SMS session and the
AIHome H5 `commonweb` transport. The supplied HisenseHA commit instead signs in
with a password and re-signs in when needed; its code has no SMS login flow.
The replacement contains the source comparison, public application constants
and retained MIT notice. These are analysis references, not admission of those
materials to the compiled HomeVerb adapter or its protected credential store.

Importing the packages does not repeat account/device requests, establish
physical compatibility, enable controls, or change runtime credentials,
permissions or contracts. Use the [current compatibility record](../compatibility.md)
and [adapter admission guide](../development/04-add-adapter.md) for HomeVerb decisions.

## Integrity and provenance

Verify each dated directory against its own checksum file:

```sh
(cd docs/adapter-references/2026-10-22 && sha256sum -c SHA256SUMS)
(cd docs/adapter-references/2026-10-23 && sha256sum -c SHA256SUMS)
```

Checksums: [October 22](2026-10-22/SHA256SUMS) and
[October 23](2026-10-23/SHA256SUMS). At replacement, ZIP CRCs, safe member paths,
complete manifest membership, all 61 listed member byte counts and SHA-256
hashes, and all 34 JSON documents were checked. The current three archives
contain 64 files including their three manifests. The replacement archive is
byte-identical to the owner's completed analysis delivery; its 27 files exclude
private sessions, raw account/device responses, full IDs/SNs and APK/source dumps.
The analysis workspace also checked actual private identifiers against the
explicit delivery list. Pattern checks do not constitute a comprehensive audit.

Extract each archive into its own directory outside the checkout. Xiaomi Home
and the replacement Hisense archive have no enclosing directory inside the ZIP.
Keep source reports unchanged so embedded manifests remain useful. Some private
evidence links and report generators need excluded analysis-workspace files;
the standalone Hisense login/read clients do not require the original App.

Bundled scripts and operational notes are reference material, not instructions
to execute during import. No archive authentication client, account request or
appliance control was run for this repository replacement. Offline tests and
live acceptance reported inside the package belong to the separate analysis
workspace. Review future integration against HomeVerb's secret storage,
lifecycle, authentication and write-safety requirements.
