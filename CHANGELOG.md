# Changelog

## Unreleased

### Open-source preparation

- Remove superseded visual studies and their galleries while retaining application
  assets and the self-contained 82-state prototype used by visual checks.
- Add support, third-party attribution and source-publication requirements; update
  security reporting and the existing-repository publishing procedure.
- Adopt Apache-2.0 following the owner's subsequent decision, with a root license,
  Rust/JavaScript package metadata and explicit contribution terms. Third-party
  material retains its original licenses and notices. Meiju publication scope,
  full-history review and private vulnerability intake remain release prerequisites.
- Preserve runtime behavior, public APIs, schemas and device compatibility. This
  preparation does not publish a repository, package or application release.

The earlier entries below describe individual development milestones and their
limits at that time. Use the [task status](docs/task-breakdown.md) and
[compatibility record](docs/compatibility.md) for the current integrated scope.

### Meiju account-bound Core reads

- Verify the protected local device and cloud connection before a default state request, retaining known model evidence and component identity.
- Exercise AC/E1/CA decoding through Core and SQLite with fictional read-only descriptors; preserve stale evidence and the previous cache on invalid responses.
- Require thread-safe AccountLock guards so leased reads satisfy Core's Send future contract; native API 1.0.0, SQLite schema 3 and live authentication admission remain unchanged.

### Meiju private account and device bindings

- Persist the provider-validated subject, opaque session and stable account-scoped appliance bindings in one protected vault payload.
- Preserve local IDs across retrieval and session replacement; reject identity conflicts and retain mappings missing from partial or empty lists.
- Return new bindings only after confirmed storage, keep uncertain updates pending, and refuse implicit resets of missing/corrupt protected state.
- Add synthetic SQLite/secret-store coverage and progressive integration guidance; native API 1.0.0 and live authentication admission are unchanged.

### Meiju cloud state decoding

- Add bounded, explicit cloud state queries and selected AC/E1/CA field decoding behind the injected read transport.
- Keep model-specific codes in vendor namespaces, separate refrigerator compartments, and omit missing values without defaults.
- Mark unproven cloud observations stale, preserve local receipt time, and redact private source state; no live authentication, controls or native UI methods are enabled.

### Meiju authentication admission review

- Verify the official developer-secret token flow and the separate community app-facing login requirements against primary sources.
- Record proposed ADR-0022 for a scoped owner decision on community signing material; preserve the current no-backend rule and disabled live authentication until admission is resolved.

### Persistent account vault

- Upgrade SQLite schema 1/2 to 3 with a verified backup, persisting random profile/account/credential references without session bytes or provider identifiers.
- Compose trusted native account storage with the OS secret store, and hold account leases across rereading and updating sessions.
- Leave uncertain or unfinished credential updates pending; recover through explicit validation of the exact stored bytes without repeating a secret write.
- Keep native UI API 1.0.0 unchanged. Provider login, public account methods, account removal and mobile secure stores remain pending; older schema-2 binaries refuse upgraded profiles.

### Meiju inventory protocol

- Add the native Midea cloud home/appliance inventory protocol with an injected read transport, account-scoped deduplication, unknown-family retention, partial errors, and shared cancellation/deadlines.
- Keep provider identifiers and response bodies out of serializable DTOs and Debug/error output; do not register live authentication or device control without the remaining evidence.

### Native UI contract

- Publish API version 1.0.0 with a local `get_api_info` query, 22 method schemas, actual Mock examples, and screen-to-interface coverage for independent Flutter work.
- Reject undeclared fields on no-argument native methods consistently with the existing closed-request policy.
- Verify the catalog through the built native library, including query/admitted-execution response differences, uncertainty, cancellation, and unavailable methods.

### Mock MVP

