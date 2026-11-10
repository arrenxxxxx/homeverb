# ADR-0076: Protected credentials and bounded account reauthentication

Status: Accepted by the owner on November 10, 2026.

## Context

The owner requests remembered platform passwords and automatic sign-in after
authorization expires. Existing Meiju sessions retain tokens only. Passwords must
never enter ordinary JSON, SQLite, preferences, diagnostics or logs.

## Decision

Successful native Meiju sign-in saves its validated login inputs in the existing
system secret-store account payload. Protected payload version 4 adds an optional
zeroized credential record and durable automatic-login attempt state. Versions
1–3 remain readable; accounts without saved inputs require one explicit sign-in.
Older binaries refuse version 4 instead of discarding its identity or credentials.
Account removal deletes the same protected record. Public JSON and C ABI stay
unchanged; no credential getter or plaintext fallback is introduced.

Only a definite `AUTH_REQUIRED` from an account inventory or device state read
may cause one automatic login and one retry of that read within the original
operation deadline and cancellation scope. The account lease serializes recovery.
Save a disabled-attempt marker before login; any login, identity, persistence or
cancellation failure requires explicit sign-in before another automatic attempt.
Successful recovery retains a five-minute cooldown, including across restart;
clock rollback also blocks attempts until the cooldown passes.

The returned subject must match the protected account identity. Reuse existing
bindings, membership, preferences and grants; do not merge or migrate identities
automatically. Challenges and rejected passwords are not bypassed. No inventory
refresh is added to device reads. No timer, background login, CLI/MCP idle work,
automatic command retry or uncertain-write replay is introduced. Command paths
retain their original behavior; foreground reads can repair a session for a later
explicit command.

## Verification

Fictional transports and secret stores must cover credential persistence,
same-subject recovery, identity refusal, missing credentials, attempt blocking,
cooldown, cancellation, storage failure and unchanged bindings. Real account and
phone acceptance must be recorded separately from these checks.
