# ADR-0024: Persist Meiju device bindings with the protected account session

Status: accepted for offline native implementation. ADR-0022 remains proposed.

## Decision

Store the Meiju session, a provider-validated opaque account subject, and appliance-to-local-ID bindings in one versioned payload under the account vault's existing secret reference. Use the system secret store, never plaintext SQLite or a separate credential file. Bind the payload to the immutable local account ID, credential reference, provider and region. Keep the account lease across reads and updates; do not cache session bytes outside the operation.

The future admitted authentication implementation supplies the subject and session only after validating their association with the provider account. This storage layer does not define a login response field or prove authentication. Reauthorization must match the exact stored subject; a different subject cannot replace the existing account or inherit its device permissions. Creation reserves a new account through the vault; an existing lease cannot initialize or reset its map. Recovery continues to require native validation and exact-byte confirmation under ADR-0021.

Allocate independent random local device, cloud-connection and identity-reference UUIDs for each previously unseen appliance ID within that account. Persist them before returning them to a descriptor builder. Repeated retrieval keeps all three IDs. Preserve known family/model evidence and reject conflicting evidence before any secure-store mutation. Missing models may become known; missing appliances in either complete or partial inventories never delete mappings. Names, online hints and vendor rooms do not determine identity. Unrecognized families may have identities without gaining capabilities.

Any failed or uncertain update returns no new bindings. The vault's pending state prevents reuse until explicit recovery; never regenerate IDs by resetting unreadable, missing or uncertain state. Known mappings can be reused without a secret-store write. Bound input sizes, validate stored structure and local UUID uniqueness, redact Debug/errors, and keep public native JSON free of provider identifiers and session bytes.

## Limits and verification

This is account-scoped identity persistence, not Core descriptor admission, cross-account deduplication, route merging, authorization, or physical identity proof. A future discovery coordinator must resolve potential cross-account duplicates before publishing cards. Losing the protected entry requires explicit recovery; it must not silently create a replacement account map. The existing Core still validates established connection bindings and preserves local preferences.

Synthetic integration tests use SQLite and an injected secret store to verify reopen, reauthorization, partial inventory, conflict preflight, private-data boundaries, uncertainty, cancellation and local organization preservation. No real keyring or account is accessed. SQLite schema 3, native API 1.0.0, C ABI and registered adapters are unchanged. Live authentication and physical-device acceptance remain pending.
