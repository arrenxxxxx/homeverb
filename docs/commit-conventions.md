# Commit, pull-request, and label conventions

## Language

All repository documentation, diagram labels, example messages, code comments, issue/PR text, and commit subjects/bodies are English. Stable public identifiers are English. The automated language guard rejects non-Latin alphabetic scripts in maintained text; it is a regression guard, not a semantic proof that prose is good English. Reviewers still check wording.

## Commit format

```text
<type>(<optional-scope>)[!]: <English imperative summary>

<optional English explanation>

<optional footer, such as BREAKING CHANGE: ...>
```

The selected convention is [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/). Use `feat`, not `feature` or `fateture`. Keep subjects concise; this repository enforces a maximum of 100 characters. Lowercase types and scopes are required; scopes may contain letters, numbers, hyphens, and slashes.

| Type | Use |
|---|---|
| feat | Add an actual application/library capability |
| fix | Correct a reproducible defect |
| docs | Change documentation, design descriptions, or diagrams |
| test | Add or update tests and fixtures |
| refactor | Restructure without intentionally changing behavior |
| perf | Improve performance with evidence |
| ci | Change continuous integration |
| build | Change build tooling or dependencies |
| chore | Repository maintenance or ancillary tooling |
| style | Formatting without behavioral changes |
| revert | Revert a previous change with an explanation |

A proposed feature described only in Markdown is `docs`, not proof of an implemented `feat`. Do not manufacture a `fix` commit merely to use every label.

## Examples

```text
feat(core): add capability-aware command validation
fix(midea): preserve unknown outcomes after lost acknowledgements
docs: document the no-daemon architecture in English
test(contracts): reject unsafe retryable write outcomes
ci: validate English documentation and commit messages
chore(repo): add a private GitHub publishing helper
```

Except where they appear in actual repository history, these are illustrative commit messages, not implementation claims. Breaking changes use `!` and/or an English `BREAKING CHANGE:` footer, with a corresponding ADR where required.

## Pull requests and branches

Use the same format for PR titles. Branch examples: `feat/native-auth`, `fix/uncertain-write`, `docs/adapter-contract`. Prefer a squash merge with a conforming title, or rebase a clean sequence of conforming commits. Unmodified generated `Merge pull request ...` subjects do not satisfy this repository's stricter history policy.

## GitHub labels

The English label catalog is [labels.json](../.github/labels.json). Type labels include `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `ci`, `build`, `chore`, `style`, and `revert`. Additional labels identify `adapter`, `security`, `blocked`, and `breaking-change`.

Commit prefixes and issue/PR labels are distinct metadata. A typed commit does not automatically label an issue. The publishing helper explicitly creates or updates this label catalog after a verified private push.

## Local and CI checks

```bash
git config core.hooksPath .githooks
python3 scripts/validate_repository.py
python3 scripts/validate_commits.py --all
python3 -m unittest discover -s tests -v
```

The hook validates new commit messages. CI validates committed text and history, plus PR titles when provided. Hooks are optional local conveniences; server-side enforcement requires actual GitHub settings and successful CI. This package does not claim that branch protection or remote rules have been configured.
