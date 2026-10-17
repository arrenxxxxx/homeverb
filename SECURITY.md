# Security policy

HomeVerb implements an embedded Rust Core, CLI, local stdio MCP, native bridges
and a Flutter application with experimental appliance integration. Development
builds and isolated acceptance results do not establish a supported production
release. See [compatibility](docs/compatibility.md) and
[security and privacy](docs/security-and-privacy.md) for the current boundaries.

## Report privately

Do not post vulnerabilities, credentials, live device identifiers, packet captures
or sensitive exploit details in public issues, discussions or pull requests.
Use GitHub's [private vulnerability report form](https://github.com/arrenxxxxx/homeverb/security/advisories/new)
while signed in, or open the repository's Security tab and choose **Report a
vulnerability**. The repository was made public on October 17, 2026; private
vulnerability reporting was enabled and its API setting read back as `enabled: true`.
Reports submitted through this form are private, not public issues.
No maintainer email is published as a reporting address by this project.
If the form is unavailable, do not substitute a public issue or an assumed email
address for a private channel.

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
