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

## Verified push and public conversion

Only `main` was pushed, using the exact old-SHA lease
`82fd26a7e2904fd367fcfaef89977f6e827bc8e1`. GitHub and an independent bare clone
both confirmed `3185e0e8f68e193c1ed0b06d63467cbb4478544d`. The independent clone
contained no reachable `output/imagegen/` path or identified marketing-image blob.
Local Codex refs and the private backup were not pushed.

The owner then explicitly authorized changing `arrenxxxxx/homeverb` to public.
GitHub returned `visibility: public` and `private: false`; an unauthenticated API
request independently confirmed this state, default branch `main` and the
Apache-2.0 license. Private vulnerability reporting was enabled and read back as
`enabled: true`. [SECURITY.md](../SECURITY.md) now links to the private report form.
The anonymous report-form request redirects to GitHub sign-in, as expected.
No sensitive test report was submitted to exercise the intake. The issue chooser
also links to private reporting.

The public metadata inventory contains only `main`, with no tags, issues/pull
requests, releases or Actions artifacts. `main` has no branch protection rule;
CI execution does not imply merge enforcement.

Update other checkouts from the cleaned branch; merging an old branch can restore
removed history. GitHub cached old commit views, unreachable server objects and
third-party clones are outside this ref-rewrite verification; the screenshot is
a rights-review item, not an exposed credential.

## CI evidence

The earlier private runs `39033802547` and `39033802515`, and the first attempts
of `39038575679` and `39038575653`, failed before any job steps ran. Their check
annotations cited account payments/spending limits. No billing setting was changed.
Private vulnerability-reporting calls had returned 404; GitHub documents that
[the feature requires a public repository](https://docs.github.com/en/code-security/security-advisories/working-with-repository-security-advisories/configuring-private-vulnerability-reporting-for-a-repository).
These initial infrastructure findings are superseded by verified public conversion.

Both workflows for `3185e0e` were rerun after public conversion and started actual
job steps. [Documentation checks](https://github.com/arrenxxxxx/homeverb/actions/runs/39038575653)
passed. The [Mock/native run](https://github.com/arrenxxxxx/homeverb/actions/runs/39038575679)
records Linux runtime, Android packaging and iOS Simulator results separately.
For subsequent revisions, consult the [latest main workflows](https://github.com/arrenxxxxx/homeverb/actions?query=branch%3Amain)
and match their head SHA to the intended commit. This record does not substitute
an earlier run for the latest revision's results.

The initial public runs on `3185e0e` and `7c91bec` passed documentation, Linux
runtime and Android checks, plus iOS object tests and XCFramework packaging.
Their isolated iOS acceptance steps failed before app launch: `simctl` container
lookup and installation respectively exceeded the former 60-second setup limit.
The runner now allows a bounded 240 seconds for each of those cold-setup calls,
matching boot readiness, and prints stage progress. It performs no retry and
retains the app/report deadlines, assertions, exact-device cleanup and production
Keychain policy. Those earlier failures remain recorded, not retroactively passed;
the latest exact-commit run supplies the result for the adjusted runner.

Inspection also found the isolated iOS app still expected native API 1.22.0 while
the production contract is 1.23.0. Its exact assertion now matches that contract,
with a portable cross-file check to catch future drift. No production API or
acceptance assertion was removed.

## Scope

This delivery changes source availability and documentation, not runtime behavior,
APIs, schemas, storage, authority or physical-device support. No real appliance
operation, package-registry publication or signed application release is part of
this source-delivery work. Source availability does not close the separate
[application and physical acceptance tasks](task-breakdown.md).
