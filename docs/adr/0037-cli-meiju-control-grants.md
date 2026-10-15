# ADR-0037: Admit exact Meiju CLI controls through explicit startup grants

Status: accepted for implementation; physical acceptance remains deferred.

## Decision

Extend the existing trusted native power/fan grant model to the Linux CLI through
an explicitly supplied `--meiju-grants` file. Do not automatically load permissions
from the profile, environment, command JSON or device metadata. Default Meiju
startup stays read-only. MCP cannot select this adapter or acquire these grants.

The file has a closed, versioned schema and the same independent exact-device/value
lists as native startup. It can admit only the already reviewed low-risk AC power
states and vendor fan codes over cloud. It cannot grant arbitrary actions, elevated
risk, wildcard devices or provider identities. The caller must own the regular
file, with no group/other write permission; reject relative paths, final symlinks,
nonregular files and content above 32 KiB. Validate the complete file before opening
a profile. This protects against other OS users, not a compromised owning process.

The appliance owner reviews each local device/value before creating the file.
Supplying it is trusted host configuration, not approval inferred from a command
parameter such as `confirmed=true`. The coding agent still requires separate owner
authorization before any real send. Implementation tests use fictional identities.

Pass these grants into the existing Core/adapter composition for discovery, account
retrieval, planning and execution. Do not create another execution path or secretly
refresh descriptors before a write. An explicit discovery/refresh with the same
grants exposes admitted actions. Local dry runs do not check protected membership,
cloud reachability or physical state. Live execution repeats those existing guards.

Require an explicit request ID for real-adapter single writes and previews. Batch
files retain their explicit per-command IDs and whole-list preflight. Permission
files grant no replay exception: accepted/unknown requests retain their receipts,
changed commands conflict and retired IDs remain refused. Removing permissions
does not erase receipts; use receipt lookup without grants after interruption.

## Compatibility and verification

This adds a CLI startup option and a Rust composition helper. Existing Mock commands,
native API 1.10.0, C functions, capabilities, execution outcomes, SQLite schema 4
and protected account payload version 2 are unchanged. No new appliance command,
automatic retry, polling, login or verification read is introduced.

Verify schema/native-grant parity, bounded/secure file admission, permission
separation, profile creation refusal, explicit request IDs, actual CLI local plans,
pre-send failures and receipts using fictional profiles. Existing synthetic native
tests cover acknowledgements, uncertainty and cancellation. Neither these tests nor
a locally valid grant file establish physical-model or off-LAN support.
