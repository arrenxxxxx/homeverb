#!/usr/bin/env python3
"""Verify dedicated login ABI admission without contacting an account service."""
import ctypes
import json
import sys
import tempfile
from native_contract import ROOT, load, validator


def verify_account_login():
    lib = ctypes.CDLL(str(ROOT / ('target/debug/libhomeverb_ffi.dylib' if sys.platform == 'darwin' else 'target/debug/libhomeverb_ffi.so')))
    signatures = {
        'homeverb_open': ([ctypes.c_char_p], ctypes.c_void_p),
        'homeverb_call': ([ctypes.c_uint64, ctypes.c_char_p], ctypes.c_void_p),
        'homeverb_close': ([ctypes.c_uint64], None),
        'homeverb_string_free': ([ctypes.c_void_p], None),
        'homeverb_set_foreground': ([ctypes.c_uint64, ctypes.c_bool], None),
        'homeverb_begin_account_login': ([ctypes.c_uint64, ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t], ctypes.c_void_p),
        'homeverb_continue_account_login': ([ctypes.c_uint64, ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t], ctypes.c_void_p),
        'homeverb_discard_account_login': ([ctypes.c_uint64, ctypes.c_uint64], ctypes.c_bool),
    }
    for name, (arguments, result) in signatures.items():
        method = getattr(lib, name)
        method.argtypes, method.restype = arguments, result
    examples = load('packages/native-contract/account-login-examples.json')
    schema = validator('native-account-login', 'result')

    def decode(pointer):
        assert pointer
        try:
            value = json.loads(ctypes.string_at(pointer))
            assert 'PRIVATE' not in json.dumps(value)
            return value
        finally:
            lib.homeverb_string_free(pointer)

    def invoke(handle, metadata, begin=True, lengths=None):
        secrets = [ctypes.create_string_buffer(value) for value in
                   ([b'PRIVATE@example.invalid', b'PRIVATE-PASSWORD'] if begin else [b'PRIVATE'])]
        arguments = []
        for index, secret in enumerate(secrets):
            length = lengths[index] if lengths else len(secret)-1
            arguments.extend([secret, length])
        method = lib.homeverb_begin_account_login if begin else lib.homeverb_continue_account_login
        result = decode(method(handle, json.dumps(metadata).encode() if metadata is not None else None, *arguments))
        for index, secret in enumerate(secrets):
            length = lengths[index] if lengths else len(secret)-1
            if 1 <= length <= 256:
                assert not any(secret.raw)
        schema.validate(result)
        return result['error']['code']

    with tempfile.TemporaryDirectory(prefix='homeverb-login-abi-') as directory:
        handle = decode(lib.homeverb_open(json.dumps({'schema_version':'1.0', 'data_dir':directory,
            'adapter':'homeverb.mock', 'allow_mock_writes':False}).encode()))['data']['handle']
        try:
            for begin, definition in [(True, 'begin'), (False, 'continue')]:
                valid = examples[definition]
                validator('native-account-login', definition).validate(valid)
                for metadata in [None, {}, *[{**valid, key: 'PRIVATE'} for key in ['token','password','answer','principal','confirmed']]]:
                    assert invoke(handle, metadata, begin) == 'INVALID_ARGUMENT'
                assert invoke(handle, valid, begin) == ('AUTH_FLOW_UNSUPPORTED' if begin else 'MIJIA_AUTH_CHALLENGE_REQUIRED')
            # One invalid input must not prevent erasing the other valid buffer.
            for lengths in [(0, 16), (257, 16), (23, 0), (23, 257)]:
                assert invoke(handle, examples['begin'], lengths=lengths) == 'INVALID_ARGUMENT'
            for length in [0, 257]:
                assert invoke(handle, examples['continue'], False, [length]) == 'INVALID_ARGUMENT'
            for method in ['begin_account_login', 'continue_account_login', 'discard_account_login']:
                request = {'schema_version':'1.0','operation_id':'ordinary','timeout_ms':1000,'request':{'method':method}}
                result = decode(lib.homeverb_call(handle,json.dumps(request).encode()))
                assert result['error']['code'] == 'INVALID_ARGUMENT'
            assert not lib.homeverb_discard_account_login(handle, 1)
            lib.homeverb_set_foreground(handle, False)
            assert invoke(handle, examples['begin']) == 'HOST_BACKGROUND'
            assert invoke(handle, examples['continue'], False) == 'MIJIA_AUTH_CHALLENGE_REQUIRED'
        finally:
            lib.homeverb_close(handle)
        assert invoke(handle, examples['begin']) == 'HOST_CLOSED'
        assert invoke(handle, examples['continue'], False) == 'HOST_CLOSED'
        assert not lib.homeverb_discard_account_login(handle, 1)
    print('PASS: three login exports, protected input erasure, closed metadata, ordinary-dispatch refusal and lifecycle admission; no account traffic.')


if __name__ == '__main__':
    verify_account_login()
