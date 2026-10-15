# ADR-0015: Apply declared freshness limits per property

Status: accepted for the unreleased Core MVP.

## Decision

Add optional `max_age_ms` to property descriptors. It is a positive integer number of milliseconds, bounded to JSON's interoperable exact integer range (1 through 9007199254740991). Integer-valued numeric representations such as `1000.0` are equivalent. Omission retains the existing 30000 ms Mock default; zero does not disable freshness checks.

Live reads, cached reads, and state-match verification use the same validated property metadata. An observation becomes stale when its age is greater than or equal to its property's limit. Use trustworthy `observed_at` when present and otherwise `received_at`; receiving old source data again does not reset its age. An existing stale flag or a cached future receipt remains stale regardless of the declared limit.

Write verification additionally requires target observations and receipt times at or after transmission. A relaxed freshness limit cannot make a pre-send observation verify a write. Invalid freshness metadata fails descriptor admission before adapter calls. An accepted write whose evidence expires remains `accepted` with `VERIFICATION_PENDING`, without resending.

## Compatibility and limits

Older descriptors retain their 30-second behavior. Existing property and state fields, native requests, C ABI functions, and SQLite schema are unchanged. Earlier strict capability validators must accept the optional property field before consuming descriptors that include it. Warning text now refers to a property freshness limit instead of assuming every limit is 30 seconds.

Cache reads apply current capability metadata without rewriting stored observations. Existing stored stale flags are not cleared by increasing a limit. No timer, polling worker, or subscription is introduced. Limits do not guarantee physical accuracy; real adapters must justify them with protocol and model-specific evidence. The default is a compatibility baseline, not a universal appliance promise.

## Verification

Deterministic-clock tests cover shorter and longer limits, equality at expiry, omitted defaults, integer-valued floats, retained stale flags, old-source timestamps, post-send verification, and preserved cache. Contract counterexamples reject zero, negative, fractional, oversized, boolean, string, and null limits. Runtime acceptance exercises mixed limits through the CLI.
