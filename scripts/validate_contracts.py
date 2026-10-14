#!/usr/bin/env python3
"""Validate fictional fixtures and semantic contract counterexamples."""
from __future__ import annotations
import copy
import json
import sys
from pathlib import Path
try:
    from jsonschema import Draft202012Validator, FormatChecker
except ImportError:
    print("Install development dependency: python -m pip install -r requirements-dev.txt", file=sys.stderr)
    raise SystemExit(2)
ROOT = Path(__file__).resolve().parents[1]
def load(path: str) -> dict:
    return json.loads((ROOT / path).read_text(encoding="utf-8"))
def main() -> int:
    pairs = [
        ("device", "device"), ("capability", "capability.climate"),
        ("command", "command"), ("adapter-manifest", "adapter-manifest"),
        ("state", "state"), ("command-result", "command-result.verified"),
        ("command-result", "command-result.accepted"),
        ("command-result", "command-result.unknown"),
    ]
    validators = {}
    for path in (ROOT / "schemas").glob("*.schema.json"):
        sch = json.loads(path.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(sch)
        validators[path.name.removesuffix(".schema.json")] = Draft202012Validator(sch, format_checker=FormatChecker())
    for sch, fixture in pairs:
        validators[sch].validate(load(f"examples/{fixture}.json"))
    parameters = load("examples/capability.climate.json")["actions"]["set"]["parameters"]
    Draft202012Validator.check_schema(parameters)
    action_validator = Draft202012Validator(parameters)
    action_validator.validate(load("examples/command.json")["arguments"])
    negative = []
    bad = load("examples/command-result.verified.json"); bad["verification"] = None
    negative.append(("verified without evidence", validators["command-result"], bad))
    bad = load("examples/command-result.unknown.json"); bad["error"]["retryable"] = True
    negative.append(("unknown allows blind retry", validators["command-result"], bad))
    bad = load("examples/adapter-manifest.json"); bad["requires_daemon"] = True
    negative.append(("daemon adapter", validators["adapter-manifest"], bad))
    bad = load("examples/adapter-manifest.json"); bad["requires_project_backend"] = True
    negative.append(("project backend adapter", validators["adapter-manifest"], bad))
    bad = load("examples/command.json"); bad["connection"] = "gateway"
    negative.append(("unsupported route", validators["command"], bad))
    bad = load("examples/command.json"); bad["timeout_ms"] = 0
    negative.append(("unbounded/zero budget", validators["command"], bad))
    negative.append(("temperature outside capability", action_validator, {"target_temperature":60}))
    negative.append(("unknown action parameter", action_validator, {"force":True}))
    for name, validator, data in negative:
        if validator.is_valid(data):
            raise AssertionError(f"Counterexample was not rejected: {name}")
    # Check that evidence samples do not claim a pre-command observation.
    verified = load("examples/command-result.verified.json")
    assert verified["verification"]["received_at"] >= verified["started_at"]
    assert load("examples/device.json")["id"] == load("examples/command.json")["device_id"]
    print(f"PASS: {len(validators)} schemas; {len(pairs)} positive fixtures; action arguments; {len(negative)} negative fixtures.")
    print("Only documentation contracts were validated; no device or application code was executed.")
    return 0
if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, AssertionError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        raise SystemExit(1)