- Add the Rust domain/Core/Mock/storage/host workspace, JSON CLI, read-only MCP stdio, and native C ABI.
- Implement capability validation, scoped write policy, routing, deadlines, uncertain outcomes, journal recovery, and per-item composite receipts.
- Add local rooms, aliases, favorites, and ordering without changing device identity or control permissions.
- Add atomic batch room assignment and device-filtered operation history to Core, CLI, and native hosts, with matching en-US/zh-CN labels.
- Upgrade SQLite schema 1 to 2 with a verified backup and exclusive migration access; retain 100 finalized detailed receipts plus active records and a growing retired-ID ledger.
- Count validation/storage time toward the write budget and preserve verification and composite receipts when local persistence fails.
- Bind write grants to exact components, argument schemas, and transports; use the same whole-command and per-step preflight for planning and execution.
- Add asynchronous native credential contracts and a Linux Secret Service library with profile-scoped references, zeroizing buffers, bounded cleanup, and explicit uncertain mutation errors.
- Add bounded local diagnostic reports through Core, CLI, and the native bridge; omit private identifiers, arguments, state values, and raw messages, and normalize unreviewed error codes.
- Validate complete snapshots before caching or verifying writes; preserve acceptance and old cache on invalid evidence, use source observation age, and mark clock-shifted cached values stale.
- Validate live, cached, and verification values against component-specific readable property contracts, preserving device-specific ranges and read-only vendor capabilities.
- Update JSON Schema validation to 0.45.1 to accept valid negative fractional-step values while retaining device and permission bounds.
- Verify equal integer/float state values without type coercion, approximate matching, or loss of large-integer precision; preserve observed representations and request-ID conflict checks.
- Validate complete adapter descriptor batches before saving and revalidate stored metadata before operational use, preventing unsupported or ambiguous definitions from authorizing device calls.
- Require explicit independent-field declarations before splitting setter arguments; keep other multi-argument actions intact and reject contradictory descriptors before sends.
- Preserve device connection identities during rediscovery, preflight adapter responses, and reject conflicting SQLite updates transactionally without transferring existing grants.
- Add exact local receipt lookup through Core, CLI, native requests, and read-only MCP without replay, current write grants, or receipt mutation.
- Apply optional per-property observation freshness limits consistently to live/cache reads and verification, preserving the existing default and post-send evidence requirements.
- Verify bounded adapter-correlated completion evidence without state fabrication or write replay, preserve it across journal failures, and add a fictional completion scenario.
- Add explicit device batches through Core, CLI, and native requests with whole-list admission, bounded concurrency, shared cancellation/deadlines, and ordered per-device receipts.
- Resolve component-scoped capability definitions consistently for commands, observations, freshness, and verification; reject ambiguous shared/scoped mixtures while preserving legacy definitions.
- Support the full caller identifier limit in composite requests and device locks, preserving replay checks and legacy short-ID lock filenames.
- Add en-US/zh-CN presentation resources and progressive implementation guides.
- Remove the obsolete static checks report and duplicated FAQ; preserve normative references.
- Keep real-account integration, real-device support, mobile packaging, and non-Linux credential stores explicitly pending.

### Changed

- Adopt HomeVerb as the owner-approved project and app name, with `homeverb` as the repository and CLI identifier.
- Update all maintained command examples, MCP configuration, diagram labels, publishing targets, and related tests.
- Preserve prior English Git history and the no-daemon design boundary; no appliance-control feature is implemented by this rename.

### Added

- Record the naming decision and proposed module identifiers in ADR-0006.
- Add naming-consistency regression tests.

## 0.1.1-docs - 2026-10-14

### Changed

- Translate all baseline documentation, diagram labels, templates, and example messages into English.
- Require English Conventional Commits and provide a repository-wide language guard.
- Keep the no-daemon boundary: no NAS gateway, project backend, background automation, or remote HTTP MCP.

### Added

- Commit conventions, issue label definitions, a commit-message hook, and repository-tool tests.
- Private GitHub publishing instructions and an authenticated publishing helper.
- Local Git history with typed English commit messages in the distribution bundle.

These are documentation and repository-tooling changes, not an appliance-control release. Local validation does not imply remote publication or physical-device support.

## 0.1.0-docs - 2026-10-14

- Establish the no-daemon, no-project-backend design with three independent entry points.
- Add agent instructions and product, architecture, capabilities, authentication, routing, CLI, App, MCP, and testing documents.
- Add Mermaid/SVG/PNG diagrams and architecture decision records.
- Add JSON Schemas, fictional fixtures, counterexample validation, and repository templates.
- Keep all real adapters at candidate status; provide no control binary or verified driver.
