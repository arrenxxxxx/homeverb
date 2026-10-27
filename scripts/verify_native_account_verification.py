#!/usr/bin/env python3
"""Verify protected verification C buffers and refusal without provider traffic."""
import ctypes
import json
import sys
import tempfile
from native_contract import ROOT, load, validator


def verify():
    lib = ctypes.CDLL(str(ROOT / ('target/debug/libhomeverb_ffi.dylib' if sys.platform == 'darwin' else 'target/debug/libhomeverb_ffi.so')))
    signatures = {
        'homeverb_open': ([ctypes.c_char_p], ctypes.c_void_p),
        'homeverb_call': ([ctypes.c_uint64, ctypes.c_char_p], ctypes.c_void_p),
        'homeverb_close': ([ctypes.c_uint64], None),
        'homeverb_string_free': ([ctypes.c_void_p], None),
        'homeverb_set_foreground': ([ctypes.c_uint64, ctypes.c_bool], None),
        'homeverb_begin_account_verification': ([ctypes.c_uint64, ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t], ctypes.c_void_p),
        'homeverb_continue_account_verification': ([ctypes.c_uint64, ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t], ctypes.c_void_p),
        'homeverb_request_account_verification_code': ([ctypes.c_uint64, ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t], ctypes.c_void_p),
        'homeverb_discard_account_verification': ([ctypes.c_uint64, ctypes.c_uint64], ctypes.c_bool),
    }
    for name, (arguments, result) in signatures.items():
        method = getattr(lib, name)
        method.argtypes, method.restype = arguments, result
    examples = load('packages/native-contract/account-verification-examples.json')
    shape = validator('native-account-verification', 'result')

    def decode(pointer):
        assert pointer
        try:
            value = json.loads(ctypes.string_at(pointer))
            assert 'PRIVATE' not in json.dumps(value)
            return value
        finally:
            lib.homeverb_string_free(pointer)

    def invoke(handle, metadata, begin=True, lengths=None, capacity=131072, null_output=False, null_input=False):
        secrets = [ctypes.create_string_buffer(value) for value in
                   ([b'PRIVATE@example.invalid', b'PRIVATE-PASSWORD'] if begin else [b'PRIVATE-TICKET'])]
        if lengths:
            for index, length in enumerate(lengths):
                if 1 <= length <= (256 if begin else 2048) and length > len(secrets[index])-1:
                    secrets[index] = ctypes.create_string_buffer(b'P' * length)
        output = (ctypes.c_uint8 * 131072)(*([0x41] * 131072))
        arguments = []
        for index, secret in enumerate(secrets):
            length = lengths[index] if lengths else len(secret)-1
            arguments.extend([None if null_input and index == 0 else secret, length])
        method = lib.homeverb_begin_account_verification if begin else lib.homeverb_continue_account_verification
        result = decode(method(handle, json.dumps(metadata).encode() if metadata is not None else None,
                               *arguments, None if null_output else output, capacity))
        for index, secret in enumerate(secrets):
            length = lengths[index] if lengths else len(secret)-1
            maximum = 256 if begin else 2048
            if 1 <= length <= maximum and not (null_input and index == 0):
                assert not any(secret.raw[:length])
        if capacity == 131072 and not null_output:
            assert not any(output)
        else:
            assert all(value == 0x41 for value in output)
        shape.validate(result)
        return result['error']['code']

    def request_code(handle, metadata, capacity=131072):
        output=(ctypes.c_uint8 * 131072)(*([0x41]*131072))
        result=decode(lib.homeverb_request_account_verification_code(handle,json.dumps(metadata).encode() if metadata is not None else None,output,capacity))
        assert all(value == (0 if capacity == 131072 else 0x41) for value in output)
        validator('native-account-verification','code_result').validate(result)
        return result['error']['code']

    with tempfile.TemporaryDirectory(prefix='homeverb-verification-abi-') as directory:
        handle = decode(lib.homeverb_open(json.dumps({'schema_version':'1.0', 'data_dir':directory,
            'adapter':'homeverb.mock', 'allow_mock_writes':False}).encode()))['data']['handle']
        try:
            for begin, definition in [(True, 'begin'), (False, 'continue')]:
                valid = examples[definition]
                validator('native-account-verification', definition).validate(valid)
                for metadata in [None, {}, *[{**valid, key: 'PRIVATE'} for key in ['token','password','answer','ticket','provider_page','principal','confirmed']]]:
                    assert invoke(handle, metadata, begin) == 'INVALID_ARGUMENT'
                assert invoke(handle, valid, begin) == ('AUTH_FLOW_UNSUPPORTED' if begin else 'MIJIA_AUTH_CHALLENGE_REQUIRED')
                for capacity in [0, 1, 131071, 131073]:
                    assert invoke(handle, valid, begin, capacity=capacity) == 'INVALID_ARGUMENT'
                assert invoke(handle, valid, begin, null_output=True) == 'INVALID_ARGUMENT'
                assert invoke(handle, valid, begin, null_input=True) == 'INVALID_ARGUMENT'
            for lengths in [(0,16), (257,16), (23,0), (23,257)]:
                assert invoke(handle, examples['begin'], lengths=lengths) == 'INVALID_ARGUMENT'
            assert invoke(handle, examples['begin'], lengths=[256, 256]) == 'AUTH_FLOW_UNSUPPORTED'
            for length in [1, 32, 256, 2048]:
                assert invoke(handle, examples['continue'], False, [length]) == 'MIJIA_AUTH_CHALLENGE_REQUIRED'
            for length in [0, 2049]:
                assert invoke(handle, examples['continue'], False, [length]) == 'INVALID_ARGUMENT'
            for method in ['begin_account_verification', 'continue_account_verification', 'discard_account_verification', 'request_account_verification_code']:
                request = {'schema_version':'1.0','operation_id':'ordinary','timeout_ms':1000,'request':{'method':method}}
                assert decode(lib.homeverb_call(handle,json.dumps(request).encode()))['error']['code'] == 'INVALID_ARGUMENT'
            code_metadata=examples['request_code']
            for metadata in [None, {}, {**code_metadata,'method':'other'}, *[{**code_metadata,key:'PRIVATE'} for key in ['ticket','phone','email','url','principal']]]:
                assert request_code(handle,metadata) == 'INVALID_ARGUMENT'
            for capacity in [0,1,131071,131073]:
                assert request_code(handle,code_metadata,capacity) == 'INVALID_ARGUMENT'
            assert request_code(handle,code_metadata) == 'MIJIA_AUTH_CHALLENGE_REQUIRED'
            assert not lib.homeverb_discard_account_verification(handle, 1)
            lib.homeverb_set_foreground(handle, False)
            assert invoke(handle, examples['begin']) == 'HOST_BACKGROUND'
            assert invoke(handle, examples['continue'], False) == 'MIJIA_AUTH_CHALLENGE_REQUIRED'
        finally:
            lib.homeverb_close(handle)
        assert invoke(handle, examples['begin']) == 'HOST_CLOSED'
        assert invoke(handle, examples['continue'], False) == 'HOST_CLOSED'
        assert not lib.homeverb_discard_account_verification(handle, 1)
        assert request_code(handle,examples['request_code']) == 'HOST_CLOSED'
    print('PASS: four verification exports, input/output clearing, invalid capacity/metadata refusal, closed/background admission and ordinary JSON refusal; no account traffic.')


if __name__ == '__main__':
    verify()
