# Test plan and quality gates

## Checks available now

```bash
python3 scripts/validate_docs.py
python3 -m pip install -r requirements-dev.txt
python3 scripts/validate_contracts.py
python3 scripts/validate_repository.py
python3 scripts/validate_commits.py --all
python3 -m unittest discover -s tests -v
```

These cover documentation links, JSON syntax, schema fixtures/counterexamples, repository language guards, commit syntax, and repository-tool tests. They are not an appliance simulator, a control-CLI test, a mobile build, or physical-device verification.

## Future test layers

| Layer | Focus | Dependencies |
|---|---|---|
| Domain unit tests | Bounds, units, capability mapping, transitions | No network |
| Adapter contract tests | Identity, acknowledgement classification, budgets/cancellation | Mock transport |
| Core fault injection | Pre-send failure, lost ACK, rate limits, refresh races | Controlled clock/network |
| Entry-point parity | Same input has the same semantics in CLI/App/MCP | Mock Core/Adapter |
| Lifecycle | No orphaned work, polling, or persistent listeners | Process/network observation |
| Physical-device tests | Authentication, reads, authorized writes, independent verification | Owner-authorized hardware |

## Mandatory cases

1. Duplicate discovery merges only strong identities; weak matches require confirmation.
2. Cached temperature 25, desired 26, and an HTTP acknowledgement produce `accepted`, not `verified`.
3. A cleaning start followed by disconnect produces `unknown`, without cross-route replay.
4. `--connection lan` forbids cloud control fallback. Any separately required cloud onboarding is disclosed and authorized independently.
5. A requested temperature of 60 outside the descriptor is rejected before send, not clipped to 30.
6. MCP is read-only by default; forged principal/confirmation fields or generic execute do not grant writes.
7. EOF/cancellation releases networking; backgrounded apps stop polling.
8. Two local processes rotating a refresh token use locks and reread to avoid overwriting a new token.
9. A crash after persisting a pending write does not trigger automatic replay on restart.
10. A partially applied climate command is not reported as complete success and is not automatically rolled back.

## Physical-device safety

Start read-only. Obtain explicit device-owner authorization for writes; prefer low-risk reversible set-value tests. Never place production credentials into public CI. Missing hardware means `blocked`, not fabricated screenshots, logs, or success rates.

## Release gate

Documentation/contracts pass; P0 execution and policy tests pass; at least one desktop CLI and Android app independently control a verified device; local MCP works in the target host; every advertised model has evidence. Broad cross-platform claims require two different protocol families. Untested iOS is not listed as supported.
