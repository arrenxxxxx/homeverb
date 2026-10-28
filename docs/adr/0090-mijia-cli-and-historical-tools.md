# ADR-0090: Mijia CLI hosting and bounded historical tools

Status: implemented with fictional Rust and process tests; real-provider acceptance pending.
Date: October 29, 2026.

## Decision

Add explicit `--adapter xiaomi.mijia` to the local CLI/MCP host. Its default
profile is `mijia`, separate from Mock and Meiju. Startup composes the existing
read-only native Mijia Core without logging in, opening credentials, discovering
or refreshing accounts. Meiju grant files and all appliance writes are refused
for this adapter before opening the profile. Existing Meiju grant behavior is
unchanged. A local shared profile does not authorize another platform adapter.

CLI account commands reuse the native Mijia Session for list/details, protected
inventory inspection, refresh/recovery, rename/removal and one-shot
terminal connection. Credentials use existing hidden interactive input and owned
zeroizing buffers, never command arguments or ordinary JSON. A challenge is an
explicit failure, not an automatic message request, browser launch or retry;
interactive challenge continuation remains the trusted App flow. Partial inventory
stays partial, and generic discovery cannot claim its completeness. No account
operation enters MCP, and idle MCP never refreshes an account. Saved cross-account
conflict inspection retains the native Mijia unsupported result.

Expose ADR-0089 through `history options DEVICE_ID --connection-id ID` and
`history read --file PATH`. The UTF-8 JSON file is at most 16 KiB and contains the
existing closed HistoryRequest DTO: exact local IDs, inspected target, aggregation
and canonical UTC interval. No global route/wait/request-ID/dry-run overrides are
accepted for these queries. Reads inherit the command deadline and cancellation,
return the complete query envelope, release resources and exit.

MCP gains `history_options` (exact local device/connection IDs) and `history_read`
(one closed `history` object). Both are read-only tools available without control
grants, using the existing ten-second operation budget, request cancellation,
EOF/host shutdown and bounded concurrency. Tool metadata states the distinction
between local inspection and explicit external reading; a result never becomes
current state or command verification. Six tools are available by default, eight
with existing admitted Meiju controls. No caller identity, credential, provider
key, endpoint, record limit or privilege override is accepted.

Core and the protected adapter independently validate identity, model, route,
series and result. Outputs preserve timestamps, units, zero values, rejected
counts, limit evidence and unproven completeness. No retries, fallback, historical
polling, persistence, gap filling or calendar totals are added. Mock and Meiju
continue to return empty history options. Native API 1.33, C exports, storage,
protected payloads and physical compatibility remain unchanged.

## Verification

Required evidence includes real CLI process schema/exit tests in fictional local
profiles, empty Mijia startup, grant/write refusal before profile creation, hidden
input refusal without a terminal, exact history arguments and malformed/oversized
file refusal. MCP tests must cover catalog/schema consistency, a successful
fictional historical adapter, cancellation during a read, no automatic reads,
unchanged receipts/cache and denied forged fields. No provider/account/appliance
operation is authorized by these development tests.
