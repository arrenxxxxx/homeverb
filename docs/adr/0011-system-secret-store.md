# ADR-0011: Use bounded native secret-store operations

Status: accepted for the requested Core implementation. Date: October 14, 2026.

## Decision

Replace the initial synchronous read-only SecretStore placeholder with asynchronous get, put, and delete operations sharing a caller-supplied deadline and cancellation token. Secret references and profile namespaces are random UUIDs. Long-lived credential bytes are not serializable DTOs, have redacted Debug output, and use zeroizing owned buffers. This does not guarantee that upstream libraries or the operating system erase all transient copies.

Implement Linux Secret Service through the maintained `secret-service` crate, with Tokio and its Rust cryptography backend. Use an encrypted DH session over the user's local D-Bus connection. Each explicit operation opens its own connection and closes it afterward; construction and idle hosts perform no keyring I/O. The operating system's existing secret service is not a HomeVerb daemon.

Search only attributes matching application `homeverb`, a trusted profile UUID, and the exact credential UUID. Labels contain no account name, vendor identity, network address, or credential material. Use the default unlocked collection only; never enumerate unrelated items, fall back to plaintext files, create a replacement collection, or unlock the user's collection automatically. Duplicate matches fail closed. Real platform authentication and mobile secret-store backends remain separate work.

Credential mutations that time out or lose their reply after starting are reported as `SECRET_STORE_WRITE_UNKNOWN`, without automatic retry. Callers must inspect the exact reference before choosing recovery. A confirmed deletion is local deletion, not vendor revocation. Cancellation does not roll back a service mutation. No public CLI/MCP/native JSON method accepts or returns credential bytes.

Create/delete may invoke the service's own authorization prompt during the bounded explicit operation; HomeVerb never calls Unlock. After work stops, allow at most 100 ms of additional connection-cleanup waiting before dropping handles. Close the socket before waiting for remaining proxy references, preserving the operation's known result. Tokio may reclaim aborted reader tasks asynchronously. Prompted/locked-provider integration is not established by an unlocked-service test.

## Compatibility and verification

This changes the unreleased Rust SecretStore trait and return type. Update the unavailable provider and callers together. Command/result wire contracts and the C ABI remain unchanged. Profile namespaces must be persisted as nonsecret metadata by the future account host; changing a namespace does not migrate stored credentials.

Verify temporary synthetic credentials on the actual local service: create, get across separate store instances, replace, profile isolation, delete, and absence after deletion. Tests use only newly generated references and remove their own entries. Default automated tests must not access a developer's keyring; desktop integration is an explicit ignored test. Missing/locked service behavior, cancellation, and redacted errors must fail closed. Do not claim Android/iOS or real-account verification from this probe.
