#!/usr/bin/env python3
"""Inspect one pinned public Meiju APK without installing or executing its code.

Optional research dependencies: androguard==4.1.3 and pycryptodome==4.0.1.
Inputs are an existing APK and the pinned public midea_auto_cloud checkout.
Only hashes, known field names and structural checks are emitted. No networking,
account access, APK keys, app source or appliance identifiers are exported.
"""

import argparse
import ast
import hashlib
import json
from pathlib import Path
import zipfile

from loguru import logger

logger.remove()  # DEX parser diagnostics must not dump arbitrary app strings.
from androguard.core.dex import DEX  # noqa: E402
from Crypto.Cipher import AES  # noqa: E402
from Crypto.Util.Padding import unpad  # noqa: E402


APK_SHA256 = "5da32e92638806d55e9ce533b917bbbbba6db0bd7d3a02c45e08ed0dee62d849"
DECODER_SHA256 = "2c679f9c3ba59df17c33c89c55019757820d79248afc2cadd34aadb16488a7ee"
PROTOCOL_CLASS = "Lcom/midea/iot/msmart/protocol/ProtocolControlManager;"
CALLBACK_CLASS = "Lcom/midea/iot/msmart/protocol/ProtocolControlManager$2;"
LOOKUP_FIELDS = {
    "applianceSn", "applianceType", "applianceMFCode", "modelNumber",
    "version", "iotAppId", "smartProductId",
}
DECODED_DIGESTS = {
    "T_0000_CA_5.lua": "2cafdc84e967d79b86e7687939f2b114353690b22444987f52ce05e8bdbe6b5a",
    "T_0000_E1_3.lua": "95ed6b64d97b978d66e7d699d57e92aaa868c6ae777e97f0c68528d07d3437e1",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def literal_strings(method):
    return {
        instruction.get_operands()[-1][-1]
        for instruction in method.get_instructions()
        if instruction.get_name().startswith("const-string")
    }


def inspect(apk_path: Path, checkout: Path):
    with apk_path.open("rb") as source:
        actual = hashlib.file_digest(source, "sha256").hexdigest()
    if actual != APK_SHA256:
        raise ValueError("Unreviewed APK digest; update evidence before inspecting another release.")
    security = (checkout / "custom_components/midea_auto_cloud/core/security.py").read_bytes()
    if digest(security) != DECODER_SHA256:
        raise ValueError("Unreviewed community decoder digest.")
    # Read the existing ADR-0022 community decoding literal; execute no upstream
    # Python and extract no new signing/credential material from the APK.
    tree = ast.parse(security)
    cls = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == "MeijuCloudSecurity")
    init = next(n for n in cls.body if isinstance(n, ast.FunctionDef) and n.name == "__init__")
    key = format(ast.literal_eval(init.body[0].value.args[-1]), "x").encode("ascii")
    report = {"apk_sha256": actual, "version": "11.7.10.6", "bundled_protocols": {}}
    with zipfile.ZipFile(apk_path) as archive:
        for name, expected in DECODED_DIGESTS.items():
            encoded = archive.read("assets/lua/" + name)
            decoded = unpad(AES.new(key, AES.MODE_ECB).decrypt(bytes.fromhex(encoded.decode("ascii"))), len(key))
            if digest(decoded) != expected:
                raise ValueError("Decoded protocol digest mismatch.")
            # The decoded Lua is data here: it is never evaluated or written out.
            report["bundled_protocols"][name] = {
                "encrypted_sha256": digest(encoded), "decoded_sha256": expected,
                "contains_exact_model_literal": b"600WUFIPZM" in decoded,
            }
        dex = DEX(archive.read("classes8.dex"))
    protocol = next(c for c in dex.get_classes() if c.get_name() == PROTOCOL_CLASS)
    lookup = next(m for m in protocol.get_methods() if m.get_name() == "getLastVersion")
    fields = literal_strings(lookup) & LOOKUP_FIELDS
    assert fields == LOOKUP_FIELDS
    report["lookup_field_literals"] = sorted(fields)
    prepare = next(m for m in protocol.get_methods() if m.get_name() == "prepareLocalLuaFile")
    calls = [i.get_output() for i in prepare.get_instructions() if i.get_name().startswith("invoke")]
    checks = {
        name: any("->" + name + "(" in call for call in calls)
        for name in ["getLocalLuaFiles", "getAssetsLuaFiles", "checkDownloadLastLuaFile"]
    }
    assert all(checks.values())
    report["preparation_calls"] = checks
    callback = next(c for c in dex.get_classes() if c.get_name() == CALLBACK_CLASS)
    methods = [m for m in callback.get_methods() if m.get_name() == "onComplete"]
    fields = set().union(*(literal_strings(m) for m in methods)) & {"deviceinfo", "deviceSubType", "control", "status"}
    assert fields == {"deviceinfo", "deviceSubType", "control", "status"}
    assert any("->encodeJsonToBytes(" in i.get_output() for m in methods for i in m.get_instructions())
    report["encoder_field_literals"] = sorted(fields)
    report["scope"] = "Static structure only; no exact-model association or physical verification."
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("apk", type=Path)
    parser.add_argument("community_checkout", type=Path)
    args = parser.parse_args()
    print(json.dumps(inspect(args.apk, args.community_checkout), indent=2))
