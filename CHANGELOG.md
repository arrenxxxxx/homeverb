# Changelog

## Unreleased

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
