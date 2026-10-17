# Security policy

HomeVerb implements an embedded Rust Core, CLI, local stdio MCP, native bridges
and a Flutter application with experimental appliance integration. Development
builds and isolated acceptance results do not establish a supported production
release. See [compatibility](docs/compatibility.md) and
[security and privacy](docs/security-and-privacy.md) for the current boundaries.

## Report privately

Do not post vulnerabilities, credentials, live device identifiers, packet captures
or sensitive exploit details in public issues, discussions or pull requests.
Use an existing verified private channel to the repository owner if one is
already available to you. No maintainer email is published by this project.

A public reporter intake is **not ready**. On October 17, 2026 the repository was
verified private; the GitHub private-vulnerability-reporting API returned HTTP 404.
That response does not establish whether reporting is enabled or accessible.
Before publication, the owner must enable and verify GitHub private vulnerability
reporting, or publish a verified private alternative here. Do not send sensitive
reports to an assumed address or substitute a public issue for a private channel.

## Useful report contents

Include the affected commit/version and component, a minimal fictional or redacted
reproduction, expected and observed behavior, and impact. Especially relevant are
credential exposure, identity or permission bypass, uncertain-write replay,
duplicate physical actions, lifecycle leaks and unintended background work.
Never include a real password or token merely to demonstrate a problem. Avoid
operating a physical appliance without its owner's explicit authorization.

## Supported versions and disclosure

There is currently no designated supported public release series or security
backport commitment. Development reports should name the exact commit; maintainers
will assess fixes against current development. No response or patch SLA is promised.
A release must publish its supported version range and update path before claiming
security maintenance for that version.

After private contact is established, coordinate a minimal reproduction, affected
versions, remediation and disclosure timing with the owner. Publish sanitized
advisories and affected/fixed versions when appropriate, without including live
credentials or household data. The reporting channel, release support policy and
actual repository settings are tracked in the
[publication checklist](docs/open-source-readiness.md).
