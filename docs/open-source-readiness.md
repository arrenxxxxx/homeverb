# Open-source release preparation

Status: preparation in progress, October 17, 2026. The owner requested open-source
preparation and removal of unused design studies, then selected Apache-2.0
after initially deferring the main license decision. The owner subsequently authorized obsolete-history cleanup, remote synchronization
and CI/security-reporting setup; see [delivery status](publication-status.md).
Package publication and appliance writes remain outside this source-delivery scope.

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
| Project license | Complete: owner selected Apache-2.0 | Root LICENSE, license decision, README, Rust/JavaScript metadata and contribution terms agree; existing rights-holder notices remain intact |
| Meiju source scope | Included in owner-requested project-wide preparation | Existing reviewed integration and upstream notices are included; no duplicate owner approval. Unofficial service access is not vendor endorsement. |
| Third-party and asset obligations | Source inventory reviewed; identified historical screenshot removed from candidate refs | See the [audit](open-source-audit.md): 320 Rust and 74 Dart/SDK entries, added Flutter notice, the identified historical promotional screenshot is excluded by the [history cleanup](publication-status.md); binary notices remain separate |
| Private vulnerability reporting | Pending public conversion: GitHub reporting is public-repository only | Enable and verify reporting at public conversion, then update SECURITY.md; private-repository read/enable calls returned 404 |
| History and sensitive data | Filtered history re-scanned; same five classified findings | Gitleaks 8.30.1 findings classified as fictional/prose; author identity is preserved; the historical screenshot is absent from the candidate refs. Verify final pushed refs. |
| Repository settings and CI | Pending exact-commit run; prior private runs blocked by billing | See [delivery status](publication-status.md); verify exact commit results and required checks without claiming an unstarted job passed |
| Public-facing status | Preparation complete, final release review pending | README/support/compatibility/changelog describe the chosen release scope without claiming untested models or signed phone acceptance |

The subsequent [source audit](open-source-audit.md) used a checksum-verified
Gitleaks 8.30.1 binary in private temporary storage. History and tracked-source
scans returned five reviewed fictional/prose findings, not a zero-finding result.
Commit metadata and 522 historical image blobs received separate review. The
report records limits and specific remaining decisions; it does not establish
absence of every secret or grant third-party rights. Public Meiju material was
traced to the pinned upstream source and is included in the requested project scope.
Do not blanket-ignore a source directory or print secret values into a report.

## Source publication scope

Publish source and development instructions first, with Mock as the reproducible
starting point and explicit experimental status for exact Meiju mappings. Do not
claim official vendor endorsement, general brand compatibility, signed mobile
release support or complete official-App parity.

The owner-requested source scope includes the existing Meiju code and pinned
public app-compatibility constants. Keep their provenance and license notices.
Do not turn the earlier personal-MVP wording into a new approval requirement.
A Mock default does not remove this code from an archive or change its unofficial
status. No new integration, credential use or physical write is authorized by
this source-preparation scope.

## Git and GitHub sequence

1. Keep the selected Apache-2.0 license and finish attribution, reporting and
   publication-scope decisions.
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
those pending tests into a claim that repository publication or application
acceptance has already been completed. The Apache-2.0 license is selected.
`publish = false` and `publish_to: none` may remain: public source does not require
publishing packages to a registry.

## Initial preparation validation

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

The subsequent Apache-2.0 selection was checked with
`cargo metadata --offline --locked --no-deps --format-version 1`: all nine Rust
packages inherit `Apache-2.0` and remain non-publishable. The JavaScript manifest
uses the same identifier and retains `private: true`. The root license matches
the complete standard Apache-2.0 text already retained with the upstream port.
Documentation, repository hygiene and contract validation passed again. This
license/metadata change does not alter runtime behavior, APIs or device support;
no application build, physical test or remote publication was performed.
