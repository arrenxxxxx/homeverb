# Adapter analysis references

These owner-supplied analysis packages were imported on October 22, 2026 for
future adapter work. The original ZIP bytes, source-language reports, structured
contracts, evidence summaries and scripts are preserved together. This English
index describes the archive contents; it does not establish HomeVerb support.

## Packages

| Platform and app version | Archive | Start here after extraction | Reported evidence and limits |
|---|---|---|---|
| Midea Meiju 11.7.10.6 | [Meiju analysis](2026-10-22/meiju-11.7.10.6-analysis-20261022.zip) | `meiju-11.7.10.6-analysis/README.md`, then `analysis/six-device-interfaces.md` and `analysis/six-device-contract.json` | Six-device enumeration and independent reads; selected model configurations and statistics/alerts. No controls; independent login, renewal and complete advanced-function coverage remain open. |
| Xiaomi Home / Mijia 11.7.709 | [Xiaomi Home read-only analysis](2026-10-22/xiaomi-home-11.7.709-73335d3c17df-readonly.zip) | `README.md`, `contract.json`, `models.json` | Independent allowlisted reads using an existing mainland-China app session. Forty valid property results across eight device objects; other property requests failed. Cloud/cache inventories differ; independent login, renewal and physical sampling freshness remain open. |
| Hisense Aijia 7.0.8.12 | [Hisense washer analysis](2026-10-22/hisense-aijia-7.0.8.12-washers-readonly-20261022.zip) | `hisense-aijia-7.0.8.12-washers-readonly/README.md`, then `contract.json` and `models.json` | Official-app panel/cache observations for `WV20W-1` and `WH120E5S-1`, with model-specific program definitions. Independent gateway signing/login renewal and per-program parameter coverage remain open; no standalone cloud adapter is delivered. |

The evidence in this table is reported by the supplied packages. Importing them
does not repeat account/device requests, verify physical support, enable controls,
or change runtime contracts, credentials, permissions or compatibility status.
Use the [current compatibility record](../compatibility.md) and
[adapter admission guide](../development/04-add-adapter.md) for HomeVerb decisions.

## Integrity and provenance

The dated directory contains the three original archives and their
[SHA-256 checksums](2026-10-22/SHA256SUMS). Verify them from that directory:

```sh
sha256sum -c SHA256SUMS
```

At import, ZIP CRCs, member paths, complete manifest membership, all 48 listed
member byte counts and SHA-256 hashes, and all 29 JSON documents were checked.
The packages contain 51 files including their three manifests:
`PACKAGE-MANIFEST.json` for Meiju and `delivery_manifest.json` for the other two.
Common credential-assignment, bearer-token, private-key and MAC-address patterns
were also checked without printing values; this is not a comprehensive security
audit. The source manifests describe their redaction and exclusion boundaries.

Extract each archive into its own directory outside the checkout. Xiaomi Home
has no enclosing directory inside its ZIP. Keep source reports unchanged so the
embedded manifests remain useful. Some source/private-evidence links and report
generators require excluded material in the original analysis workspace; the
archives are not fully self-contained reproductions of those investigations.

Bundled scripts and operational notes are reference material, not instructions
to execute during import. No bundled script, account authentication, appliance
request or control was run for this documentation change. Review any future use
against HomeVerb's authentication, secret-storage, lifecycle and write-safety
requirements. Importing these reports grants no additional execution authority.
