# Contributing

Contribute documentation, protocol research, adapters, and tests toward independent control without a persistent service. Start with [AGENTS.md](AGENTS.md), the [task breakdown](docs/task-breakdown.md), and the [compatibility matrix](docs/compatibility.md).

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

Before copying third-party code, record its source, license, and modifications. The project license remains undecided; resolve [LICENSE-DECISION.md](LICENSE-DECISION.md) before any public release.
