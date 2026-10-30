# ADR-0099: Mijia directory device groups

Status: Accepted; implementation in progress.
Date: October 30, 2026.

## Evidence and decision

The pinned October 22 reference archive's `scripts/build_report.py` identifies
`pid=21` as `logical_split_child` and reads cloud `parent_id` independently of
cached `parentId`. Its public model ledger contains ordinary kinds 0, 6, 16 and
18, plus two kind-21 logical entries for `opple.switch.0010d6`. Kind-18 switches
also have parents but are independent devices. A parent field alone must therefore
not collapse devices. The pinned Apache-2.0 HA source also normalizes numeric/text
`pid` and treats kind 21 specially; it is corroborating evidence, not a dependency.
Do not infer relationships from names, ID prefixes/suffixes or similar models.

Extend protected modern directory entries with bounded, canonical numeric/text
kind and optional parent identity. An empty parent string is absent. Malformed
kinds, malformed nonempty parent fields and non-string identities fail parsing.
Neither field is exposed in ordinary JSON, logs or plaintext storage. Missing
kind evidence remains representable for compatibility but cannot establish a
physical candidate. Unknown kinds are retained unresolved, not guessed.

A consuming typed directory reconciliation groups each admitted ordinary kind
(0, 6, 16 or 18) as its own candidate, retaining the exact original entry. Kind 21
can join only its explicit, present ordinary parent with equal model and no
contradictory reported spec. It never creates another physical card. Distinct
kind-18 children of a gateway remain distinct groups. Missing/unknown parents and
nested kind-21 parents remain unresolved with nonsecret reason enums. Conflicting
models/specs, duplicate IDs, mixed authenticated subjects and cyclic parent graphs
fail the whole reconciliation before any result is published. Grouping never
rewrites property IDs, fuses capability mappings, changes routing or authorizes a
write. Each retained entry keeps its independently validated property mapping.

Use an inherited bounded Operation for cancellation/deadline checks and linear
graph traversal, with the existing 4096-entry bound. Retain every input entry in
exactly one group or unresolved record and preserve original directory issues.
Unresolved records add a bounded generic partial-inventory issue. The inventory
remains incomplete even if every entry is grouped: this does not prove full
membership, physical existence, reachability or permission to remove old cards.

This phase adds typed protocol evidence and grouping, without changing saved
payload versions, Core descriptors, native API 1.36 or App report shape. Protected
route provenance, stable cross-protocol binding reconciliation, Core publication
and modern Core reads are the next dependent implementation. Do not claim these
features from a passing grouping test. Actual provider/physical acceptance remains
open; no account credentials, source observations or live IDs are test fixtures.

## Required checks

Numeric/text kind equivalence; kind-18 gateway relationships; multiple kind-21
children; missing/unknown parents and kinds; nested logical parents; cycles;
conflicting model/spec; foreign subject/cUserId; duplicates across pages; order
independence; a complete partition at the 4096-entry bound; cancellation/deadline;
redacted debug and serialization refusal; no network or inferred property rewrite
in reconciliation; existing modern account and legacy protocol tests.
