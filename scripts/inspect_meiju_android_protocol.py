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
import re
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
DEMO_SHA256 = "9fddb089c412c9662e1a5ff5aa52ab92b4e3acacc9ae040b43f471ef4c21ce27"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def literal_strings(method):
    return {
        instruction.get_operands()[-1][-1]
        for instruction in method.get_instructions()
        if instruction.get_name().startswith("const-string")
    }


def calls(method):
    return [i.get_output() for i in method.get_instructions() if i.get_name().startswith("invoke")]


def require_calls(method, names):
    found = {name: any("->" + name + "(" in call for call in calls(method)) for name in names}
    assert all(found.values()), (method.get_name(), found)
    return found


def plugin_annotations(dex):
    cls = next(c for c in dex.get_classes() if c.get_name() ==
               "Lcom/midea/business/plugin/api/PluginDownloadInfoApiService;")
    expected = {
        "getModelPluginDownloadInfo": "proxy?alias=/v1/plugin/update/getModelPlugin",
        "getLoginModelPluginDownloadInfo": "proxy?alias=/v1/plugin/update/getLoginModelPlugin",
        "getPluginDownloadInfo": "proxy?alias=/v1/plugin/update/getplugin",
        "getVirtualPluginDownloadInfo": "proxy?alias=/v1/plugin/virtualplugin/get",
    }
    found = {}
    methods = {m.get_method_idx(): m for m in cls.get_methods()}
    for item in cls.annotations_directory_item.get_method_annotations():
        name = methods[item.get_method_idx()].get_name()
        if name not in expected:
            continue
        annotations = dex.CM.get_annotation_set_item(item.get_annotations_off())
        for offset in annotations.get_annotation_off_item():
            annotation = offset.get_annotation_item().get_annotation()
            if dex.CM.get_type(annotation.get_type_idx()) != "Lretrofit2/http/POST;":
                continue
            for element in annotation.get_elements():
                if dex.CM.get_string(element.get_name_idx()) == "value":
                    value = element.get_value().get_value()
                    assert value == expected[name]
                    found[name] = expected[name]
    assert found == expected
    return found


def inspect_demo(path):
    """Inspect the public historical demo as data, never as a device profile."""
    with path.open("rb") as source:
        actual = hashlib.file_digest(source, "sha256").hexdigest()
    if actual != DEMO_SHA256:
        raise ValueError("Unreviewed demo digest; update evidence before inspecting another archive.")
    with zipfile.ZipFile(path) as archive:
        source = archive.read("T0xCA_plugdemo/weex.js").decode("utf-8")
        # Restrict the extraction to the reviewed dispatch function, rather than
        # finding a similarly named observation or simulated demo response.
        start = source.index("setFridgeCtrlModeCmd:function")
        end = source.index("modifyKongShiZhuangtai:function", start)
        function = source[start:end]
        fields = sorted(set(re.findall(r"setLuaFridgeModeCtrl\(\{([a-z_]+):i\}", function)))
        expected = sorted([
            "storage_mode", "freezing_mode", "holiday_mode", "intelligent_mode",
            "energy_saving_mode", "moisturize_mode", "preservation_mode",
            "acme_freezing_mode", "radar_mode_power", "milk_mode_power",
            "icea_mode_power", "plasma_aseptic_mode_power", "microcrystal_fresh",
            "electronic_smell", "eradicate_pesticide_residue",
        ])
        assert fields == expected
        start = source.index("setLuaFridgeModeCtrl:function")
        end = source.index("setFridgeCtrlModeCmd:function", start)
        assert 'operation:"luaControl",params:e' in source[start:end]
        assert 'var i=t?"on":"off"' in function
        assert '1!=t.isDummy' in source
    return {
        "sha256": DEMO_SHA256, "toggle_dispatch_fields": fields,
        "bridge_operation": "luaControl", "toggle_values": ["on", "off"],
        "scope": "Historical demo source with a non-dummy branch; no exact-model admission.",
    }


