# Support

HomeVerb is under development. Start with the [Mock walkthrough](docs/development/01-run-mock.md),
[Flutter build guide](apps/mobile/README.md), [compatibility record](docs/compatibility.md)
and [testing guide](docs/testing.md). Exact model and platform evidence determines
support; a successful Mock run is not physical-device acceptance.

## Bugs and feature requests

Use the repository's [issue templates](https://github.com/arrenxxxxx/homeverb/issues/new/choose)
for bugs and feature requests. The repository is public. Search existing issues first and keep
one reproducible problem or feature request per issue. Requests are triaged on a
best-effort basis; there is no response-time or implementation guarantee.

Include the commit or version, operating system, entry point, exact model/firmware
when relevant, minimal reproduction, expected behavior and actual result. Prefer
fictional Mock fixtures. Distinguish `accepted`, `verified`, `failed`, `unknown`
and `partial`; do not resend an uncertain physical command merely to reproduce it.

Never attach account passwords, tokens, databases, pairing material, raw provider
responses, unredacted logs, household layouts or live device identifiers. Review
screenshots for account names and identifying details. Share only the minimum
redacted evidence needed; diagnostic uploads require explicit owner authorization.

## Security and compatibility

Report suspected permission bypass, credential exposure or unsafe duplicate
execution through the procedure in [SECURITY.md](SECURITY.md), not a public issue.
GitHub private vulnerability reporting is enabled; the security policy links to the form.

Use the adapter-request template for a new exact model or provider. State region,
public protocol references, requested capabilities and available test hardware.
Do not send credentials or assume brand-wide support. Physical testing and writes
require separate device-owner authorization.

## Contributions and project scope

Read [CONTRIBUTING.md](CONTRIBUTING.md) before proposing changes. The project keeps
an embedded Core, foreground App work, one-shot CLI and host-managed local stdio
MCP. It does not provide a hosted control service or unattended automation.
Source publication and signed application releases are separate milestones; see
[release preparation](docs/open-source-readiness.md).
