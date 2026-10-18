#!/usr/bin/env python3
"""Verify trusted account-recovery ABI admission with fictional credentials only."""
import ctypes
import json
import sys
import tempfile
from native_contract import ROOT, load, validator


def verify_identity_recovery():
    lib = ctypes.CDLL(str(ROOT / ('target/debug/libhomeverb_ffi.dylib' if sys.platform == 'darwin' else 'target/debug/libhomeverb_ffi.so')))
    lib.homeverb_open.argtypes = [ctypes.c_char_p]
    lib.homeverb_open.restype = ctypes.c_void_p
    lib.homeverb_string_free.argtypes = [ctypes.c_void_p]
    lib.homeverb_string_free.restype = None
    lib.homeverb_close.argtypes = [ctypes.c_uint64]
    lib.homeverb_close.restype = None
    lib.homeverb_set_foreground.argtypes = [ctypes.c_uint64, ctypes.c_bool]
    lib.homeverb_set_foreground.restype = None
    for name in ['homeverb_call', 'homeverb_confirm_identity_recovery']:
        method = getattr(lib, name)
        method.argtypes = [ctypes.c_uint64, ctypes.c_char_p]
        method.restype = ctypes.c_void_p
    lib.homeverb_prepare_identity_recovery.argtypes = [ctypes.c_uint64, ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t]
    lib.homeverb_prepare_identity_recovery.restype = ctypes.c_void_p
    lib.homeverb_discard_identity_recovery.argtypes = [ctypes.c_uint64, ctypes.c_uint64]
    lib.homeverb_discard_identity_recovery.restype = ctypes.c_bool
    examples = load('packages/native-contract/identity-recovery-examples.json')
    def decode(pointer):
        assert pointer
        try:
            result = json.loads(ctypes.string_at(pointer))
            assert 'PRIVATE' not in json.dumps(result)
            return result
        finally:
            lib.homeverb_string_free(pointer)
    def prepare(handle, metadata):
        login = ctypes.create_string_buffer(b'PRIVATE@example.invalid')
        password = ctypes.create_string_buffer(b'PRIVATE-PASSWORD')
        result = decode(lib.homeverb_prepare_identity_recovery(handle,
            json.dumps(metadata).encode() if metadata is not None else None,
            login, len(login)-1, password, len(password)-1))
        assert not any(login.raw) and not any(password.raw)
        validator('native-identity-recovery', 'prepare_result').validate(result)
        return result
    with tempfile.TemporaryDirectory(prefix='homeverb-identity-abi-') as directory:
        handle = decode(lib.homeverb_open(json.dumps({'schema_version':'1.0', 'data_dir':directory,
            'adapter':'homeverb.mock', 'allow_mock_writes':False}).encode()))['data']['handle']
        try:
            for metadata in [None, {}, *[{**examples['prepare'], key: True} for key in ['confirmed','principal','token']]]:
                assert prepare(handle, metadata)['error']['code'] == 'INVALID_ARGUMENT'
            assert prepare(handle, examples['prepare'])['error']['code'] == 'AUTH_FLOW_UNSUPPORTED'
            for key in ['confirmed','principal','token']:
                request = {**examples['confirm'],key:True}
                result = decode(lib.homeverb_confirm_identity_recovery(handle,json.dumps(request).encode()))
                assert result['error']['code'] == 'INVALID_ARGUMENT'
            result = decode(lib.homeverb_confirm_identity_recovery(handle,json.dumps(examples['confirm']).encode()))
            validator('native-identity-recovery','confirm_result').validate(result)
            assert result['error']['code'] == 'ACCOUNT_IDENTITY_CONFLICT'
            assert not lib.homeverb_discard_identity_recovery(handle,1)
            for method in ['prepare_identity_recovery','confirm_identity_recovery','discard_identity_recovery']:
                request = {'schema_version':'1.0','operation_id':'ordinary','timeout_ms':1000,'request':{'method':method}}
                assert decode(lib.homeverb_call(handle,json.dumps(request).encode()))['error']['code'] == 'INVALID_ARGUMENT'
            lib.homeverb_set_foreground(handle,False)
            assert prepare(handle,examples['prepare'])['error']['code'] == 'HOST_BACKGROUND'
        finally:
            lib.homeverb_close(handle)
        assert prepare(handle,examples['prepare'])['error']['code'] == 'HOST_CLOSED'
    print('PASS: three identity-recovery exports, protected buffer erasure, closed metadata, ordinary-dispatch refusal and host lifecycle admission; fictional credentials only.')


if __name__ == '__main__':
    verify_identity_recovery()