def inspect_consumables(archive):
    """Trace the bundled service UI without exporting unrelated source literals."""
    manifest = json.loads(archive.read("assets/weex_packages.json"))
    modules = {entry["moduleCode"]: entry["moduleNo"] for entry in manifest["weexPackets"]}
    assert modules["customer-service"] == "30001"
    path = "assets/assetsWeex/service-center/deviceInfo.js"
    raw = archive.read(path)
    source = raw.decode("utf-8")

    def section(start, end):
        offset = source.index(start)
        return source[offset:source.index(end, offset)]

    route = "/v2/remaining/queryByHomeGroupList"
    request = section("t.queryByHomeGroupList=function", "t.getFaultInfoList=function")
    assert f'sendCssHttpRequestWrapper)("{route}",e,t,!1,!1)' in request
    wrapper = section("t.sendCssHttpRequestWrapper=function", "t.sendMideaCloundSaleRequest=function")
    assert 'isWrapWithBody:r' in wrapper and 't.isWrapWithBody?{body:e}:e' in wrapper
    start = source.rfind("h=function(", 0, source.index("t.sendCssHttpRequestWrapper=function"))
    helper = source[start:source.index("t.sendCssHttpRequestWrapper=function")]
    assert 'method:"POST"' in helper and 's.sendCentralCloundRequest(h,' in helper
    assert 'plugin_version:r.default.plugin_version||"0.0.1"' in helper
    assert 'tm:Math.round(Date.now()/1e3)' in helper
    assert 'homegroupId:e.homegroupId,remainingStatusSet:["0","1","2"]' in source
    assert 'b={homeGroupInfoList:y}' in source
    assert 'I=C.data,!C.success' in source
    assert 'T.applianceConsumablesList[Symbol.iterator]()' in source
    assert 'O=P.value).applianceCode==e.deviceInfo.iotDeviceId' in source
    assert 'e.homeId=T.homeGroupId,e.consumables=O.consumables' in source
    wave = section("setWavePercent:function", "caculatePercent:function")
    assert 'e.remainingValue/e.total' in wave
    assert '4==e.remainingValueType' in wave and '2==e.remainingStatus?.4:.05' in wave
    detail = section("goDetailThrottle:function", "getDeviceInfo:function")
    assert 'url:"customer-service/materialDetail.js?from=externalPage"' in detail
    return {
        "asset": path, "sha256": digest(raw), "method": "POST", "path": route,
        "bridge": "sendCentralCloundRequest", "body_wrapper": False,
        "request_fields": ["homeGroupInfoList", "homegroupId", "remainingStatusSet", "plugin_version", "tm"],
        "response_fields": ["success", "data", "homeGroupId", "applianceConsumablesList",
                            "applianceCode", "consumables", "remainingValue", "total",
                            "remainingValueType", "remainingStatus"],
        "categorical_display": "Type 4 uses status 2 for sufficient; decorative wave height is not a measured percentage.",
        "detail_module": "customer-service/materialDetail.js",
        "detail_module_number": modules["customer-service"],
        "scope": "Bundled service-center path; no current dishwasher plugin association or authenticated response.",
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
        report["consumables_service"] = inspect_consumables(archive)
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
    classes = {c.get_name(): c for c in dex.get_classes()}

    def method(class_name, name):
        matches = [m for m in classes[class_name].get_methods() if m.get_name() == name]
        assert len(matches) == 1
        return matches[0]

    transport = "Lcom/midea/iot/msmart/control/DataTransportHelper;"
    report["control_state_dependencies"] = require_calls(method(transport, "controlDeviceState"),
        ["getDeviceStateBySN", "queryDeviceState"])
    report["send_task_dependencies"] = require_calls(method(
        "Lcom/midea/iot/msmart/control/DataTransportHelper$SendStateTask;", "run"),
        ["getDeviceStateBySN", "getStateMap", "getControlStatusProtocol"])
    wan = method(transport, "startSendDevDataByWan")
    report["wan_transport_calls"] = require_calls(wan,
        ["getTransparentUrl", "bytesToDecString", "encodeAES", "submitPost"])
    report["wan_body_fields"] = sorted(literal_strings(wan) &
        {"applianceCode", "order", "timestamp", "isFull"})
    assert len(report["wan_body_fields"]) == 4
    report["plugin_post_annotations"] = plugin_annotations(dex)
    service_bridge = method("Lcom/midea/hybrid/api/meiju/bridge/impl/DefaultBridgeBusinessNetImpl;",
                            "sendCentralCloundRequest")
    report["consumables_service"]["native_bridge_calls"] = require_calls(service_bridge,
        ["getReqWrapper", "getJsonRequest", "getBaseBody", "setBodyJson", "doPost"])
    downloader = method("Lcom/midea/hybrid/minix/plugin/service/PluginPacketDownloaderServiceImpl;",
                        "getRemotePluginPackageDownloadInfo")
    report["plugin_lookup_calls"] = require_calls(downloader,
        ["isLogin", "getModelPluginDownloadInfo", "getLoginModelPluginDownloadInfo", "setModelList"])
    report["scope"] = "Static structure only; no exact-model association or physical verification."
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("apk", type=Path)
    parser.add_argument("community_checkout", type=Path)
    parser.add_argument("--demo-plugin", type=Path, help="Optional pinned historical refrigerator demo ZIP")
    args = parser.parse_args()
    report = inspect(args.apk, args.community_checkout)
    if args.demo_plugin:
        report["historical_refrigerator_demo"] = inspect_demo(args.demo_plugin)
    print(json.dumps(report, indent=2))
