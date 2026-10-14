# Publish a private GitHub repository

## Delivery status and target

Target: `arrenxxxxx/homeverb`, with private visibility and `main` as the default branch. This is a requested target, not a claim that the remote repository exists. The package contains local English commits; no remote creation or push was completed in the preparation environment because only read actions were available and there were no Git push credentials.

Do not paste an access token into a chat, source file, issue, or command argument. Authenticate locally using GitHub CLI's trusted browser flow.

## Requirements

Git, Python 3.10 or newer, and an authenticated GitHub CLI (`gh`) installation. The local account must be `arrenxxxxx` for the default target. Authentication must permit private-repository creation, content pushes including workflows, and issue-label management. Check the [official CLI installation and login documentation](https://cli.github.com/manual/).

## Publish from the ZIP with Git history

The distribution ZIP includes the `homeverb/` working tree and its clean `.git` history. Extract the whole archive, including hidden files, then run:

```bash
cd homeverb
gh auth login --hostname github.com --web --git-protocol https --scopes repo,workflow
python3 scripts/publish_github.py
```

Already authenticated users can skip login if the current account and permissions are correct. The helper uses the local `gh` credential helper for Git without writing secrets or changing global Git configuration.

The helper checks local status and commit syntax, checks the authenticated owner, creates a **private** repository, verifies visibility, pushes `main` without force, verifies the remote commit, applies English labels, and verifies private visibility again. It never creates a public repository, changes an existing repository's visibility, overwrites an unrelated remote, or force-pushes history.

## Restore from a Git bundle instead

A bundle is an optional portable backup of the same commits. It contains no login credentials or configured GitHub remote.

```bash
git clone homeverb-en.bundle homeverb
cd homeverb
git remote remove origin
python3 scripts/publish_github.py
```

Authenticate with `gh` first if needed. The local bundle origin must be removed because it is not a GitHub destination. A source-only archive without `.git` is not interchangeable with the history-preserving ZIP or bundle.

## Safe retries and conflicts

Without `origin`, the helper attempts to create a new repository. If that name is already taken or creation fails, it stops rather than adopting an unrelated existing repository. After successful creation, a rerun can use the exact configured target origin only after checking private visibility. Pushes are normal fast-forward pushes; history conflicts stop rather than overwrite.

If repository creation succeeds but a later push or label request fails, the helper reports failure and leaves the partial remote state intact. It does not delete a repository as compensation. Inspect the error, correct permissions, and rerun from the same checkout. GitHub labels are applied only after the uploaded `main` SHA has been verified.

## Offline preview

```bash
python3 scripts/publish_github.py --dry-run
```

This prints the intended target and actions without authenticating, creating a repository, pushing, or contacting GitHub. An offline preview or mocked test is not evidence of a successful publication.

## Manual equivalent

For a clean local Git repository with no origin:

```bash
gh repo create arrenxxxxx/homeverb --private --source . --remote origin \
  --description "Cross-brand appliance control with an embedded core, CLI, mobile app, and local MCP. No persistent service."
gh repo view arrenxxxxx/homeverb --json visibility,url
# Continue only after checking that visibility is PRIVATE.
git -c credential.helper= -c 'credential.helper=!gh auth git-credential' push -u origin main
```

The helper adds checks and label setup around this workflow. See the [official creation command](https://cli.github.com/manual/gh_repo_create). Remote CI, security reporting, branch protection, license choice, and any later public release still require separate verified actions.
