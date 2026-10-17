# Contributing

Contribute documentation, protocol research, adapters, and tests toward independent control without a persistent service. Start with [AGENTS.md](AGENTS.md), the [task breakdown](docs/task-breakdown.md), and the [compatibility matrix](docs/compatibility.md).

For build questions and ordinary bugs, see [SUPPORT.md](SUPPORT.md). Report
suspected vulnerabilities through [SECURITY.md](SECURITY.md), not a public PR.

## Workflow

Select a task with acceptance criteria. Write a contract or regression test, implement the change, and update documentation, examples, and compatibility evidence. Adding a rebranded product using the same protocol does not prove heterogeneous-platform support. Changes to architectural boundaries require an ADR first.

## Language and commits

Use English for documentation, comments, diagrams, examples, issue/PR text, and commit subjects/bodies. The only translated source exception is the owner-requested `zh-CN` UI catalog. Follow [Commit conventions](docs/commit-conventions.md): `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `ci`, `build`, `chore`, `style`, or `revert`. Use `docs` for documentation-only changes; do not mislabel a design as a delivered feature.

Enable the optional local hook:

```bash
git config core.hooksPath .githooks
```

## Pre-submission checks

```bash
python3 scripts/validate_docs.py
python3 -m pip install -r requirements-dev.txt
python3 scripts/validate_contracts.py
python3 scripts/validate_repository.py
python3 scripts/validate_commits.py --all
python3 -m unittest discover -s tests -v
```

Run the additional Rust, native runtime, and locale checks in [Testing](docs/testing.md) for implementation changes. These checks establish Mock behavior only; report mobile and hardware validation separately.

## Device evidence and privacy

Include exact model, firmware, region, vendor app, adapter version, verified capabilities, and limitations. Exclude credentials, unredacted captures, real IPs/MACs/serials, user names, and home layouts. Real-device writes require owner authorization.

## Pull requests

Explain the task, design changes, tests, no-daemon impact, authentication/policy changes, uncertain-write and duplicate-execution risks, and compatibility evidence. Use the [PR template](.github/PULL_REQUEST_TEMPLATE.md). Squash-merge with an English Conventional Commit title, or preserve a clean sequence of conforming commits; do not introduce untyped generated merge messages.

## Licensing

Contribute only material you have the right to submit. Record the origin, license,
copyright notices and modifications of copied or adapted code and assets in the
appropriate source notice and [third-party index](THIRD_PARTY_NOTICES.md).
Do not submit employer-owned or third-party material without permission.

The owner has deferred the project license decision; resolve
[LICENSE-DECISION.md](LICENSE-DECISION.md) and the inbound contribution terms before
accepting external contributions for public distribution. Do not assume that a
PR transfers copyright or grants an unspecified project license. No CLA or DCO
requirement is introduced by this preparation phase.

## Review conduct

Keep discussion respectful and focused on reproducible behavior and design
tradeoffs. Do not post harassment, personal information or private account/device
data. Maintainers may close off-topic or abusive contributions. A dedicated
community code and private conduct-reporting channel can be added when their
maintenance responsibilities and contact route are established.
