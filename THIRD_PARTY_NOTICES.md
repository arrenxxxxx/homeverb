# Third-party notices

HomeVerb uses [Apache-2.0](LICENSE), selected by the owner on October 17, 2026;
see [LICENSE-DECISION.md](LICENSE-DECISION.md). This index records known bundled
and ported material with its own terms. It does not replace those license texts
or relicense third-party material under the project license.

## Ported protocol implementations

| Source | Pinned revision | License | Scope and retained notices |
|---|---|---|---|
| [Midea Auto Cloud](https://github.com/sususweet/midea_auto_cloud) | `ce0c4b88224def0f6b53714312c19f71aa9c2a50` | Apache-2.0 | Meiju cloud protocol, security transforms and model mappings; [attribution and modifications](third-party/midea-auto-cloud/NOTICE.md), [license](third-party/midea-auto-cloud/LICENSE.txt) |
| [Midea Local](https://github.com/midea-lan/midea-local) | `ef28169926914d15f40fe521b9cc3d3c2f553c59` | MIT | LAN discovery, framing, credentials and query codecs; [attribution and modifications](third-party/midea-local/NOTICE.md), [license](third-party/midea-local/LICENSE.txt) |

Retain the upstream copyright statements, license texts and applicable notices
when redistributing derived portions, including in binary distribution notices.
The per-source records identify upstream files and HomeVerb modifications. They
remain authoritative when this summary is shorter than the implementation scope.
The HomeVerb license does not replace these upstream obligations.

The existing Meiju integration and pinned public app-compatibility material are
included in the owner's project-wide open-source preparation; see the clarification
in [ADR-0022](docs/adr/0022-meiju-community-cloud-admission.md). No duplicate
integration-specific owner approval is required. The integration is unofficial;
upstream source licensing does not establish provider endorsement or guaranteed
service access. Preserve the applicable source licenses and notices.

## Flutter application scaffolding

The mobile platform scaffolds derive from Flutter templates under BSD-3-Clause.
Retain the [Flutter notice](third-party/flutter/NOTICE.md) and
[license](third-party/flutter/LICENSE.txt), including for modified template code.
Flutter runtime/engine and transitive package notices remain separate obligations.

## Fonts and visual assets

- Bundled Inter Regular and SemiBold fonts are copyright the Inter Project
  Authors and use SIL Open Font License 1.1. Preserve the
  [font license](apps/mobile/assets/fonts/LICENSE.txt) with the font files.
  Do not relicense the fonts under the project license.
- The app's refrigerator and dishwasher illustrations have a local
  [generation and refinement record](apps/mobile/artwork.md). Generated artwork
  is decorative and does not establish exact model support or trademark rights.
- The retained [Window Light prototype](output/prototypes/homeverb-window-light-v1/README.md)
  records its deterministic composition, bundled fictional reference artwork and
  font requirements. It is a test/design reference, not a product screenshot.
  Superseded image-generation experiments have been removed from the current
  tree and public branch history after review; see the
  [source publication record](docs/publication-status.md).
- Review the provenance and distribution rights of every remaining shipped
  image, icon and externally referenced asset before a public release. This
  index is not a completed asset-rights audit or a blanket license grant.

## Dependency review before distribution

`Cargo.lock` and `apps/mobile/pubspec.lock` pin dependency resolutions; they do
not supply all attribution obligations. Review Rust, Flutter/Dart, Android and
iOS dependencies for the actual shipped targets, including transitive and
statically linked native components. Include required notices in source archives
and application/library packages, not only this repository.

For each release, record the dependency name, resolved version, source, license
expression, required notices, target and local modifications. Check compatibility
with the chosen project license and distinguish build-only dependencies from
redistributed components. A generated SBOM or license inventory helps document
this work but does not establish permission on its own. The [source audit](docs/open-source-audit.md) inventories 320 Rust and 74 Dart/SDK
packages and records combined licenses, missing upstream notice files and the
historical marketing screenshot. Actual target binary notice packaging remains
separate work; this index does not claim a completed binary-license audit.
