#!/usr/bin/env python3
"""Compare admitted power payloads with the pinned public E1 mapping as data.

No upstream imports, appliance traffic, account access or plugin execution.
"""
import argparse
import ast
import hashlib
import json
from pathlib import Path

SHA256 = "95fd43083418b05c9284acbb67aa8aedc4925a8858142cca4454f537028a26dd"
PRODUCTS = ("7600020L", "76006481", "7600649Q", "7600V1E7", "760Y0026")


def value(node):
    if isinstance(node, ast.Constant):
        return node.value
    if isinstance(node, ast.Dict):
        return {value(k): value(v) for k, v in zip(node.keys, node.values)}
    if isinstance(node, (ast.List, ast.Tuple)):
        return [value(v) for v in node.elts]
    if isinstance(node, ast.Attribute):
        # Imported metadata enums are names only; never import Home Assistant.
        return ast.unparse(node)
    if isinstance(node, ast.Name):
        return node.id
    raise ValueError("Unexpected executable expression in pinned mapping")


def inspect(checkout):
    source = (checkout / "custom_components/midea_auto_cloud/device_mapping/T0xE1.py").read_bytes()
    if hashlib.sha256(source).hexdigest() != SHA256:
        raise ValueError("Unreviewed E1 mapping digest")
    tree = ast.parse(source)
    mapping = value(next(n.value for n in tree.body if isinstance(n, ast.Assign)
                         and any(isinstance(t, ast.Name) and t.id == "DEVICE_MAPPING" for t in n.targets)))
    vectors = []
    for product in PRODUCTS:
        profile = mapping[product]
        centralized = profile["centralized"]
        assert centralized == (["additional"] if product == "760Y0026" else [])
        select = profile["entities"]["Platform.SELECT"]
        options = select["work_status"]["options"]
        if centralized:
            codes = sorted(v["additional"] for v in select["additional"]["options"].values())
            assert codes == [0, 9, 10, 13, 14]
        else:
            codes = [None]
        for action in ("power_on", "power_off"):
            assert options[action] == {"work_status": action}
            for additional in codes:
                payload = dict(options[action])
                if additional is not None:
                    payload["additional"] = additional
                vectors.append({"product": product, "action": action, "control": payload})
    assert mapping["7600016L"]["centralized"] == ["additional", "wash_region", "door_auto_open"]
    return {"source_sha256": SHA256, "vectors": vectors,
            "scope": "Pinned public mapping comparison only; no physical verification."}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("community_checkout", type=Path)
    args = parser.parse_args()
    print(json.dumps(inspect(args.community_checkout), indent=2))
