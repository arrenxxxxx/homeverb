# Repository layout

```text
apps/cli/                  # One-shot CLI and local MCP host with explicit startup policy
crates/domain/             # Versioned DTOs and action-schema validation
crates/core/               # Local organization, policy, routing, execution, verification
crates/storage/            # SQLite metadata/journal, migration, retention, and device locks
crates/secret-store/       # Linux Secret Service and Android JNI credential backends
crates/mock/               # Persistent fictional appliance and fault scenarios
crates/midea/              # Meiju account/cloud protocols, protected bindings and scoped controls
crates/host/               # Composition root and foreground session contract
crates/ffi/                # Native in-process C ABI and public header
packages/android/          # Android Keystore/ciphertext package and synthetic JVM fixtures
packages/presentation/     # en-US/zh-CN catalogs and framework-neutral helpers
schemas/                   # Runtime JSON contracts
examples/                  # Fictional positive fixtures
scripts/                   # Validation, runtime acceptance, publishing helper
tests/                     # Repository-tool tests
docs/development/          # Progressive contributor guides
docs/adr/                  # Architecture decisions
```

Rust package names use the `homeverb-` prefix. The executable is `homeverb`. `Cargo.lock` records resolved dependency versions and belongs in implementation commits; the supported minimum compiler is Rust 1.96. The local stable toolchain was verified at 1.96.0. Flutter/Android project files are not generated placeholders; add them when implementing and validating the actual frontend.

Runtime data belongs outside the repository. Preserve unrelated local changes and never stage IDE files or device data. Prefer small commits with English [Conventional Commit](commit-conventions.md) subjects. Publishing requires explicit owner authorization and remote read-back; see [publishing](publishing.md).
