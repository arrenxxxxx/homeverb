# Repository and release publishing

## Current repository

The existing target is `arrenxxxxx/homeverb`, with `main` as the default branch.
On October 17, 2026 a GitHub metadata read verified private visibility. Do not run
a repository-creation command for this existing checkout. The earlier private
bootstrap helper remains available for its original purpose; it is not a public
release tool.

Open-source preparation is tracked in the [release checklist](open-source-readiness.md).
The owner selected [Apache-2.0](../LICENSE); preparation includes the existing
Meiju integration. Full-history review, dependency rights, private security
reporting and release-commit checks
remain prerequisites. Preparing files or a local commit does not change visibility.

Do not paste access tokens into chat, files or command arguments. Authenticate
with the GitHub CLI's trusted browser flow when needed; see the
[official CLI documentation](https://cli.github.com/manual/gh_auth_login).

## Inspect before pushing

These commands inspect the current checkout and remote configuration. Fetch
updates remote-tracking refs but does not upload local commits:

```bash
git status --short
git branch -vv
git remote -v
git fetch origin
git log --oneline origin/main..main
git log --oneline main..origin/main
git diff --stat origin/main...main
gh repo view arrenxxxxx/homeverb --json visibility,defaultBranchRef,url
```

Review every intended commit and resolve divergence without discarding unrelated
work. Audit all refs intended for public visibility, not just the branch tip.
Removing a file or adding it to `.gitignore` leaves its historical contents intact.
Keep secrets and private scanner reports outside the repository. See the
[history-review requirements](open-source-readiness.md).

## Authorized push and verification

After the applicable [checks](testing.md) pass and the owner authorizes the push,
use a normal fast-forward update:

```bash
git push origin main
git rev-parse HEAD
gh api repos/arrenxxxxx/homeverb/commits/main --jq .sha
gh run list --repo arrenxxxxx/homeverb --branch main --limit 10
```

Require the remote SHA to equal the intended local commit. Inspect the runs for
that exact SHA and their job conclusions; an older successful run or a local
check is insufficient. Do not bypass failed checks or use force push as an
integration shortcut. Branch protection and required checks must name the
checks actually present in this repository.

## Public source publication

Finish all source gates in [open-source readiness](open-source-readiness.md) first.
Then present the exact commit, license, Meiju scope, history-review results and
repository settings for the owner's explicit publication decision. Changing
visibility exposes reachable history and repository metadata, not only the latest
source tree. Review issues, pull requests, releases and Actions artifacts/logs too.

Change visibility through the repository settings only after that decision, then
read back visibility and verify unauthenticated access to the intended source and
license. Enable a working private security-reporting channel and update
[SECURITY.md](../SECURITY.md) with the verified procedure. An API error is not
evidence that a setting is enabled. Check the reporting path again after changing
visibility because private and public repository behavior can differ.

Public source is separate from creating tags, GitHub Releases, publishing crates,
shipping APKs/IPA files or uploading diagnostics. Each requires its own authorized
scope. Include third-party notices in distributed archives/binaries and record
actual installation, signing and model acceptance separately from Mock tests.

## Existing private bootstrap helper

`scripts/publish_github.py` supports a clean checkout with no origin, or its exact
already-configured private target. It checks the authenticated owner, repository
visibility, commit syntax and clean status; it creates a private repository only
when no origin exists, pushes without force, verifies the remote SHA, applies
labels and verifies private visibility again. If a step fails it leaves partial
remote state for inspection instead of deleting a repository as compensation.

The offline preview does not authenticate, contact GitHub or change anything:

```bash
python3 scripts/publish_github.py --dry-run
```

Do not use this helper after public conversion: it intentionally refuses a public
repository. Do not loosen that guard to implement public releases. A future
release helper needs its own reviewed workflow and explicit owner authorization.
