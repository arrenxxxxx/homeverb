# 1. Run and verify the Mock MVP

The Mock appliance is a persistent simulation, not a network device. No real account, credential, or appliance is needed. Use Linux for the currently verified CLI/native acceptance path.

After trying the flows below, run `./target/debug/homeverb --mock diagnostics` to preview a bounded local report. It summarizes stored counts and outcomes without device identifiers, names, argument/state values, or raw messages. Review it before saving or sharing. It does not contact devices or upload the report; see [diagnostic limits](../security-and-privacy.md#local-diagnostic-reports).

## Build and operate

```bash
cargo build --workspace --locked
./target/debug/homeverb --mock --profile demo device discover
./target/debug/homeverb --mock --profile demo device list
./target/debug/homeverb --mock --profile demo device describe mock-ac
./target/debug/homeverb --mock --profile demo power set mock-ac --on
./target/debug/homeverb --mock --profile demo climate set mock-ac --temperature 25 --mode cool
./target/debug/homeverb --mock --profile demo device state mock-ac --fresh
./target/debug/homeverb --mock --profile demo executions
```

Discovery is explicit. The revision-2 Mock climate capability declares `independent_fields: true` for per-field execution. Rediscover an older profile to obtain this declaration; earlier persisted descriptors keep their arguments together. Existing request IDs still return their saved receipts. `device list` does not discover or refresh. The fictional climate capability permits 16-30 Celsius in 0.5 steps; these are Mock-specific values. Unknown device state is never converted to false or zero.

`--data-dir` selects an application directory; the profile is appended to it. The default is `$XDG_DATA_HOME/homeverb` or `$HOME/.local/share/homeverb`. Do not put runtime databases in the repository.

## Exercise failures

```bash
./target/debug/homeverb --mock --profile demo --dry-run power set mock-ac --off
./target/debug/homeverb --mock --profile demo --scenario lost-ack power set mock-ac --off
./target/debug/homeverb --mock --profile demo --scenario lan-offline --connection lan power set mock-ac --on
./target/debug/homeverb --mock --profile demo --scenario slow-after-send --timeout-ms 100 power set mock-ac --on
./target/debug/homeverb --mock --profile demo --scenario partial climate set mock-ac --mode heat --temperature 24
```

Expected exit codes: dry run 0, lost ACK 11, unavailable LAN 5, timeout after send 11, mixed composite 13. Scenarios also include `normal`, `reject`, `accept-only`, `offline`, `slow-before-send`, and `stale-read`. `accept-only` can still verify if fresh observed state already matches; matching does not establish causality.

Use `--request-id` to correlate a write. Reusing an identical command and ID returns the recorded outcome without transmitting. Changing its arguments with the same ID fails with `REQUEST_CONFLICT`. Do not generate a new ID automatically after uncertainty.

## Inspect an uncertain command

```bash
./target/debug/homeverb --mock --profile demo --scenario lost-ack --request-id inspect-demo power set mock-ac --on
./target/debug/homeverb --mock --profile demo execution inspect-demo
```

The write exits 11 with `unknown`. The lookup exits 0 because the query succeeded; its `data.status` remains `unknown`. A fresh state read is separate evidence and does not rewrite the receipt. You can also look up a composite child ID. Missing or retired receipts do not establish non-execution and must not trigger automatic resending.

## Exercise correlated completion

```bash
./target/debug/homeverb --mock --profile completion-demo --scenario correlated-completion device discover
./target/debug/homeverb --mock --profile completion-demo --scenario correlated-completion power set mock-ac --on
./target/debug/homeverb --mock --profile completion-demo --scenario correlated-completion climate set mock-ac --mode heat --temperature 24
```

These return `verified` with fictional `correlated_completion` evidence. Discovery declares that method; use an isolated profile so normal demos retain state matching. Completion does not refresh the state cache. An ordinary write without the completion scenario in this profile returns `accepted` with `VERIFICATION_PENDING`; an acknowledgement is insufficient even if the state changed. Rediscover without the scenario to restore normal descriptors. See [ADR-0016](../adr/0016-correlated-completion.md).

## Execute an explicit device batch

Save this nonsecret example in a local file such as `/tmp/homeverb-batch.json` after discovering the Mock device in the `demo` profile:

```json
{
  "schema_version": "1.0",
  "commands": [{
    "schema_version": "1.0",
    "request_id": "demo-batch-power-1",
    "device_id": "dev-mock-ac",
    "component_id": "main",
    "capability": "power",
    "action": "set",
    "arguments": {"state": true},
    "connection": "auto",
    "timeout_ms": 10000,
    "wait_for": "verified"
  }]
}
```

```bash
./target/debug/homeverb --mock --profile demo --dry-run batch --file /tmp/homeverb-batch.json
./target/debug/homeverb --mock --profile demo batch --file /tmp/homeverb-batch.json
./target/debug/homeverb --mock --profile demo execution demo-batch-power-1
```

The batch returns ordered `items`; this simulator has only one device. Dedicated integration tests exercise multiple devices and the four-device concurrency limit. Every selected device needs its own exact ID, request ID, and trusted grant. Reusing a recorded command returns its receipt, but a batch is not a durable job: after interruption, inspect individual IDs without automatically resubmitting missing items.

## Verify

Run the commands in [Testing](../testing.md). `scripts/verify_mvp.py` builds no artifacts itself: it exercises the compiled CLI, local MCP, and Linux native library in temporary directories. Its SIGINT test requires normal local signal delivery; a restrictive agent sandbox may need an approved test run outside that sandbox.

Next: [Work on Core](02-core.md).
