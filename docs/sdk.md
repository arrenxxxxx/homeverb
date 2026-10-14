# Core SDK and host interfaces

## Goal

CLI, Flutter bindings, and MCP handle input/output and host interaction. The SDK owns access and control semantics. It does not start a server, require a persistent global runtime, or include hidden telemetry.

```text
CoreBuilder
  .storage(Storage)
  .secrets(SecretStore)
  .adapters(AdapterRegistry)
  .policy(PolicyEngine)
  .clock(Clock)
  .network(NetworkPolicy)
  .build() -> Core

Core.list_devices(Query, Context)
Core.describe_device(DeviceSelector, Context)
Core.read_state(ReadRequest, Context)
Core.execute(Command, TrustedPrincipal, Context)
Core.begin_auth(AuthRequest, TrustedInteraction, Context)
Core.close()
```

This is responsibility-level pseudocode, not SDK source. Rust signatures, asynchronous interfaces, and mobile binding generation are prototype decisions.

## FFI boundary

Exchange versioned DTOs and structured errors. Cancellation uses an explicit operation ID and cancel API, not an assumption that destroying a UI object will stop networking. Exchange secrets through restricted platform interfaces or references rather than unnecessary Dart strings or log copies.

Do not block the UI thread with networking. The host owns the runtime lifecycle. Do not use a second local HTTP server as the default Flutter/Rust bridge.

## Dependency injection and tests

Replace clocks, networking, random IDs, and secret stores for deterministic deadline and state-machine tests. Tests use mock adapters and fictional devices by default, not real vendor endpoints. Business modules must not import vendor SDKs and add brand-selection branches in the upper layers.

## Versions

Version the SDK, JSON `schema_version`, and adapters separately. FFI DTO changes require synchronized generated bindings on both sides. Reject incompatible major versions during initialization and preserve additive compatibility where possible.
