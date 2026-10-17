# Open-source release preparation

Status: preparation in progress, October 17, 2026. The owner requested open-source
preparation and removal of unused design studies, and explicitly deferred the
main license decision. This record does not authorize changing repository visibility,
pushing, publishing packages or performing appliance writes.

## Verified preparation

- GitHub metadata for `arrenxxxxx/homeverb` reported `PRIVATE`, with default branch
  `main`, on October 17, 2026. No remote setting was changed during preparation.
- The repository has contribution guidance, issue/PR templates, a changelog,
  compatibility records and documentation/Mock/native CI definitions. Workflow
  files are not evidence that the release commit passed remote CI.
- Superseded `output/imagegen/` studies were removed: 314 tracked files, approximately
  144 MiB in the working tree, including images and supporting study files.
  Application assets and the self-contained 82-state prototype remain in use.
  Their integrity validator checks 138 images and 555 local links. Deletion in a
  new commit does not remove images from Git history or reduce a full clone by
  the same amount.
- [Third-party notices](../THIRD_PARTY_NOTICES.md), [support](../SUPPORT.md),
  [security policy](../SECURITY.md) and the [publishing procedure](publishing.md)
  separate current evidence from release prerequisites.

## Required before public source publication

| Gate | Current status | Completion evidence |
|---|---|---|
| Project license and rights holders | Blocked: owner deferred selection | Owner-approved license, accurate rights-holder information, root license text and consistent README/package/contribution metadata |
| Meiju publication scope | Blocked: ADR-0022 admits personal use only | Separate owner decision recording exact publicly distributable protocol material and review of provider/third-party restrictions; amend the ADR scope explicitly before publication |
| Third-party and asset obligations | Partial: known port/font notices retained | Review resolved dependencies for intended targets, remaining asset provenance, license compatibility and source/binary notice packaging |
| Private vulnerability reporting | Blocked: no verified public reporter intake | Enable GitHub private reporting or a verified private alternative, confirm access and update SECURITY.md; the October 17 API read returned 404 |
| History and sensitive data | Pending full scan and manual review | Scan every ref intended for publication plus commit metadata; manually inspect images, fixtures and unusual findings; record scanner version, scope, redacted findings and resolution |
| Repository settings and CI | Pending release-commit verification | Owner-approved branch rules, required checks and available security scanning; exact commit's actual remote check results |
| Public-facing status | Preparation complete, final release review pending | README/support/compatibility/changelog describe the chosen release scope without claiming untested models or signed phone acceptance |

Existing basic hygiene checks are not comprehensive secret or license scanners.
`gitleaks` and `trufflehog` were not installed on the preparation workstation; no
full-history secret-scan success is claimed. Treat known public protocol constants
separately from private credentials: a reviewed public constant may be an expected
scanner finding, but it still requires the publication decision above. Do not
blanket-ignore a source directory or print secret values into a report.

## Proposed publication scope for the owner to review

Publish source and development instructions first, with Mock as the reproducible
starting point and explicit experimental status for exact Meiju mappings. Do not
claim official vendor endorsement, general brand compatibility, signed mobile
release support or complete official-App parity.

The current tree includes Meiju code and public app-compatibility constants. A
Mock default, runtime switch or unused code path does not exclude that material
from a public source archive or its history. The owner must decide whether to
publish the reviewed integration or prepare a separately reviewed source scope.
The latter requires reviewing history as well as build dependencies; merely
removing a current file is insufficient. No integration was removed or reauthorized
by this documentation phase.

## Git and GitHub sequence

1. Finish the local license, attribution, reporting and publication-scope decisions.
   Commit each verified phase using English Conventional Commits.
2. Before a final audit, fetch authorized remote refs and inspect local/remote
   differences. Scan all branches and tags that would be public, including content
   deleted from the current tree. Review commit author/committer email privacy.
3. If a real secret is found, revoke or rotate it first. Agree any history rewrite
   with collaborators, preserve an appropriate private backup and re-scan the
   resulting refs. Design-image removal alone does not require a history rewrite.
   Do not force-push, delete tags or recreate the repository as routine cleanup.
4. Run the applicable checks from [testing](testing.md), then obtain authorization
   for a normal push. Verify the remote SHA and actual CI outcomes for that SHA.
5. Configure and verify branch rules, reporting and available security features.
   Review visible issues, PRs, releases, Actions logs/artifacts and other metadata
   that a Git-only scan does not cover. Obtain explicit owner authorization for
   the concrete public scope and visibility change; read back the resulting state.
6. Treat version tags, release archives, signed binaries, package registries and
   stores as separate publication actions. Preserve all required notices in each
   distributed artifact and verify its published state.

## Source release versus application release

Source can be published as experimental after the source gates are met. Signed
Android/iOS distribution, physical-phone storage/lifecycle acceptance, off-LAN
account journeys and model-specific controls have their own remaining gates in
[task status](task-breakdown.md) and [compatibility](compatibility.md). Do not turn
those pending tests into a claim that source licensing or repository publication
has already been completed. `publish = false` and `publish_to: none` may remain:
public source does not require publishing packages to a registry.

## Preparation validation

Executed locally on October 17, 2026:

- `python3 output/prototypes/homeverb-window-light-v1/validate.py`: passed,
  including 82 screens, 138 images, 555 local links and no layout overflow.
- `python3 scripts/validate_docs.py`: passed for Markdown targets and JSON syntax.
- `.venv/bin/python scripts/validate_contracts.py`: passed, 20 schemas,
  49 positive fixtures and 267 negative fixtures.
- `python3 scripts/validate_repository.py`: passed basic language/hygiene checks.
- `python3 scripts/validate_commits.py --all`: passed the existing history before
  the documentation phase commit.
- `.venv/bin/python -m unittest discover -s tests -v`: all 66 tests passed. The
  initial system-Python invocation failed to import `jsonschema`; the existing
  development environment resolved that dependency without an installation.
- `git diff --check`: passed.

No runtime code or maintained test-reference image changed. Rust/Flutter builds,
physical-device tests, full-history secret scanning, dependency-license auditing
and remote CI were not executed for this documentation/asset-cleanup phase.
Repository metadata reads were the only remote actions; no settings were changed,
source pushed, history rewritten or release published.
