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

R8 and R9 were consulted for this English repository revision. Other entries are retained from the original design baseline and must be revalidated for implementation decisions. Do not repeat unverified claims of universal brand coverage or fixed API prices. Authentication, regional policy, protocol details, and licensing require fresh adapter-specific evidence.
