#!/usr/bin/env python3
"""Verify the modern QR C boundary with fictional metadata and no provider traffic."""
import ctypes
import json
import sys
import tempfile
from native_contract import ROOT, load, validator

CAPACITY = 8192


def verify():
    library = ROOT / ('target/debug/libhomeverb_ffi.dylib' if sys.platform == 'darwin' else 'target/debug/libhomeverb_ffi.so')
    lib = ctypes.CDLL(str(library))
    signatures = {
        'homeverb_open': ([ctypes.c_char_p], ctypes.c_void_p),
        'homeverb_call': ([ctypes.c_uint64, ctypes.c_char_p], ctypes.c_void_p),
        'homeverb_close': ([ctypes.c_uint64], None),
        'homeverb_string_free': ([ctypes.c_void_p], None),
        'homeverb_set_foreground': ([ctypes.c_uint64, ctypes.c_bool], None),
        'homeverb_cancel': ([ctypes.c_uint64, ctypes.c_char_p], ctypes.c_bool),
        'homeverb_begin_modern_account_login': ([ctypes.c_uint64, ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t], ctypes.c_void_p),
        'homeverb_continue_modern_account_login': ([ctypes.c_uint64, ctypes.c_char_p], ctypes.c_void_p),
        'homeverb_discard_modern_account_login': ([ctypes.c_uint64, ctypes.c_uint64], ctypes.c_bool),
        'homeverb_modern_account_session': ([ctypes.c_uint64, ctypes.c_char_p], ctypes.c_void_p),
    }
    for name, (arguments, result) in signatures.items():
        function = getattr(lib, name)
        function.argtypes, function.restype = arguments, result
    example = load('packages/native-contract/modern-account-examples.json')
    manifest = load('packages/native-contract/modern-account.json')
    assert manifest['api_version'] == load('packages/native-contract/api.json')['api_version']
    assert set(manifest['functions']) == {name for name in signatures if 'modern_account' in name}
    cases = 0

    def decode(pointer):
        assert pointer
        try:
            value = json.loads(ctypes.string_at(pointer))
            assert 'PRIVATE' not in json.dumps(value)
            return value
        finally:
            lib.homeverb_string_free(pointer)

    def encode(metadata):
        if isinstance(metadata, bytes):
            return metadata
        return json.dumps(metadata).encode() if metadata is not None else None

    def begin(handle, metadata, capacity=CAPACITY, null=False):
        nonlocal cases
        buffer = (ctypes.c_uint8 * CAPACITY)(*([0x41] * CAPACITY))
        value = decode(lib.homeverb_begin_modern_account_login(handle, encode(metadata), None if null else buffer, capacity))
        assert all(byte == (0 if capacity == CAPACITY and not null else 0x41) for byte in buffer)
        validator('native-modern-account', 'begin_result').validate(value)
        assert value['data'] is None
        cases += 1
        return value['error']['code']

    def continuation(handle, metadata):
        nonlocal cases
        value = decode(lib.homeverb_continue_modern_account_login(handle, encode(metadata)))
        validator('native-modern-account', 'continue_result').validate(value)
        assert value['data'] is None
        cases += 1
        return value['error']['code']

    def session(handle, metadata):
        nonlocal cases
        value = decode(lib.homeverb_modern_account_session(handle, encode(metadata)))
        validator('native-modern-account', 'session_result').validate(value)
        assert value['data'] is None
        cases += 1
        return value['error']['code']

    with tempfile.TemporaryDirectory(prefix='homeverb-modern-abi-') as directory:
        handle = decode(lib.homeverb_open(json.dumps({'schema_version':'1.0', 'data_dir':directory,
            'adapter':'homeverb.mock', 'allow_mock_writes':False}).encode()))['data']['handle']
        try:
            for action, name in [(begin, 'begin'), (continuation, 'continue'), (session, 'session')]:
                valid = example[name]
                validator('native-modern-account', name).validate(valid)
                for invalid in [None, {}, b'{', b'\xff', *[{**valid, field:'PRIVATE'} for field in
                                ['scan_payload','token','password','url','origin','principal','confirmed']]]:
                    assert action(handle, invalid) == 'INVALID_ARGUMENT'
            for capacity in [0, 1, CAPACITY-1, CAPACITY+1, 2**63]:
                assert begin(handle, example['begin'], capacity=capacity) == 'INVALID_ARGUMENT'
            assert begin(handle, example['begin'], null=True) == 'INVALID_ARGUMENT'
            assert begin(handle, example['begin']) == 'AUTH_FLOW_UNSUPPORTED'
            assert continuation(handle, example['continue']) == 'MIJIA_AUTH_CHALLENGE_REQUIRED'
            assert not lib.homeverb_discard_modern_account_login(handle, example['continue']['challenge_id'])
            for operation in ['refresh','recover','renew']:
                assert session(handle, {**example['session'], 'operation':operation}) == 'AUTH_FLOW_UNSUPPORTED'
            for operation in ['', 'retry', 'reconnect', True, None]:
                assert session(handle, {**example['session'], 'operation':operation}) == 'INVALID_ARGUMENT'
            assert session(handle, {**example['session'], 'account_id':'invalid'}) == 'INVALID_ARGUMENT'
            for request in [{**example['begin'], 'schema_version':'future'}, {**example['begin'], 'timeout_ms':0},
                            {**example['begin'], 'operation_id':''}]:
                assert begin(handle, request) == 'INVALID_ARGUMENT'
            for provider in [None, 'midea.meiju']:
                assert begin(handle, {**example['begin'], 'provider':provider}) == 'AUTH_FLOW_UNSUPPORTED'
            for method in ['begin_modern_account_login','continue_modern_account_login','discard_modern_account_login','modern_account_session']:
                request = {'schema_version':'1.0','operation_id':'ordinary','timeout_ms':1000,'request':{'method':method}}
                assert decode(lib.homeverb_call(handle,json.dumps(request).encode()))['error']['code'] == 'INVALID_ARGUMENT'
            lib.homeverb_set_foreground(handle, False)
            assert begin(handle, example['begin']) == 'HOST_BACKGROUND'
            assert session(handle, example['session']) == 'HOST_BACKGROUND'
            assert continuation(handle, example['continue']) == 'MIJIA_AUTH_CHALLENGE_REQUIRED'
            lib.homeverb_set_foreground(handle, True)
            assert not lib.homeverb_cancel(handle, b'not-active')
            assert begin(handle, example['begin']) == 'AUTH_FLOW_UNSUPPORTED'
        finally:
            lib.homeverb_close(handle)
        assert begin(handle, example['begin']) == 'HOST_CLOSED'
        assert continuation(handle, example['continue']) == 'HOST_CLOSED'
        assert session(handle, example['session']) == 'HOST_CLOSED'
        assert not lib.homeverb_discard_modern_account_login(handle, 7)
    print(f'PASS: four modern-account C exports; {cases} refusal/buffer/lifecycle cases. No provider traffic; successful QR/account behavior uses fictional Host and projection tests.')


if __name__ == '__main__':
    verify()
