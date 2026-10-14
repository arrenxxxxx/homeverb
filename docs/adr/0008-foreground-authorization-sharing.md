# ADR-0008: Allow explicit foreground authorization sharing on a local network

Status: accepted as a product boundary; implementation pending. Date: October 14, 2026.

## Context

The owner requested a Flutter app without a HomeVerb login and the ability to share third-party platform authorization with another person using a LocalSend-style nearby transfer. Sharing belongs inside Settings rather than a separate primary page. The previous authentication baseline excluded credential migration from the first release.

## Decision

Permit user-initiated, time-bounded authorization sharing between foreground HomeVerb apps on a mutually reachable local network. This supersedes the blanket first-release exclusion of credential migration only for this reviewed flow. Ordinary backups and diagnostic exports still exclude credentials.

The receiver explicitly enters receive mode. The sender selects an eligible platform account and a discovered receiver. Both sides verify a pairing code bound to the authenticated encrypted session and consent to the actual scope before sensitive material is sent. A discovered name or address alone is not peer authentication. LocalSend is an interaction reference; protocol interoperability is not part of this decision.

Discovery and transfer may use temporary local-network sockets/listeners owned by the foreground operation. Limit them to the selected local network and the authorization-transfer protocol. Close them on completion, cancellation, deadline, or backgrounding. Do not expose device-control APIs, remote HTTP MCP, public internet endpoints, background receivers, relays, or automatic cross-device synchronization. Existing authentication callbacks remain loopback-only.

Core and platform services own transfer validation, cancellation, and secret-store access; adapters own provider-specific eligibility and authorization semantics. Flutter presents the flow without retaining unnecessary secret copies. This document does not establish a new public transfer API or wire contract.

## Provider and credential requirements

- Prefer provider-issued member access or separate grants. Transfer an existing session only with evidence that the provider permits it and it can operate safely on the receiving installation.
- Reject device-bound sessions and unsafe credential reuse, including refresh-token rotation that cannot safely support the intended installations. Never extract another app's private credentials or bypass provider authentication.
- Display actual scope, known expiry, and revocation support. Room assignments or a device-selection list cannot restrict a full-account credential. Do not invent recipient-specific revocation or enforceable expiry.
- Require consent on both devices and receiver secure storage before import. Do not write plaintext transfer files, ordinary SQLite fields, logs, clipboard content, or backup ZIPs. Resolve existing-account conflicts explicitly.
- Separate transfer delivery, secure import, and a successful platform authorization check. A lost final acknowledgement is an unconfirmed result, not permission to resend automatically. Interrupted flows do not resume on startup.
- The receiver subsequently connects directly to the provider or appliance. Local deletion does not revoke another installation's access. Revocation requires provider support and a verified result.

## Consequences and implementation gates

Sharing eligibility is independent of ordinary login support. All real platforms remain unverified for sharing until provider-specific authorization, device-binding, refresh-token, permission, and revocation behavior has evidence. Unsupported platforms guide recipients to authorize their own accounts.

Before implementation, define and review the transfer contract, authenticated key establishment, peer verification, replay resistance, payload validation, and bounded network behavior. Do not create custom cryptography. Add fictional positive/negative fixtures and platform lifecycle tests before enabling real credential transfer. This design approval does not authorize sending live credentials or performing real-device writes.

The [mobile screen inventory](../mobile-app.md) and [planned sharing task](../task-breakdown.md) describe the UI and acceptance criteria. This ADR changes documented scope; it does not establish a working feature or real-platform compatibility.

## Change control

Adding remote reachability, persistent listeners, unattended receiving, account relays, or cloud synchronization requires another explicit owner decision and a new ADR.
