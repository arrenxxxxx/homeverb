#!/usr/bin/env python3
"""Check trusted scan C metadata and refusal paths without accounts or network traffic."""

import ctypes
import json
import tempfile

from native_contract import ROOT, load, validator


def verify_lan_scan():
    library = ctypes.CDLL(str(ROOT / 'target/debug/libhomeverb_ffi.so'))
    library.homeverb_open.argtypes = [ctypes.c_char_p]
    library.homeverb_open.restype = ctypes.c_void_p
    for name in ['homeverb_call', 'homeverb_confirm_lan_scan']:
        function = getattr(library, name)
        function.argtypes = [ctypes.c_uint64, ctypes.c_char_p]
        function.restype = ctypes.c_void_p
    library.homeverb_prepare_lan_scan.argtypes = [ctypes.c_uint64, ctypes.c_char_p,
                                                ctypes.c_uint32, ctypes.c_uint8]
    library.homeverb_prepare_lan_scan.restype = ctypes.c_void_p
    library.homeverb_discard_lan_scan.argtypes = [ctypes.c_uint64, ctypes.c_uint64]
    library.homeverb_discard_lan_scan.restype = ctypes.c_bool
    library.homeverb_close.argtypes = [ctypes.c_uint64]
    library.homeverb_close.restype = None
    library.homeverb_set_foreground.argtypes = [ctypes.c_uint64, ctypes.c_bool]
    library.homeverb_set_foreground.restype = None
    library.homeverb_string_free.argtypes = [ctypes.c_void_p]
    library.homeverb_string_free.restype = None
    contract = load('packages/native-contract/lan-scan.json')
    examples = load('packages/native-contract/lan-scan-examples.json')
    assert contract['api_version'] == examples['api_version'] == load('packages/native-contract/api.json')['api_version']
    assert contract['limits'] == {'max_pending_scans': 8, 'scan_ttl_ms': 60000}
    for name in contract['functions']:
        assert getattr(library, name)
    shapes = {name: validator('native-lan-scan', name) for name in
              ['prepare', 'confirm', 'prepare_result', 'confirm_result']}

    def decode(pointer):
        assert pointer
        try:
            return json.loads(ctypes.string_at(pointer))
        finally:
            library.homeverb_string_free(pointer)

    def invoke(handle, name, value, address=0xc0a82a02, prefix=24):
        raw = json.dumps(value).encode() if value is not None else None
        args = [handle, raw]
        if name == 'prepare':
            args.extend([address, prefix])
        result = decode(getattr(library, f'homeverb_{name}_lan_scan')(*args))
        shapes[f'{name}_result'].validate(result)
        return result

    with tempfile.TemporaryDirectory(prefix='homeverb-native-scan-') as directory:
        opened = decode(library.homeverb_open(json.dumps({
            'schema_version': '1.0', 'data_dir': directory, 'allow_mock_writes': False
        }).encode()))
        assert opened['error'] is None, opened
        handle = opened['data']['handle']
        try:
            for name in ['prepare', 'confirm']:
                shapes[name].validate(examples[name])
                bad_values = [None, {}, {**examples[name], 'confirmed': True},
                              {**examples[name], 'principal': 'admin'},
                              {**examples[name], 'network': examples['prepare_result']['data']['network']}]
                for value in bad_values:
                    assert not shapes[name].is_valid(value)
                    assert invoke(handle, name, value)['error']['code'] == 'INVALID_ARGUMENT'
            # Invalid networks fail before lookup; valid input reaches local device admission.
            for address, prefix in [(0x7f000001, 24), (0xc0a82a00, 24), (0xc0a82a02, 31)]:
                assert invoke(handle, 'prepare', examples['prepare'], address, prefix)['error']['code'] == 'INVALID_ARGUMENT'
            assert invoke(handle, 'prepare', examples['prepare'])['error']['code'] == 'DEVICE_NOT_FOUND'
            assert invoke(handle, 'confirm', examples['confirm'])['error']['code'] == 'LAN_SCAN_INVALID'
            assert not library.homeverb_discard_lan_scan(handle, 1)
            for method in ['prepare_lan_scan', 'confirm_lan_scan', 'discard_lan_scan']:
                request = {'schema_version': '1.0', 'operation_id': 'ordinary-scan', 'timeout_ms': 1000,
                           'request': {'method': method}}
                assert not validator('native-request').is_valid(request)
                result = decode(library.homeverb_call(handle, json.dumps(request).encode()))
                assert result['error']['code'] == 'INVALID_ARGUMENT'
            library.homeverb_set_foreground(handle, False)
            assert invoke(handle, 'prepare', examples['prepare'])['error']['code'] == 'HOST_BACKGROUND'
        finally:
            library.homeverb_close(handle)
        for name in ['prepare', 'confirm']:
            assert invoke(handle, name, examples[name])['error']['code'] == 'HOST_CLOSED'
        assert not library.homeverb_discard_lan_scan(handle, 1)
    print('PASS: three scan C exports, metadata and local admission; no account or network access. Successful scans are covered by injected Session tests.')


if __name__ == '__main__':
    verify_lan_scan()
