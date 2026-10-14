# Feature abstraction and common capabilities

## Principle

Unify semantics and constraints, not merely method names. Device capability descriptions are the shared source for CLI argument validation, default app controls, MCP tool schemas, and contract tests. Entry points may present them differently without duplicating business rules.

## Capability families

| Capability | Example properties | Example actions | Scope |
|---|---|---|---|
| power | state | set(state) | P0; explicit on/off, not an implicit toggle |
| climate | current_temperature / target_temperature / mode | set(target_temperature, mode) | P0; device-specific ranges |
| fan | speed / preset / oscillation | set(...) | P1; distinguish numeric and enum semantics |
| light | brightness / color_temperature | set(...) | P1; explicit units and bounds |
| media | volume / muted / playback_state / source | set_volume / mute / play / pause | P1; expose actual capabilities |
| appliance_cycle | running / remaining_time / program | start / pause / stop | P2; review risk and duplicate execution |
| sensor | measured values | No writes | P1; missing values are not zero |
| vendor.<provider>.* | Vendor-specific fields | Reviewed actions | Extensible; do not disguise as standard semantics |

P0/P1/P2 are development priorities, not implemented support. A brand name does not determine capabilities.

## Capability description

A property describes its ID, type, readability, writability, unit, enum or min/max/step, nullability, and freshness requirements. An action describes its argument schema, supported routes, idempotency category, risk, cancellable phases, and verification method. An event describes its payload schema and support for bounded foreground observation. Do not assume event history is available for periods when the client was offline.

Ranges may depend on mode and need `conditional_constraints`. Reject an invalid combination instead of silently ignoring a field. `climate.set` does not implicitly turn on power; composite behavior is allowed only when the protocol supports it and the descriptor declares it.

## Common command example

```json
{
  "schema_version": "1.0",
  "request_id": "req-example-001",
  "device_id": "dev-example-ac",
  "component_id": "main",
  "capability": "climate",
  "action": "set",
  "arguments": {"target_temperature": 26, "mode": "cool"},
  "connection": "auto",
  "timeout_ms": 10000,
  "wait_for": "verified"
}
```

This is fictional. The generic Command Schema validates the envelope; the selected device's `action.parameters` must also validate `arguments`. Structural validity does not prove that a particular model accepts 26 degrees Celsius or `cool`.

## Idempotency and risk

Idempotency categories are `read_only`, `set_value`, `non_idempotent`, and `unknown`. A repeatable target value does not automatically authorize retrying every network failure. Protocol behavior and side effects still matter. Multi-field writes may require separate transmissions and are not automatically atomic.

Risk levels are `low`, `elevated`, and `high`. Generic MCP writes for sensitive heating, door-lock, or similar actions are excluded from the first release. Adding them requires trusted user approval and risk tests. Model-supplied `confirmed=true` is not approval.

## Extension review

A new standard capability needs equivalent semantics demonstrated across at least two heterogeneous adapters, with units, limits, errors, verification, and counterexamples. A single vendor's unusual mode belongs in its namespace first. Read-only support is valid; an unimplemented write must never return success.
