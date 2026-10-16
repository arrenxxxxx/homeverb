# References and evidence boundaries

Baseline source list: October 14, 2026. These sources inform external concepts. Product constraints, proposed interfaces, routing policies, and priorities are project design choices, not existing features of the cited projects. Recheck and pin target versions before implementation; retaining a source link does not validate every current platform policy.

| ID | Primary source | Limited use in this design |
|---|---|---|
| R1 | [MCP transports](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports) / [stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio) | Host-launched stdio subprocess architecture; no mandatory HTTP deployment |
| R2 | [W3C WoT Thing Description 1.1](https://www.w3.org/TR/wot-thing-description11/) | Property / Action / Event inspiration; no full WoT implementation claim |
| R3 | [RFC 8252: OAuth 2.0 for Native Apps](https://www.rfc-editor.org/rfc/rfc8252) | Native-app authorization, public clients, PKCE, and trust boundaries |
| R4 | [flutter_rust_bridge documentation](https://cjycode.com/flutter_rust_bridge/) | Potential Flutter/Rust binding mechanism; a project-specific prototype is still required |
| R5 | [midea-local](https://github.com/midea-lan/midea-local) | Existing local-protocol reference, not homeverb compatibility evidence |
| R6 | [SmartThings CLI](https://github.com/SmartThingsCommunity/smartthings-cli) | Existing API CLI, not evidence that homeverb native authentication works |
| R7 | [OpenCLI](https://github.com/jackwener/opencli) | CLI ergonomics reference, not a device driver or required dependency |
| R8 | [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) | Commit structure and `feat`/`fix` semantics |
| R9 | [GitHub CLI: gh repo create](https://cli.github.com/manual/gh_repo_create) | Authenticated private-repository creation from local source |
| R10 | [secret-service 5.2.0 API](https://docs.rs/secret-service/5.2.0/secret_service/) / [upstream source](https://github.com/hwchen/secret-service-rs) / [Secret Service protocol](https://specifications.freedesktop.org/secret-service/latest/) | Implemented Linux library dependency, MIT OR Apache-2.0; DH sessions, exact collection searches, native get/create/delete; no source port or mobile claim |
| R11 | [Midea AC LAN](https://github.com/georgezhao2010/midea_ac_lan) | HA community integration evidence for the selected AC/E1/CA families; reference only, not a HomeVerb runtime dependency |
| R12 | [Midea Auto Cloud](https://github.com/sususweet/midea_auto_cloud) | HA community cloud account enumeration, state/control, and capability-mapping evidence; Apache-2.0 reference, not native-client admission or physical verification |
| R13 | [Midea official v3 API](https://mis-cdn.smartmidea.net/docs/control-midea-cloud-devices/cloud-2-cloud-v3-api.html) / [Android SDK](https://iot.midea.com/docs/control-midea-cloud-devices/android-sdk.html) | Required developer secret for token/signing operations and documented third-party-server token flow; no established independent native-client admission |
| R14 | [Apple Keychain access groups](https://developer.apple.com/documentation/security/ksecattraccessgroup), [public SecItem declarations](https://github.com/apple-oss-distributions/Security/blob/main/keychain/headers/SecItem.h), [public status constants](https://github.com/apple-oss-distributions/Security/blob/main/base/SecBase.h) and [core-foundation-sys 0.8.7](https://docs.rs/core-foundation-sys/0.8.7/core_foundation_sys/) | iOS-only native credential implementation under ADR-0062; existing MIT OR Apache-2.0 CF bindings; public Apple API declarations, no vendor source implementation port or signed-device claim |

R8 and R9 were consulted for this English repository revision. R1-R4, R6, and R7 are retained from the original design baseline and must be revalidated for implementation decisions. Do not repeat unverified claims of universal brand coverage or fixed API prices. Authentication, regional policy, protocol details, and licensing require fresh adapter-specific evidence.

R10's downloaded crate source and the resolved zbus 5.19.0 connection lifecycle were inspected for the Linux implementation on October 14, 2026. Cargo.lock records exact dependency versions. This source review and synthetic service test do not replace a dependency vulnerability audit or the project's license decision.

R5, R11, and R12 were inspected on October 15, 2026. The [Midea/Hualing integration guide](development/05-midea.md) records exact revisions, MIT/Apache-2.0 licensing, cloud versus LAN evidence, upstream runtime differences, and authentication questions. No device or account was contacted, and no upstream credential material was imported into HomeVerb.

R13 was read on October 15, 2026. The official documents and pinned community implementation expose different authentication paths. [ADR-0022](adr/0022-meiju-community-cloud-admission.md) records the distinction and the proposed owner decision; finding no documented PKCE support does not prove that a provider-approved native alternative cannot exist.

R14 was inspected on October 16, 2026. Apple's C header distinguishes iOS
`kSecValuePersistentRef` queries from macOS `kSecMatchItemList`, which the generic
web reference does not consistently distinguish. Use the platform-specific header
and the signed runtime checklist together. The [iOS decision](adr/0062-ios-keychain-credentials.md)
also records the deliberately deprecated no-prompt option and exact evidence limits.
