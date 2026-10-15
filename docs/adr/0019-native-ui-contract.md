# ADR-0019: Publish a versioned native UI contract

Status: accepted for the unreleased Flutter handoff.

## Decision

Keep Flutter -> in-process C ABI -> Session -> Core. Publish offline JSON Schemas for native startup, requests, and method-specific responses, and a method catalog bundled into the host. Add the local `get_api_info` request so an independently developed UI can inspect the contract version and available methods before enabling features. The catalog describes callable APIs, not device permissions or platform compatibility.

Contract version `1.0.0` covers the existing JSON wire shapes and this additive query. The envelope's `schema_version` remains `1.0`. Compatible additions increment the contract minor version; incompatible wire changes require a major version and a new ADR. Clients require a supported major version, check method availability, and retain unknown optional response fields. Requests remain closed objects. An older host without `get_api_info` returns `INVALID_ARGUMENT`; show an upgrade-required state instead of guessing which mutations exist.

Preserve the existing response distinction: queries return `{schema_version,data,warnings,error}`; admitted `execute` and `execute_batch` return execution results directly. Either execution method can return a query error before admission. A successful query with null data, such as room deletion, is not an error. A fulfilled FFI call is not evidence that an appliance executed a command. Operation IDs identify active calls; command request IDs identify durable write receipts.

The public contract does not expose credentials, grants, principal selection, arbitrary paths, or a `confirmed` override. Native startup remains trusted and explicitly Mock-only in this release. Unimplemented account, sharing, settings-persistence, cleanup, and identity-merge methods are absent from the callable catalog. Their UI states and required future contracts are documented without inventing executable success responses. Frontend-only preview fixtures must be isolated from a production session.

Flutter owns private nonsensitive UI preferences such as skin, locale, sharing display name, and display route choices. Resolve device > installation > Auto before submitting an explicit read/command route; Core still validates permissions and capability-supported routes. Core owns room/card metadata, so these UI preferences must not expand the existing DevicePreferences payload or bypass its storage rules. No native theme service is needed.

## Compatibility and verification

No existing JSON payload, C symbol, database schema, or execution outcome changes. No-field request variants must reject extra fields just like requests with arguments; accepting previously ignored fields is not part of the supported contract. The unreleased Rust enum may use empty struct variants to enforce that existing rule.

Test every catalog method through the built native library against its declared response schema, including null success, admission errors, composite/batch outcomes, cancellation, background refusal, and closed handles. Use only fictional profiles. Schema references resolve from an offline registry; validation must not fetch schemas from a network. Phone packaging, Flutter bindings, real account authorization, and physical-device behavior need separate evidence.
