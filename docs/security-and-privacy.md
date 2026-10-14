# Security and privacy design

## Threat boundaries

Protect vendor accounts, appliance-control authority, device identities/home layout, and operation history. Threats include malicious LAN responses, untrusted AI input, third-party adapters, log disclosure, uncertain physical state, and local concurrency.

If the local OS user is fully compromised, application encryption cannot guarantee protection of unlocked secrets. In-process adapters are not sandboxed. Use least privilege, reviewed code, and refusal to load unknown plugins.

## Authorization

Default to read access. Write grants bind a principal, device ID, component, capability, action, argument bounds, and permitted route. Trusted host configuration establishes the principal; ordinary requests and models cannot select a privileged identity. Renaming an alias does not change authorization bindings.

Generic AI tools do not expose high-risk actions initially. Future support requires trusted user interaction, short-lived approval, and auditable binding to the exact action. Model-supplied confirmation, special prompts, and generic `execute` cannot bypass policy.

## Connections and endpoints

Verify vendor-cloud TLS; never globally disable certificate checking to fix connectivity. Document weak-security LAN protocols and constrain network targets. A discovered URL is not automatically a trusted endpoint.

Use viable native OAuth flows. Shared secrets embedded in a public application are not confidential backend credentials. [R3](references.md) Do not build an account relay or extract another app's private data to impersonate legitimate authorization.

## Data minimization

No default telemetry. Diagnostic exports use redacted allowlists. Remove tokens, Authorization, cookies, phone numbers, emails, device serials, MACs, LAN IPs, and secrets embedded in user input. Debug mode does not authorize credential disclosure. Users can preview diagnostics before voluntarily sharing them.

## Supply chain

Compile reviewed adapters into the application. Record upstream source, license, ported files, and version. Add lockfiles, dependency review, and vulnerability checks with the actual implementation. This documentation baseline has not passed a future application's security audit.

## Tests

No secrets in stdout, stderr, logs, or exports. Malicious discovery does not trigger arbitrary endpoints. Authorization rejects before transmission. Timeouts do not duplicate writes. Backgrounding does not keep the app alive. Auth listeners close on timeout. Forged MCP admin fields are ineffective. Report issues using [SECURITY.md](../SECURITY.md).
