# Security and privacy design

## Threat boundaries

Protect vendor accounts, appliance-control authority, device identities/home layout, and operation history. Threats include malicious LAN responses, untrusted AI input, third-party adapters, log disclosure, uncertain physical state, and local concurrency.

If the local OS user is fully compromised, application encryption cannot guarantee protection of unlocked secrets. In-process adapters are not sandboxed. Use least privilege, reviewed code, and refusal to load unknown plugins.

## Authorization

Default to read access. Write grants bind a principal, device ID, component, capability, action, argument bounds, and permitted route. Trusted host configuration establishes the principal; ordinary requests and models cannot select a privileged identity. Renaming an alias does not change authorization bindings.

The current Core enforces exact device/component/action binding, local argument-schema constraints, and allowed transports through trusted in-memory grants. Its principal is fixed by host construction. Each command must match one complete grant; permissions for separate parameters are not merged. Eligible transports are the intersection of matching grants, action support, and requested routing policy. Multiple complete matching grants may contribute routes. Invalid grant schemas add no authority. Composite steps cannot expand the parent command's route authority. See [ADR-0010](adr/0010-scoped-write-grants.md).

Generic AI tools do not expose high-risk actions initially. Future support requires trusted user interaction, short-lived approval, and auditable binding to the exact action. Model-supplied confirmation, special prompts, and generic `execute` cannot bypass policy.

Meiju MCP defaults to four read-only tools. A trusted host may explicitly load the
same exact-device power/fan [startup grants](cli.md#scoped-meiju-controls) used by
the CLI; only then are plan/execute tools visible. Core policy remains authoritative
even if a client forges a hidden tool call. Grants are immutable for that MCP
session: file edits require host restart, and stopping a session cannot roll back
an already sent command. No tool accepts account credentials, a different profile,
principal or replacement permissions. See [ADR-0038](adr/0038-scoped-meiju-mcp.md).

## Connections and endpoints

Verify vendor-cloud TLS; never globally disable certificate checking to fix connectivity. Document weak-security LAN protocols and constrain network targets. A discovered URL is not automatically a trusted endpoint.

Use viable native OAuth flows. Shared secrets embedded in a public application are not confidential backend credentials. [R3](references.md) Do not build an account relay or extract another app's private data to impersonate legitimate authorization.

The planned [foreground authorization-sharing flow](adr/0008-foreground-authorization-sharing.md) requires authenticated encrypted peer transfer, explicit consent on both devices, provider-specific eligibility, and receiver secure storage. A discovered peer name is untrusted. Its temporary local-network listener exposes no appliance-control API, and clearing a transfer record does not revoke the recipient's authorization.

## Data minimization

No default telemetry. Diagnostic exports use redacted allowlists. Remove tokens, Authorization, cookies, phone numbers, emails, device serials, MACs, LAN IPs, and secrets embedded in user input. Debug mode does not authorize credential disclosure. Users can preview diagnostics before voluntarily sharing them.

## Local diagnostic reports

`Core::diagnostics`, CLI `diagnostics`, and native `{"method":"diagnostics"}` generate the same allowlisted [report contract](../schemas/diagnostics.schema.json). They query local metadata and retained receipts only; no adapter or secret-store method is called. There is no background generation or upload. The native method retains the host's foreground/closed-state gate. Review the result before explicitly saving or sharing it.

The report includes the application version and report-generation time; device/room/transport counts; and at most 100 newest retained receipt summaries. Each summary contains status, an allowlisted error code, verification kind, warning count, direct-child outcome counts, and at most 32 transport/phase/duration entries. Additional attempts are counted as omitted. Unknown error codes become `OTHER`; they are never copied through as arbitrary strings. Store failures return a fixed error message. Identifiers, credentials, names, brand/model strings, state values, command arguments, operation timestamps, evidence notes, and raw errors/warnings are never serialized into the report.

The projection constructs dedicated diagnostic DTOs instead of serializing a private record and removing selected fields. New fields on Device, Execution, or Attempt therefore do not automatically become exportable. Tests inject private markers into stored metadata, results, nested items, and errors, and validate the actual CLI/native report against a closed schema. Do not add an include-secrets or unredacted mode.

Counts and outcomes can still reveal usage patterns; this is a minimized diagnostic report, not an anonymity guarantee. Counts are best-effort local reads, not one atomic cross-table snapshot or a connectivity check. Parent and child receipt rows can both appear, so row counts are not physical-action counts. The sample does not restore retired details or imply complete history. This reporting boundary does not certify every future adapter's live logs or errors as redacted.

## Supply chain

Compile reviewed adapters into the application. Record upstream source, license, ported files, and version. Add lockfiles, dependency review, and vulnerability checks with the actual implementation. This documentation baseline has not passed a future application's security audit.

## Tests

No secrets in stdout, stderr, logs, or exports. Malicious discovery does not trigger arbitrary endpoints. Authorization rejects before transmission. Timeouts do not duplicate writes. Backgrounding does not keep the app alive. Auth listeners close on timeout. Forged MCP admin fields are ineffective. Report issues using [SECURITY.md](../SECURITY.md).
