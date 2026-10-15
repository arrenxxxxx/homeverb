# Accounts, authentication, and credentials

## Unify the flow, not every login mechanism

HomeVerb itself has no account or sign-in requirement. Authentication applies only to third-party platform accounts and device pairing; see the [mobile app design](mobile-app.md).

State machine: `created -> challenge_required -> awaiting_user -> authenticated`. Any applicable stage may become `expired`, `cancelled`, or `failed`. Challenges may involve a browser, verification code, local pairing approval, or another vendor-approved interaction. Adapters declare only methods they actually support; universal QR login is not promised.

CLI uses trusted interactive input, not password arguments. The app uses trusted platform interactions. MCP defaults to `AUTH_REQUIRED` with user-side login guidance; the model must not collect passwords, one-time codes, or refresh tokens.

## Native-client admission

Prefer vendor-approved public-client flows, system browsers, and PKCE. A native application is not a confidential server capable of protecting a global shared secret. An integration requiring a confidential-client backend without a native alternative is `blocked`. [R3](references.md)

There is no background refresh worker. Check the session immediately before an actual operation and refresh only when necessary. Coordinate refresh-token rotation with a local account lock and reread any session another process has updated. Authentication failures must not create an endless login loop.

Cloud login may be needed only to retrieve a LAN pairing key. Therefore "local operation" does not necessarily mean "fully offline onboarding". Record `onboarding_cloud_required` and `operation_cloud_required` separately.

## Midea account connection and device retrieval

The first integration must support an existing Midea Meiju account's cloud device list, state, and permitted controls, including away from home. Retrieving LAN credentials alone does not satisfy this scope. Native login feasibility remains unverified; the [source guide](development/05-midea.md) separates community evidence from HomeVerb support.

Connect Platform previews the account/region access and automatic device retrieval. After successful authorization and secure persistence, the same user-initiated foreground flow retrieves accessible homes and bound appliances under a finite budget. Automatically add supported devices without per-device re-pairing or selection; report unsupported devices separately from controllable capabilities. No LAN access or local-network permission is required for this cloud operation.

Authorization success and enumeration success are separate results. On retrieval failure, retain a safely saved valid session and offer explicit retry; do not report an empty account or force another login. Reauthorization of the same account preserves local device IDs and organization. A different account/region must not silently replace established bindings. Cancellation or backgrounding stops retrieval, and reopening does not resume it automatically. Account-device retrieval is not periodic cloud synchronization.

## Credential storage

The business database stores `credential_ref`, not credentials. Implement the system secret-store abstraction separately for each operating system and verify it there. When secure storage is unavailable, do not fall back to plaintext JSON. The first release may reject persistence and offer a bounded interactive in-memory session or instructions to configure secure storage.

The Linux Secret Service library is implemented and tested with synthetic credentials; it is not wired to a real-platform account flow. It uses opaque profile/credential UUIDs, operation budgets, redacted secret buffers, and conservative mutation errors. See [storage](storage.md#native-credential-storage). Mobile backends, account persistence, and token-refresh coordination remain pending.

Exports contain configuration, aliases, and redacted metadata by default; do not include tokens in backup ZIPs. The owner-approved [foreground authorization-sharing flow](adr/0008-foreground-authorization-sharing.md) is a separate planned operation requiring peer verification, encrypted transfer, receiver consent, secure storage, and provider-specific eligibility evidence. Ordinary login support does not establish that a session is transferable. Redact logs, errors, and MCP output. Use local `account_id` selectors rather than repeatedly exposing emails or phone numbers.

## Logout

Delete local credentials, associated caches, and local authorization state. Revoke vendor tokens only when the platform supports it and the user authorizes that action. If revocation was unavailable or not attempted, explicitly report local deletion only; do not claim remote revocation.

## Tests

Denied consent; mismatched OAuth state; expired authorization codes; invalid verification codes; cancellation; rotated-token races; rejected device pairing; account-region mismatch; and unavailable secret storage. Identity failures must not silently retry with another account.
