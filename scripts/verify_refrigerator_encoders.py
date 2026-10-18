#!/usr/bin/env python3
"""Compare pinned public CA encoders with fictional data; no provider access.

Optional research dependency: lupa==2.8 (Lua 5.4). Pass a local midea-local
checkout; this script neither downloads code nor changes production admission.
"""

import argparse
import hashlib
from pathlib import Path

from lupa.lua54 import LuaError, LuaRuntime


SOURCES = {
    "T_0000_CA_5.lua": "e12f0821f4a3cda358f16140c88b12391ef22398688c3109fa6123e9259fd349",
    "T_0000_CA_310A1700_58.lua": "1f7f09e0e68bda362820a86e4beaead02cefbb8519ebb247e2e4a8b4ea553a39",
    "T_0000_CA_310A2245_66.lua": "b71b645c2db88e070c0ac4086e9b98d19ee977c8a47d50adb3786354999ac846",
    "T_0000_CA_310A1792_2026050801.lua": "dde98d255dec703a2b5329ead43951237356cee0cc8b3032a2a355f67f8d2cec",
}
# Integer vectors only. This shim does not establish the provider runtime's
# fractional conversion. Both public module shapes refer to the same bit table.
SHIM = """
bit = {}
function bit.band(a, b, ...) local v = a & b; for _, n in ipairs({...}) do v = v & n end; return v end
function bit.bor(a, b, ...) local v = a | b; for _, n in ipairs({...}) do v = v | n end; return v end
function bit.bxor(a, b) return a ~ b end
function bit.bnot(a) return ~a end
function bit.lshift(a, b) return (a << b) & 0xffffffff end
function bit.rshift(a, b) return (a & 0xffffffff) >> b end
bit.bit = bit
package.preload.bit = function() return bit end
package.preload.cjson = function() return {decode=function(_) return fixture end} end
"""


def verify(checkout: Path):
    status = {key: "on" for key in [
        "storage_mode", "freezing_mode", "intelligent_mode", "energy_saving_mode",
        "holiday_mode", "moisturize_mode", "preservation_mode", "acme_freezing_mode",
    ]}
    status.update({key: "off" for key in [
        "storage_power", "freezing_power", "left_flexzone_power", "right_flexzone_power",
    ]})
    status.update(storage_temperature=4, freezing_temperature=-18)
    # These deliberately synthetic flags are not valid-mode-combination claims.
    cases = [
        ("baseline", {"hex_length": 119}, status, (0xff, 0x64, 0x1d)),
        ("target_only", {"storage_temperature": 5, "hex_length": 119}, None, (0, 5, 0)),
        ("target_with_status", {"storage_temperature": 5, "hex_length": 119}, status, (0xff, 0x65, 0x1d)),
        ("partial_status", {"storage_temperature": 5, "hex_length": 119}, {"freezing_temperature": -18}, (0, 0x65, 0)),
        ("no_length", {"storage_temperature": 5}, status, (0xff, 0x65, 0x1d)),
    ]
    encoded = expected_failures = 0
    for name, digest in SOURCES.items():
        source = (checkout / "lua" / "ca" / name).read_bytes()
        assert hashlib.sha256(source).hexdigest() == digest, f"Unreviewed source: {name}"
        for label, control, state, expected in cases:
            lua = LuaRuntime()
            lua.execute(SHIM)
            fixture = {"deviceinfo": {"deviceSubType": 0}, "control": control}
            if state is not None:
                fixture["status"] = state
            lua.globals().fixture = lua.table_from(fixture, recursive=True)
            lua.execute(source.decode("utf-8"))
            missing_input = name == "T_0000_CA_5.lua" or (
                label == "no_length" and "310A1700" not in name
            )
            try:
                packet = bytes.fromhex(lua.globals().jsonToData("fixture"))
            except LuaError as error:
                assert missing_input and "nil" in str(error), (name, label, str(error))
                expected_failures += 1
                print(f"{name} {label}: expected missing-input error")
                continue
            assert not missing_input, (name, label)
            assert packet[0] == 0xaa and packet[1] == len(packet) - 1
            assert sum(packet[1:]) % 256 == 0
            assert len(packet) == (120 if "310A1700" in name else 131)
            actual = tuple(packet[index] for index in (11, 12, 16))
            assert actual == expected, (name, label, actual)
            encoded += 1
            print(f"{name} {label}: {len(packet)} bytes, mode/temp/power "
                  + "/".join(f"{value:02x}" for value in actual))
    assert (encoded, expected_failures) == (13, 7)
    print("13 encoded vectors and 7 expected missing-input errors matched; no physical evidence.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checkout", type=Path, help="Local checkout of the pinned midea-local source")
    verify(parser.parse_args().checkout)
