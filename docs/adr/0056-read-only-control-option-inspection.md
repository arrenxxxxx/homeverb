# ADR-0056: Inspect control options before granting authority

Status: accepted for synthetic implementation within the existing Meiju scope.

## Problem

A default native Meiju session is read-only. Its descriptors omit actions that
require trusted startup grants, while exact grants require arguments selected
from those actions. Flutter cannot construct a model-specific permission form
without either guessing parameters or enabling control before user consent.

## Decision

Native API 1.16 adds `inspect_control_options` with exact local `device_id` and
`connection_id`. This explicit foreground query opens only the selected saved
account under its lease. Validate the protected connection, listed membership
and model against the Core descriptor before projecting reviewed action
definitions. Select temperature and dishwasher options from protected product
evidence using the same selectors and action constructors as execution.

Return local device/connection IDs, `source: saved_inventory`,
`authorization: not_checked` and an `options` array. Each option contains
`component_id`, `capability`, `action` and its `definition` (the existing Action
contract). This is permission-form metadata, not a replacement descriptor,
grant, approval, live state or proof of physical support. Missing reviewed
controls return an empty array; unavailable credentials, unknown membership,
pending accounts and identity conflicts return errors rather than empty success.
Refrigerators remain without write options. No product codes, provider device
IDs, credentials or session material leave the adapter.

The query sends no provider or appliance requests, mutates no credentials,
descriptors, cache, journal or session policy, and does not enable adapter control
flags. Ordinary plan/execute remain denied in a read-only session. After explicit
user consent, the trusted controller constructs the existing exact startup
grants, closes/reopens safely and explicitly refreshes descriptors. Execution
revalidates current protected evidence and policy. Elevated actions still require
separate per-invocation trusted approval; inspection never supplies it.

The method is available through the existing C call export. Mock returns
`AUTH_FLOW_UNSUPPORTED`; fictional response fixtures support UI development.
CLI/MCP do not gain a method or authority. JSON schema 1.0, the ten public C
functions, Android JNI initialization and storage versions are unchanged. Older
native versions reject the new method; negotiate API 1.16 and its advertised
method before use. Existing method response shapes remain compatible.

## Verification

Exercise Session, real SQLite and the account vault with fictional credentials:
exact protected model/profile selection, descriptor parity, empty unsupported
results, identity/membership/credential failures, cancellation and lease cleanup,
closed request/response fields and absence of network calls or mutations. Prove
that inspection cannot make ordinary or elevated actions executable. Run native
C contract checks and Android library builds; actual Flutter consent, phones and
physical/off-LAN appliance acceptance remain separate.
