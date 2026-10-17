# Source publication delivery

Updated October 17, 2026. The owner authorized removing obsolete design history,
synchronizing GitHub and completing CI/security-reporting setup. The earlier
[source audit](open-source-audit.md) remains an evidence record of the pre-cleanup
history, not a claim that its old object IDs remain in the cleaned public branch.

## Completed history preparation

A complete private Git bundle was created outside the repository and verified
before filtering. A separate single-branch clone was filtered with
`git-filter-repo` 2.47.0, removing only `output/imagegen/` throughout its history.
The filtered tip was `21a7c0dc0ce91198311440f610a63a6569d3c2fd`, with 181 retained
commits. Its current tree exactly matched the original
`352fc5787f55757001387ce809ebf87178bc2db0` tree. Author/committer identity and
timestamps were checked and preserved for every retained commit.

All candidate refs were inspected: only `main` remained, no object path under the
removed directory was reachable, and the identified Mi Home marketing-image blob
was absent. The pack was approximately 146 MiB. This removes the identified
historical-image issue from the candidate source branch without changing current
application assets, the maintained prototype, code or package lockfiles.

Gitleaks 8.30.1 scanned the filtered history: the same five previously classified
fictional/prose findings and no additional finding. Documentation, repository
hygiene, contract checks and all 181 retained commit messages passed. The original
checkout's local Codex references and the private backup are not publication refs;
never use `git push --mirror` for this release.

When updating the existing remote branch, use an exact expected-old-SHA lease,
not an unrestricted force push. The inspected remote `main` was
`82fd26a7e2904fd367fcfaef89977f6e827bc8e1`. A concurrent remote change must stop
publication for inspection. Update other checkouts from the cleaned branch;
merging an old branch can restore removed history. GitHub cached old commit views,
unreachable server objects and third-party clones are outside this ref-rewrite
verification; the screenshot is a rights-review item, not an exposed credential.

## GitHub infrastructure findings

The repository was still private when checked. Both the read and enable calls
for private vulnerability reporting returned HTTP 404. GitHub documents that
[private vulnerability reporting is available for public repositories](https://docs.github.com/en/code-security/security-advisories/working-with-repository-security-advisories/configuring-private-vulnerability-reporting-for-a-repository).
Enable and read back that setting immediately when converting to public; do not
invent a working reporting address while the repository is private.

Existing CI runs `39033802547` and `39033802515` for the previous remote tip failed
before any job steps ran. The check annotations say recent account payments failed
or a spending limit needs to be increased. This is an account/runner-start block,
not an executed test failure. No billing or payment setting was changed. Do not
mark remote CI as passed until the exact intended commit's jobs actually finish.

## Remaining delivery checks

Verify the pushed SHA, final repository visibility, private reporting setting and
exact commit's CI conclusions. Review the final public source from an unauthenticated
request after conversion. Branch rules should require the actual workflow checks,
without deleting failing checks or weakening tests to make a release appear green.
No real appliance operation, package-registry publication or signed application
release is part of this source-delivery work.
