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
def checked_formats() -> FormatChecker:
    checker = FormatChecker()
    if checker.conforms("not-a-timestamp", "date-time"):
        raise RuntimeError("Date-time checks are unavailable. Install requirements-dev.txt before validating contracts.")
    return checker
def main() -> int:
    formats = checked_formats()
    pairs = [
        ("cli-meiju-grants", "cli-meiju-grants"),
        ("native-open", "native-open.meiju-ac-features"),
        ("capability", "capability.meiju-ac-features"),
        ("command", "command.meiju-ac-features"),
        ("state", "state.meiju-ac-features"),
        ("device", "device"), ("capability", "capability.climate"),
        ("room", "room"), ("device-preferences", "device-preferences"),
        ("room-assignment", "room-assignment"),
        ("diagnostics", "diagnostics"),
        ("command-result", "command-result.retired"),
        ("command-result", "command-result.policy-denied"),
        ("command", "command"), ("adapter-manifest", "adapter-manifest"),
        ("state", "state"), ("command-result", "command-result.verified"),
        ("state", "state.live"),
        ("command-result", "command-result.accepted"),
        ("command-result", "command-result.meiju-accepted"),
        ("command-result", "command-result.meiju-unknown"),
        ("command-result", "command-result.meiju-not-listed"),
        ("command-result", "command-result.unknown"),
        ("command-result", "command-result.partial"),
        ("command-result", "command-result.composite"),
        ("command-result", "command-result.storage-failure"),
        ("command-result", "command-result.completion"),
        ("batch-command", "batch-command"), ("batch-result", "batch-result"),
        ("device", "device.refrigerator"),
        ("capability", "capability.fridge"), ("capability", "capability.freezer"),
        ("capability", "capability.meiju-fan"), ("command", "command.meiju-fan"),
        ("capability", "capability.meiju-mode"), ("command", "command.meiju-mode"),
        ("capability", "capability.meiju-temperature"), ("command", "command.meiju-temperature"),
        ("capability", "capability.meiju-dishwasher"), ("command", "command.meiju-dishwasher"),
        ("command", "command.meiju-dishwasher-power"),
        ("capability", "capability.meiju-dishwasher-cycle-additional"), ("command", "command.meiju-dishwasher-cycle-additional"),
        ("capability", "capability.meiju-dishwasher-cycle"), ("command", "command.meiju-dishwasher-cycle"),
        ("capability", "capability.approved-cycle"), ("command", "command.approved-cycle"),
        ("command-result", "command-result.approval-expired"),
    ]
    validators = {}
    for path in (ROOT / "schemas").glob("*.schema.json"):
        sch = json.loads(path.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(sch)
        validators[path.name.removesuffix(".schema.json")] = Draft202012Validator(sch, format_checker=formats)
    for sch, fixture in pairs:
        validators[sch].validate(load(f"examples/{fixture}.json"))
    for key in ("meiju_power_grants", "meiju_fan_grants", "meiju_mode_grants", "meiju_temperature_grants"):
        assert load("schemas/cli-meiju-grants.schema.json")["properties"][key] == load("schemas/native-open.schema.json")["properties"][key]
    validators["cli-meiju-grants"].validate({"schema_version": "1.0"})
    assert load("schemas/batch-command.schema.json")["$defs"]["command"] == load("schemas/command.schema.json")
    nested_execution = json.loads(json.dumps(load("schemas/command-result.schema.json")).replace(
        '"$ref": "#"', '"$ref": "#/$defs/execution"'))
    assert load("schemas/batch-result.schema.json")["$defs"]["execution"] == nested_execution
    for status in ("verified", "accepted", "unknown", "partial"):
        batch = load("examples/batch-result.json")
        batch["status"] = status
        batch["items"] = [load(f"examples/command-result.{status}.json")]
        validators["batch-result"].validate(batch)
    batch = load("examples/batch-result.json")
    batch["status"] = "verified"
    batch["items"] = [load("examples/command-result.composite.json")]
    validators["batch-result"].validate(batch)
    parameters = load("examples/capability.climate.json")["actions"]["set"]["parameters"]
    Draft202012Validator.check_schema(parameters)
    action_validator = Draft202012Validator(parameters)
    action_validator.validate(load("examples/command.json")["arguments"])
    fan_validator = Draft202012Validator(load("examples/capability.meiju-fan.json")["actions"]["set_fan_speed"]["parameters"])
    fan_validator.validate(load("examples/command.meiju-fan.json")["arguments"])
    temperature_validator = Draft202012Validator(load("examples/capability.meiju-temperature.json")["actions"]["set"]["parameters"])
    temperature_validator.validate(load("examples/command.meiju-temperature.json")["arguments"])
    for value in [16, 30.5, 26.25, "26", True, None]:
        assert not temperature_validator.is_valid({"target_temperature":value})
    for arguments in [{}, {"target_temperature":26, "mode":"cool"}, {"target_temperature":26, "state":True}]:
        assert not temperature_validator.is_valid(arguments)
    mode_validator = Draft202012Validator(load("examples/capability.meiju-mode.json")["actions"]["activate_mode"]["parameters"])
    mode_validator.validate(load("examples/command.meiju-mode.json")["arguments"])
    for mode in ["cool", "heat", "auto", "dry", "fan"]:
        mode_validator.validate({"mode": mode})
        validators["cli-meiju-grants"].validate({"schema_version":"1.0", "meiju_mode_grants":[
            {"device_id":"30000000-0000-4000-8000-000000000001", "mode":mode}]})
    for args in [{}, {"mode":"off"}, {"mode":"fan_only"}, {"mode":"COOL"}, {"mode":1},
                 {"mode":"cool", "power":"off"}, {"mode":"cool", "temperature":26}]:
        assert not mode_validator.is_valid(args), args
    for speed in [20, 40, 60, 80, 100, 102, 20.0]:
        fan_validator.validate({"wind_speed": speed})
    for args in [{}, {"wind_speed": "40"}, {"wind_speed": 40.5}, {"wind_speed": 101},
                 {"wind_speed": True}, {"wind_speed": 0}, {"wind_speed": 40, "power": "on"}]:
        assert not fan_validator.is_valid(args), args
    property_validators = {}
    for name, prop in load("examples/capability.climate.json")["properties"].items():
        Draft202012Validator.check_schema(prop)
        property_validators[name] = Draft202012Validator(prop)
        property_validators[name].validate(load("examples/state.live.json")["observations"][f"climate.{name}"]["value"])
    numeric_target = Draft202012Validator({"type": "number", "const": 25})
    numeric_target.validate(25.0)
    Draft202012Validator({"type": "number", "const": 25.0}).validate(25)
    boundary_receipt = load("examples/command-result.composite.json")
    boundary_receipt["request_id"] = "r" * 256
    for index, item in enumerate(boundary_receipt["items"]):
        item["request_id"] = boundary_receipt["request_id"] + f":{index}"
    validators["command-result"].validate(boundary_receipt)
    completion = load("examples/command-result.completion.json")
    del completion["verification"]["observed_at"]
    validators["command-result"].validate(completion)
    for declaration in (None, False):
        for idempotency in ("read_only", "set_value", "non_idempotent", "unknown"):
            intact = load("examples/capability.climate.json")
            action = intact["actions"]["set"]
            if declaration is None:
                del action["independent_fields"]
            else:
                action["independent_fields"] = declaration
            action["idempotency"] = idempotency
            validators["capability"].validate(intact)
    for max_age_ms in (1, 1000.0, 30000, 120000, 9007199254740991):
        fresh = load("examples/capability.climate.json")
        fresh["properties"]["target_temperature"]["max_age_ms"] = max_age_ms
        validators["capability"].validate(fresh)
    approved_action = Draft202012Validator(load("examples/capability.approved-cycle.json")["actions"]["start"]["parameters"])
    approved_action.validate(load("examples/command.approved-cycle.json")["arguments"])
    negative = []
    for feature, value in [("eco", "on"), ("eco", 1), ("breezeless", 4),
                           ("horizontal_direction", 30), ("unknown", True), ("self_clean", True),
                           ("breezeless", 1.5), ("display", None)]:
        bad = load("examples/native-open.meiju-ac-features.json")
        bad["meiju_ac_feature_grants"][0].update(feature=feature, value=value)
        negative.append((f"invalid AC feature {feature}/{value}", validators["native-open"], bad))
    for change in ["duplicate", "adapter", "scope", "authority", "capacity"]:
        bad = load("examples/native-open.meiju-ac-features.json")
        if change == "duplicate": bad["meiju_ac_feature_grants"] *= 2
        elif change == "adapter": bad["adapter"] = "homeverb.mock"
        elif change == "scope": bad["meiju_ac_feature_grants"][0]["device_id"] = "*"
        elif change == "authority": bad["meiju_ac_feature_grants"][0]["confirmed"] = True
        else: bad["meiju_ac_feature_grants"] = [dict(device_id=f"30000000-0000-4000-8000-{i:012d}", feature="eco", value=True) for i in range(1025)]
        negative.append((f"invalid AC startup {change}", validators["native-open"], bad))
    for value in [None, 1, "true", {}]:
        bad = load("examples/capability.meiju-ac-features.json")
        bad["properties"]["eco"]["control_requires_observation"] = value
        negative.append((f"invalid observation gate {value}", validators["capability"], bad))
    for code in ["APPROVAL_INVALID", "APPROVAL_EXPIRED"]:
        bad = load("examples/command-result.approval-expired.json")
        bad["error"]["code"] = code
        validators["command-result"].validate(bad)
        bad["attempts"] = [{"transport":"cloud", "phase":"possibly_sent", "duration_ms":1}]
        negative.append((f"sent {code} receipt", validators["command-result"], bad))
    for field in ["approved", "user_approved", "approval_id"]:
        bad = load("examples/command.approved-cycle.json")
        bad[field] = True
        negative.append((f"forged command approval {field}", validators["command"], bad))
    for value in [15.5, 30.5, 26.25, "26", True, None]:
        bad = {"schema_version":"1.0", "meiju_temperature_grants":[
            {"device_id":"30000000-0000-4000-8000-000000000001", "target_temperature":value}]}
        negative.append((f"invalid CLI setpoint {value}", validators["cli-meiju-grants"], bad))
    capacity = [{'device_id':f'30000000-0000-4000-8000-{index:012d}', 'target_temperature':26}
                for index in range(1, 258)]
    validators["cli-meiju-grants"].validate({"schema_version":"1.0", "meiju_temperature_grants":capacity[:256]})
    negative.append(("257 distinct temperature grants", validators["cli-meiju-grants"],
                     {"schema_version":"1.0", "meiju_temperature_grants":capacity}))
    for mode in [None, True, 1, "off", "fan_only", "COOL"]:
        bad = {"schema_version":"1.0", "meiju_mode_grants":[
            {"device_id":"30000000-0000-4000-8000-000000000001", "mode":mode}]}
        negative.append((f"invalid CLI activation mode {mode}", validators["cli-meiju-grants"], bad))
    for key, value in [("schema_version", "2.0"), ("confirmed", True), ("principal", "owner"),
                       ("password", "PRIVATE"), ("meiju_power_grants", None)]:
        bad = load("examples/cli-meiju-grants.json"); bad[key] = value
        negative.append((f"invalid CLI grant field {key}", validators["cli-meiju-grants"], bad))
    for key in ("meiju_power_grants", "meiju_fan_grants"):
        for field, value in [("device_id", "*"), ("device_id", "provider-appliance-id"),
                             ("connection", "lan"), ("action", "start")]:
            bad = load("examples/cli-meiju-grants.json"); bad[key][0][field] = value
            negative.append((f"invalid CLI grant scope {key}/{field}/{value}", validators["cli-meiju-grants"], bad))
        bad = load("examples/cli-meiju-grants.json"); bad[key] *= 2
        negative.append((f"duplicate CLI grants {key}", validators["cli-meiju-grants"], bad))
    bad = load("examples/command-result.meiju-not-listed.json")
    bad["status"] = "verified"
    negative.append(("inventory refusal promoted without evidence", validators["command-result"], bad))
    bad = load("examples/command.meiju-fan.json")
    bad["listed"] = True
    negative.append(("command forges protected inventory membership", validators["command"], bad))
    for status in ("accepted", "unknown"):
        bad = load(f"examples/command-result.meiju-{status}.json")
        bad["status"] = "verified"
        negative.append((f"cloud {status} promoted without evidence", validators["command-result"], bad))
    for scope in (None, True, 7, [], {}, ""):
        bad = load("examples/capability.fridge.json"); bad["component_id"] = scope
        negative.append((f"invalid capability component scope {scope}", validators["capability"], bad))
    for field, value in (("commands", []), ("commands", [load("examples/command.json")] * 33),
                         ("schema_version", "9.0"), ("principal", "admin")):
        bad = load("examples/batch-command.json"); bad[field] = value
        negative.append((f"invalid batch input {field}", validators["batch-command"], bad))
    bad = load("examples/batch-command.json"); bad["commands"][1]["arguments"] = None
    negative.append(("invalid nested command", validators["batch-command"], bad))
    for field, value in (("items", []), ("items", [load("examples/command-result.verified.json")] * 33),
                         ("status", "verified"), ("device_id", "fabricated-batch-device")):
        bad = load("examples/batch-result.json"); bad[field] = value
        negative.append((f"invalid batch result {field}", validators["batch-result"], bad))
    bad = load("examples/batch-result.json"); bad["items"][0]["verification"] = None
    negative.append(("batch verified item lacks evidence", validators["batch-result"], bad))
    for status in ("verified", "accepted", "unknown"):
        bad = load("examples/batch-result.json"); bad["items"] = [load(f"examples/command-result.{status}.json")]
        negative.append((f"uniform {status} batch mislabeled partial", validators["batch-result"], bad))
    for field, value in (("matches_target", False), ("received_at", "invalid-time"),
                         ("provider_token", "private-marker"), ("connection_id", "private-marker")):
        bad = load("examples/command-result.completion.json")
        bad["verification"][field] = value
        negative.append((f"invalid completion projection {field}", validators["command-result"], bad))
    bad = load("examples/command-result.completion.json"); bad["status"] = "accepted"
    negative.append(("accepted with completion verification", validators["command-result"], bad))
    for max_age_ms in (0, -1, 0.5, 9007199254740992, True, "1000", None):
        bad = load("examples/capability.climate.json")
        bad["properties"]["target_temperature"]["max_age_ms"] = max_age_ms
        negative.append((f"invalid property freshness {max_age_ms}", validators["capability"], bad))
    for idempotency in ("read_only", "non_idempotent", "unknown"):
        bad = load("examples/capability.climate.json")
        bad["actions"]["set"]["idempotency"] = idempotency
        negative.append((f"field splitting for {idempotency}", validators["capability"], bad))
    bad = load("examples/capability.climate.json"); bad["actions"]["set"]["atomic"] = True
    negative.append(("atomic action requests splitting", validators["capability"], bad))
    bad = load("examples/capability.climate.json"); bad["actions"]["set"]["independent_fields"] = "true"
    negative.append(("nonboolean field independence", validators["capability"], bad))
    bad = copy.deepcopy(boundary_receipt); bad["items"][0]["request_id"] = ""
    negative.append(("empty composite correlation ID", validators["command-result"], bad))
    bad = load("examples/device.json"); bad["schema_version"] = "9.0"
    negative.append(("unsupported device metadata version", validators["device"], bad))
    bad = load("examples/device.json"); bad["connections"] = []
    negative.append(("device without connection metadata", validators["device"], bad))
    bad = load("examples/capability.climate.json"); bad["schema_version"] = "9.0"
    negative.append(("unsupported capability metadata version", validators["capability"], bad))
    bad = load("examples/capability.climate.json"); bad["revision"] = 0
    negative.append(("zero capability revision", validators["capability"], bad))
    bad = load("examples/capability.climate.json"); bad["properties"]["mode"]["type"] = "unknown"
    negative.append(("unsupported property type", validators["capability"], bad))
    bad = load("examples/capability.climate.json"); bad["actions"]["set"]["transports"].append("lan")
    negative.append(("duplicate action transports", validators["capability"], bad))
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
    bad = load("examples/command.json"); bad["grants"] = [{"transports": ["cloud"]}]
    negative.append(("command supplies its own grants", validators["command"], bad))
    bad = load("examples/command.json"); bad["timeout_ms"] = 0
    negative.append(("unbounded/zero budget", validators["command"], bad))
    bad = load("examples/state.live.json"); bad["schema_version"] = "9.0"
    negative.append(("unsupported state version", validators["state"], bad))
    bad = load("examples/state.live.json"); bad["observations"]["power.state"]["source"] = "unreviewed"
    negative.append(("unknown observation source", validators["state"], bad))
    bad = load("examples/state.live.json"); bad["observations"]["power.state"]["received_at"] = "not-a-timestamp"
    negative.append(("malformed observation timestamp", validators["state"], bad))
    bad = load("examples/state.live.json"); bad["warnings"] = [""]
    negative.append(("empty observation warning", validators["state"], bad))
    negative.append(("temperature outside capability", action_validator, {"target_temperature":60}))
    negative.append(("unknown action parameter", action_validator, {"force":True}))
    for name, value in [("target_temperature", 60), ("target_temperature", 24.25),
                        ("target_temperature", "25"), ("target_temperature", None), ("mode", "unsupported")]:
        negative.append((f"invalid observed property {name}={value}", property_validators[name], value))
    for value in [25.5, "25", True, None]:
        negative.append((f"nonmatching numeric target {value}", numeric_target, value))
    negative.append(("integer precision lost during comparison",
                     Draft202012Validator({"type": "number", "const": 9007199254740993}), 9007199254740992.0))
    bad = load("examples/command-result.partial.json"); del bad["items"]
    negative.append(("partial without items", validators["command-result"], bad))
    bad = load("examples/command-result.composite.json"); bad["items"][0]["verification"] = None
    negative.append(("composite child without evidence", validators["command-result"], bad))
    bad = load("examples/command-result.composite.json"); bad["items"][0] = load("examples/command-result.unknown.json")
    negative.append(("verified composite with unknown child", validators["command-result"], bad))
    bad = load("examples/command-result.storage-failure.json"); bad["items"][1]["attempts"] = [{"transport":"lan", "phase":"accepted", "duration_ms":1}]
    negative.append(("not executed item with transmission attempt", validators["command-result"], bad))
    bad = load("examples/command-result.storage-failure.json"); bad["items"][0]["error"] = {"code":"STORAGE_ERROR", "message":"The local store is unavailable.", "retryable":False}
    negative.append(("storage error replaces verified outcome", validators["command-result"], bad))
    bad = load("examples/room.json"); bad["name"] = ""
    negative.append(("empty room name", validators["room"], bad))
    bad = load("examples/device-preferences.json"); bad["position"] = -1
    negative.append(("negative device order", validators["device-preferences"], bad))
    bad = load("examples/device-preferences.json"); bad["principal"] = "admin"
    negative.append(("preferences select principal", validators["device-preferences"], bad))
    bad = load("examples/room-assignment.json"); del bad["room_id"]
    negative.append(("room assignment omits destination", validators["room-assignment"], bad))
    bad = load("examples/room-assignment.json"); bad["device_ids"] = []
    negative.append(("empty room assignment", validators["room-assignment"], bad))
    bad = load("examples/room-assignment.json"); bad["device_ids"] *= 2
    negative.append(("duplicate room assignment IDs", validators["room-assignment"], bad))
    bad = load("examples/room-assignment.json"); bad["device_ids"] = [f"device-{i}" for i in range(257)]
    negative.append(("oversized room assignment", validators["room-assignment"], bad))
    for field in ("device_id", "arguments", "message", "warnings", "credential_ref"):
        bad = load("examples/diagnostics.json"); bad["recent_receipts"][0][field] = "private-marker"
        negative.append((f"diagnostics exposes {field}", validators["diagnostics"], bad))
    bad = load("examples/diagnostics.json"); bad["recent_receipts"][0]["error_code"] = "PRIVATE_MARKER"
    negative.append(("diagnostics exposes an unreviewed code", validators["diagnostics"], bad))
    bad = load("examples/diagnostics.json"); bad["recent_receipts"] *= 101
    negative.append(("unbounded diagnostic receipts", validators["diagnostics"], bad))
    bad = load("examples/diagnostics.json"); bad["recent_receipts"][0]["attempts"] *= 33
    negative.append(("unbounded diagnostic attempts", validators["diagnostics"], bad))
    cycle = Draft202012Validator(load("examples/capability.meiju-dishwasher.json")["actions"]["start_cycle"]["parameters"])
    cycle.validate(load("examples/command.meiju-dishwasher.json")["arguments"])
    for action in load("examples/capability.meiju-dishwasher-cycle-additional.json")["actions"].values():
        shape = Draft202012Validator(action["parameters"])
        shape.validate(load("examples/command.meiju-dishwasher-cycle-additional.json")["arguments"])
        for code in [0, 9, 10, 13, 14]:
            shape.validate({"additional": code})
        for arguments in [{}, {"additional": None}, {"additional": True}, {"additional": 1},
                          {"additional": 9.5}, {"additional": "9"}, {"additional": 9, "program": "eco_wash"}]:
            negative.append(("invalid dishwasher cycle additional option", shape, arguments))
    for action in load("examples/capability.meiju-dishwasher-cycle.json")["actions"].values():
        shape = Draft202012Validator(action["parameters"])
        shape.validate(load("examples/command.meiju-dishwasher-cycle.json")["arguments"])
        for arguments in [None, [], {"confirmed": True}, {"operator": "pause"}, {"program": "eco_wash"}]:
            negative.append(("invalid dishwasher cycle scope", shape, arguments))
    for arguments in [{"program":"eco_wash"}, {"program":"neutral_gear","additional":0},
                      {"program":"eco_wash","additional":1}, {"program":"eco_wash","additional":None},
                      {"program":"eco_wash","additional":0,"confirmed":True}]:
        negative.append(("invalid dishwasher start scope", cycle, arguments))
    # Resolve native definition references locally, without provider or network access.
    from native_contract import validator as native_validator
    approval = load("packages/native-contract/approval-examples.json")
    assert approval["api_version"] == load("packages/native-contract/api.json")["api_version"]
    for definition in ["prepare", "execute"]:
        shape = native_validator("native-approval", definition)
        shape.validate(approval[definition])
        for field, value in [("approved", True), ("principal", "admin"),
                             ("operation_id", ""), ("schema_version", "2.0")]:
            bad = json.loads(json.dumps(approval[definition])); bad[field] = value
            negative.append((f"approval {definition} rejects {field}", shape, bad))
    for value in [0, -1, 18446744073709551616, "1", True]:
        bad = dict(approval["execute"]); bad["approval_id"] = value
        negative.append(("invalid native approval ID", native_validator("native-approval", "execute"), bad))
    scan = load("packages/native-contract/lan-scan-examples.json")
    assert scan["api_version"] == load("packages/native-contract/api.json")["api_version"]
    for definition in ["prepare", "confirm", "prepare_result"]:
        shape = native_validator("native-lan-scan", definition)
        shape.validate(scan[definition])
        for field in ["confirmed", "principal", "endpoint", "network", "appliance_id"]:
            bad = copy.deepcopy(scan[definition]); bad[field] = "FORBIDDEN"
            negative.append((f"scan {definition} rejects {field}", shape, bad))
    for outcome in scan["outcomes"]:
        shape = native_validator("native-lan-scan", "confirm_result")
        shape.validate(outcome)
        for field, value in [("authenticated", True), ("route_added", True),
                             ("status", "verified"), ("endpoint", "192.168.42.3"),
                             ("appliance_id", "FORBIDDEN")]:
            bad = copy.deepcopy(outcome); bad["data"][field] = value
            negative.append((f"scan result rejects {field}", shape, bad))
    for value in [0, -1, 18446744073709551616, "1", True]:
        bad = dict(scan["confirm"]); bad["scan_id"] = value
        negative.append(("invalid scan ID", native_validator("native-lan-scan", "confirm"), bad))
    link = load("packages/native-contract/connection-link-examples.json")
    assert link["api_version"] == load("packages/native-contract/api.json")["api_version"]
    for definition in ["prepare", "confirm", "prepare_result", "confirm_result"]:
        shape = native_validator("native-connection-link", definition)
        shape.validate(link[definition])
        for field in ["confirmed", "principal", "endpoint", "token", "appliance_id"]:
            bad = copy.deepcopy(link[definition]); bad[field] = "FORBIDDEN"
            negative.append((f"link {definition} rejects {field}", shape, bad))
    for value in [0, -1, 18446744073709551616, "1", True]:
        bad = dict(link["confirm"]); bad["link_id"] = value
        negative.append(("invalid link ID", native_validator("native-connection-link", "confirm"), bad))
    for field in ["endpoint", "token", "appliance_id"]:
        bad = copy.deepcopy(link["prepare"]); bad["connection"][field] = "FORBIDDEN"
        negative.append((f"link connection rejects {field}", native_validator("native-connection-link", "prepare"), bad))
    for field in ["descriptor", "anchor", "connection", "transport_grants", "expires_at"]:
        bad = copy.deepcopy(link["prepare_result"]); del bad["data"][field]
        negative.append((f"link preview requires {field}", native_validator("native-connection-link", "prepare_result"), bad))
    recovery = load("packages/native-contract/identity-recovery-examples.json")
    assert recovery["api_version"] == load("packages/native-contract/api.json")["api_version"]
    for definition in ["prepare", "confirm", "prepare_result", "confirm_result"]:
        shape = native_validator("native-identity-recovery", definition)
        shape.validate(recovery[definition])
        for field in ["confirmed", "principal", "token", "iotUserId", "login", "password"]:
            bad = copy.deepcopy(recovery[definition]); bad[field] = "FORBIDDEN"
            negative.append((f"identity recovery {definition} rejects {field}", shape, bad))
    for value in [0, -1, 18446744073709551616, "1", True]:
        bad = dict(recovery["confirm"]); bad["recovery_id"] = value
        negative.append(("invalid recovery ID", native_validator("native-identity-recovery", "confirm"), bad))
    for name, validator, data in negative:
        if validator.is_valid(data):
            raise AssertionError(f"Counterexample was not rejected: {name}")
    # Check that evidence samples do not claim a pre-command observation.
    verified = load("examples/command-result.verified.json")
    assert verified["verification"]["received_at"] >= verified["started_at"]
    assert load("examples/device.json")["id"] == load("examples/command.json")["device_id"]
    print(f"PASS: {len(validators)} schemas; {len(pairs) + 2} positive fixtures; action arguments and observed values; {len(negative)} negative fixtures.")
    print("Only documentation contracts were validated; no device or application code was executed.")
    return 0
if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ValueError, AssertionError, RuntimeError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        raise SystemExit(1)
