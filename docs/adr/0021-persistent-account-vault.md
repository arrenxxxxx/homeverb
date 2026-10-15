# ADR-0021: Persist account references and coordinate native session updates

Status: accepted for the real-account foundation.

## Decision

Upgrade SQLite to schema 3. Persist one random profile namespace and account records containing a random local account ID, provider, region, local label, random credential reference, and credential-storage state. Do not store provider account identifiers, passwords, tokens, or session bytes in SQLite. Existing device, organization and execution records retain their meaning. Upgrades from schema 1 or 2 use the existing exclusive migration lease and verified backup procedure.

Core exposes a trusted native account vault, separate from the device Store API. Acquiring an account lease takes a profile-local process lock and then rereads its metadata. Keep that lease while reading the current secure-store entry, making a future provider refresh request, and saving its replacement. Release the lease on drop; no database transaction spans network or secret-store I/O. The host composes the OS secret store with the persisted profile namespace. Construction does not access a keyring or authenticate an account.

An account starts with `pending` credential state. Before each credential replacement, persist `pending`; after a confirmed secure-store write, persist `stored`. Once pending is saved, a crash, cancellation, uncertain secret mutation, or failed final metadata update before finalization leaves `pending`, so ordinary session reads fail closed with `ACCOUNT_SESSION_UNCONFIRMED`. Trusted recovery may inspect the referenced secret but must establish provider validity before confirming those exact bytes. Confirmation rereads and compares the entry without another secret write. There is no automatic retry, rollback, or login. `stored` means locally saved credentials, not current provider authentication or physical-device support.

Account IDs, provider/region bindings and credential references are immutable. Rename changes only the local label. Reauthorization and account identity matching remain provider responsibilities; an arbitrary UI request cannot replace a session or select credentials. The later admitted Meiju connection coordinator additionally holds a profile registration lock using the existing persisted namespace and checks secure-store access before explicit login. This adds no SQLite migration. No public JSON credential mutation or removal method is added in this phase. Local account removal and remote revocation need a later scoped flow that accounts for linked devices and uncertain operations.

## Compatibility and evidence

Schema-2 clients refuse schema 3; close all older clients before migration. C ABI and native UI API 1.0.0 stay unchanged. No real account is manufactured in the Mock host, and no credential is accessed merely by starting it.

Tests use real SQLite and synthetic secret stores to verify restart persistence, profile/account isolation, lock contention and rereading, pending/confirmed transitions, unavailable/uncertain storage, cancellation, and metadata failures. Database contents must exclude private session markers. Existing schema-1 data and schema-2 organization survive migration. Live provider login, token rotation behavior, OS account-flow acceptance and mobile secure stores remain unverified.
