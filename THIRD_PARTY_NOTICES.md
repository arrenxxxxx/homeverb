# Third-party notices

This index records known bundled and ported material. It does not select a license
for HomeVerb or replace the referenced license texts. The owner deferred the
project license decision on October 17, 2026; see [LICENSE-DECISION.md](LICENSE-DECISION.md).

## Ported protocol implementations

| Source | Pinned revision | License | Scope and retained notices |
|---|---|---|---|
| [Midea Auto Cloud](https://github.com/sususweet/midea_auto_cloud) | `ce0c4b88224def0f6b53714312c19f71aa9c2a50` | Apache-2.0 | Meiju cloud protocol, security transforms and model mappings; [attribution and modifications](third-party/midea-auto-cloud/NOTICE.md), [license](third-party/midea-auto-cloud/LICENSE.txt) |
| [Midea Local](https://github.com/midea-lan/midea-local) | `ef28169926914d15f40fe521b9cc3d3c2f553c59` | MIT | LAN discovery, framing, credentials and query codecs; [attribution and modifications](third-party/midea-local/NOTICE.md), [license](third-party/midea-local/LICENSE.txt) |

Retain the upstream copyright statements, license texts and applicable notices
when redistributing derived portions, including in binary distribution notices.
The per-source records identify upstream files and HomeVerb modifications. They
remain authoritative when this summary is shorter than the implementation scope.
A future HomeVerb license does not replace these upstream obligations.

The public Meiju app-compatibility material is admitted for the personal MVP only
under [ADR-0022](docs/adr/0022-meiju-community-cloud-admission.md). Its presence in
an open-source implementation does not establish provider approval, permission
to use the service, or authorization to publish this integration. That release
scope still needs a separate owner decision and review.

## Fonts and visual assets

- Bundled Inter Regular and SemiBold fonts are copyright the Inter Project
  Authors and use SIL Open Font License 1.1. Preserve the
  [font license](apps/mobile/assets/fonts/LICENSE.txt) with the font files.
  Do not relicense the fonts under the eventual project license.
- The app's refrigerator and dishwasher illustrations have a local
  [generation and refinement record](apps/mobile/artwork.md). Generated artwork
  is decorative and does not establish exact model support or trademark rights.
- The retained [Window Light prototype](output/prototypes/homeverb-window-light-v1/README.md)
  records its deterministic composition, bundled fictional reference artwork and
  font requirements. It is a test/design reference, not a product screenshot.
  Superseded image-generation experiments have been removed from the current
  tree; their historical commits still require publication review.
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
this work but does not establish permission on its own. The complete dependency
license audit remains pending; this file inventories known vendored/ported sources.
