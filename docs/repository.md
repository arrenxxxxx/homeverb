# Repository layout and Git workflow

## Current content

```text
homeverb/
  README.md
  AGENTS.md                 # Canonical AI contributor rules
  agent.md                  # Pointer preserving the requested filename
  CONTRIBUTING.md
  SECURITY.md
  CHANGELOG.md
  LICENSE-DECISION.md        # No project license selected yet
  docs/
    index.md
    architecture.md
    capabilities.md
    commit-conventions.md
    publishing.md
    adr/                    # Architecture decisions
    diagrams/               # SVG, PNG, Mermaid
  schemas/                  # Machine-checkable JSON contracts
  examples/                 # Fictional data only
  scripts/                  # Validation and private publishing tools
  tests/                    # Repository-tool tests, not device tests
  .githooks/commit-msg       # Optional local commit guard
  .github/
    ISSUE_TEMPLATE/
    PULL_REQUEST_TEMPLATE.md
    labels.json
    workflows/docs.yml
```

## Proposed implementation layout - not present

```text
crates/domain/         # Types and semantics
crates/core/           # Routing, execution, authorization
crates/adapter-api/    # Extension contracts
crates/storage/        # Local persistence
crates/adapters/       # Real protocols
apps/cli/              # Independent commands
apps/mcp/              # Local stdio host entry point
apps/mobile/           # Flutter with in-process FFI
```

Do not create empty implementations just to imply completeness. Add real build scripts, lockfiles, tests, and CI only when they correspond to functioning source projects.

## Git baseline

The intended default branch is `main`. The distribution includes a portable Git bundle containing English, typed local commits. A bundle has no live remote access credentials. Restore it using the steps in [Private publishing](publishing.md).

For a fresh source-only checkout without history:

```bash
git init -b main
git add .
git commit -m "docs: establish English no-daemon design baseline"
```

This requires a locally configured Git author. The command initializes local history only; it does not create a remote repository.

## Branches and review

Prefer small PRs and branches such as `feat/native-auth`, `fix/write-verification`, or `docs/adapter-contract`. Use the [commit convention](commit-conventions.md) for commits and PR titles. Each review explains lifecycle, physical-write semantics, capability, and compatibility impact.

The CI in this package validates documentation and contracts. Add actual application builds, lint, and platform tests later. Placeholder workflows are not evidence of product quality. Do not publish credentials, captures, local databases, or real device identities.
