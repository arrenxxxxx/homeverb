# Frequently asked questions

## How is this different from Home Assistant?

This is a design for an independent control library and client tools, without a required central instance. It is not a complete home-automation platform. Starting one supported client should be enough to connect and control.

## Are there no servers anywhere?

There is no project-operated backend. A device may still require direct access to its vendor cloud. Platforms that cannot satisfy native-client authentication remain unsupported or blocked.

## Why is the MCP server not a persistent service?

It is a local stdio child process owned by an AI host session, not a separately maintained daemon. Waiting for tools during that session is allowed; proactive polling and gateway behavior are not.

## Can I control a LAN-only device away from home?

Only if the current process already has a route to the device network. The project does not deploy relays, VPNs, or NAS proxies to provide one.

## Can the project start an appliance while no client is running?

No. It does not continuously evaluate conditions or schedule future commands. An appliance's existing native functions are unaffected, and an already-started physical action may continue.

## Do clients synchronize automatically?

No. Shared code is not shared state. Phones and desktops authenticate and save configuration independently. Same-user desktop CLI/MCP can share explicitly selected local profile storage.

## Why can authentication block a platform?

Region, account type, protocol, or native-client limitations may prevent safe access. A confidential shared secret cannot be hidden reliably inside an app package or CLI. Mark the integration blocked rather than adding a hidden backend.

## Can the documented homeverb commands run now?

No. This is a design, schema, example, diagram, and repository-tooling package. Its validation scripts work; the appliance-control executable does not exist yet.

## Why keep both agent.md and AGENTS.md?

The former preserves the requested filename and points to the latter, which holds the canonical AI contributor rules. This prevents conflicting copies.

## An earlier illustration mentioned an optional gateway. Is it supported?

No. Early concept illustrations are not normative. The maintained diagrams and ADRs explicitly exclude persistent gateways, background automation, and remote HTTP MCP.

## Why use feat instead of feature or fateture?

`feat` is the selected Conventional Commit type. `fix` marks bug fixes; `docs` marks documentation. See [Commit conventions](commit-conventions.md). All repository documentation and commit messages are English.
