# Roadmap

Advance by demonstrated behavior, not calendar promises. [Task status](task-breakdown.md) is the detailed implementation record.

- **M0: baseline delivered.** Product boundaries, contracts, and validation tools.
- **M1: Mock MVP implemented; mobile acceptance pending.** Rust Core, persistent simulator, CLI, read-only stdio MCP, native ABI, and English/Chinese presentation resources. Flutter/Linux and Android packaging/emulator journeys have evidence on the frontend branch. Physical phones, iOS app integration, full localization/accessibility and target MCP host validation remain pending; see the [Flutter verification record](../apps/mobile/verification.md).
- **M2: first real integration.** Connect an existing Midea Meiju account, automatically retrieve bound Midea/Hualing appliances, and implement direct cloud reads and owner-authorized writes. Establish native authentication, secure sessions, exact-model evidence, and away-from-home operation. Add LAN as another connection to the same device; see the [integration sequence](development/05-midea.md).
- **M3: heterogeneous protocol and mobile experience.** A second protocol family; independently verified phone operation and dynamic controls.
- **M4: public trial.** Licensing, platform packages, security review, and actual CI evidence.

The no-daemon, no-project-backend boundary applies to every milestone.
