# Embedded SDK

The SDK is implemented in Rust. `homeverb-core::Core` accepts injected Store, Adapter registry, Clock, and Policy values. `homeverb-host::open_mock` assembles the explicit simulator configuration without discovery. `Session` adds bounded operation registration, cancellation, foreground gating, and shutdown.

Use [the UI integration guide](development/03-ui-integration.md) for the JSON request contract and C ABI. [homeverb.h](../crates/ffi/include/homeverb.h) defines ownership and thread requirements. The UI never shells out to CLI or contacts a loopback HTTP service.

The [native UI contract](native-ui-contract.md) publishes the current API catalog, all 31 methods, startup/request/method-response schemas, and fictional native examples. `get_api_info` returns the compiled catalog without discovery or networking. Missing methods are unavailable; API presence does not confer device permissions. See [ADR-0019](adr/0019-native-ui-contract.md).

Core supports local listing/describing, room/device preferences, explicit discovery, cached/live reads, single-command and bounded-batch planning/execution, and recent outcomes. `Core::execution(request_id, operation)` retrieves one retained receipt without device access or current write grants; missing and retired IDs remain distinct. See [ADR-0014](adr/0014-local-receipt-lookup.md).

Core implements state-match verification and validates adapter-provided correlated completion under [ADR-0016](adr/0016-correlated-completion.md). The model-scoped Meiju adapter is registered for explicit account access; its cloud readings cannot establish physical verification. Real adapters must establish protocol correlation and authentication. An unavailable secret provider fails closed. Component-specific definitions resolve through `Descriptor::capability(component_id, capability_id)`; see [ADR-0018](adr/0018-component-capability-scopes.md). Do not select a definition by capability ID alone.

Trusted Rust hosts construct explicit component/argument/transport-scoped `Grant` values; ordinary JSON requests cannot provide them. `Policy::authorize` returns the permitted transports, which Core intersects with the command and action routes. Hosts with an existing deadline call `execute_with_operation`; `execute` starts a new command budget. See [Core development](development/02-core.md) and [ADR-0010](adr/0010-scoped-write-grants.md) for the unreleased Rust API change. The C ABI and Command envelope are unchanged.

JSON schema version remains `1.0`; native host requests reject other versions and unknown fields. Keep generated frontend bindings synchronized when changing DTOs. Public contract changes require an ADR and positive/negative fixtures.

Android embeds the same C ABI with a platform-only JNI credential initializer. The [native Android guide](development/08-android-native.md) supplies the Java package, arm64/x86_64 AAR builder and synthetic JVM/JNI verification. OS Keystore, APK and physical-phone acceptance remain separate; no Flutter page or background worker is introduced.
