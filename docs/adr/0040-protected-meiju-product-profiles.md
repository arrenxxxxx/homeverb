# ADR-0040: Preserve protected Meiju product mapping evidence

Status: accepted for offline implementation; no new physical capability is admitted.

## Evidence and decision

The pinned Midea Auto Cloud revision
`ce0c4b88224def0f6b53714312c19f71aa9c2a50` reads `sn8`, `modelNumber` and `category`
from the existing [home/appliance inventory](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/core/cloud.py). Its device constructor passes
`modelNumber` as the subtype, and [get_device_mapping](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/__init__.py) selects by subtype, product
code, category and finally a family default. AC temperature bounds and dishwasher
program encodings vary between those mappings. A display model alone cannot
identify that mapping. HomeVerb currently discards this evidence.

Retain those three optional fields in private inventory and protected account
bindings. Do not collect or derive a product code from the full serial number,
download Lua/plugins, add provider calls or infer a temperature/program contract.
An absent/null/empty field remains unknown. The upstream zero product-code and
subtype placeholders also remain unknown. Product codes are eight ASCII letters
or digits; subtype values are nonnegative decimal integers, normalized across
numeric/string representations; category is bounded non-control text.
Malformed known fields make that home's response invalid. Conflicting nonempty
fields for one appliance fail identity reconciliation; missing later evidence
never erases known evidence. Preflight the entire inventory before saving any
new profile or local binding.

## Storage and compatibility

Protected account payload version 3 adds optional `product_profile` to each
binding. Decode versions 1 and 2 without writing or fabricating profile evidence;
only a later explicit protected update writes version 3. Preserve local device,
connection and identity IDs, membership, preferences and receipts. Reauthorization
and recovery preserve the evidence. Old binaries refuse version 3 and must not be
used to reopen upgraded accounts. SQLite remains schema 4.

The source profile type is redacted and not serializable through public DTOs.
Only the private protected-payload representation implements serialization. No
provider code, subtype or category enters device descriptors, native/CLI/MCP
results, diagnostics, ordinary SQLite fields or logs. Native API 1.11.1 identifies this storage upgrade; all 31 methods, seven C functions
and existing controls retain their contracts. This unreleased Rust inventory type gains
one field; external literal constructors must initialize it with an unknown
profile when they have no evidence.

## Verification

Use fictional inventory and secret storage for missing/placeholders, valid
representations, malformed values, cross-home and saved-binding conflicts,
whole-update rollback, retained identities after reopen/relogin, legacy migration
and protected-store failures. Assert no profile markers appear in SQLite or public
host results and no profile update grants a capability or sends a control. No
owner-account access or appliance operation is part of this phase.
