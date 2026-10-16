#!/usr/bin/env python3
"""Check trusted connection-link C admission with fictional local metadata only."""

import copy
import ctypes
import sys
import json
import sqlite3
import tempfile
from pathlib import Path

from native_contract import ROOT, load, validator


def verify_connection_link():
    library = ctypes.CDLL(str(ROOT / ('target/debug/libhomeverb_ffi.dylib' if sys.platform == 'darwin' else 'target/debug/libhomeverb_ffi.so')))
    library.homeverb_open.argtypes = [ctypes.c_char_p]
    library.homeverb_open.restype = ctypes.c_void_p
    for name in ['homeverb_call', 'homeverb_prepare_connection_link', 'homeverb_confirm_connection_link']:
        function = getattr(library, name)
        function.argtypes = [ctypes.c_uint64, ctypes.c_char_p]
        function.restype = ctypes.c_void_p
    library.homeverb_discard_connection_link.argtypes = [ctypes.c_uint64, ctypes.c_uint64]
    library.homeverb_discard_connection_link.restype = ctypes.c_bool
    library.homeverb_close.argtypes = [ctypes.c_uint64]
    library.homeverb_close.restype = None
    library.homeverb_set_foreground.argtypes = [ctypes.c_uint64, ctypes.c_bool]
    library.homeverb_set_foreground.restype = None
    library.homeverb_string_free.argtypes = [ctypes.c_void_p]
    library.homeverb_string_free.restype = None
    contract = load('packages/native-contract/connection-link.json')
    examples = load('packages/native-contract/connection-link-examples.json')
    assert contract['api_version'] == examples['api_version'] == load('packages/native-contract/api.json')['api_version']
    assert contract['limits'] == {'max_pending_links': 8, 'link_ttl_ms': 60000}
    for name in contract['functions']:
        assert getattr(library, name)
    shapes = {name: validator('native-connection-link', name) for name in
              ['prepare', 'confirm', 'prepare_result', 'confirm_result']}
    for name, shape in shapes.items():
        shape.validate(examples[name])

    def decode(pointer):
        assert pointer
        try:
            return json.loads(ctypes.string_at(pointer))
        finally:
            library.homeverb_string_free(pointer)

    def invoke(handle, name, value):
        raw = json.dumps(value).encode() if value is not None else None
        result = decode(getattr(library, f'homeverb_{name}_connection_link')(handle, raw))
        shapes[f'{name}_result'].validate(result)
        return result

    for adapter in ['homeverb.mock', 'midea.meiju']:
        with tempfile.TemporaryDirectory(prefix='homeverb-native-link-') as directory:
            opened = decode(library.homeverb_open(json.dumps({
                'schema_version': '1.0', 'data_dir': directory, 'allow_mock_writes': False,
                'adapter': adapter
            }).encode()))
            assert opened['error'] is None, opened
            handle = opened['data']['handle']
            try:
                for name in ['prepare', 'confirm']:
                    for value in [None, {}, *[{**examples[name], field: 'FORBIDDEN'} for field in
                                             ['confirmed', 'principal', 'endpoint', 'token', 'appliance_id']]]:
                        assert not shapes[name].is_valid(value)
                        assert invoke(handle, name, value)['error']['code'] == 'INVALID_ARGUMENT'
                assert invoke(handle, 'prepare', examples['prepare'])['error']['code'] == 'DEVICE_NOT_FOUND'
                descriptor = copy.deepcopy(examples['prepare_result']['data']['descriptor'])
                descriptor['device']['connections'][0]['adapter'] = adapter
                prepare = copy.deepcopy(examples['prepare'])
                prepare['connection']['adapter'] = adapter
                with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                    database.execute('INSERT INTO devices (id, descriptor) VALUES (?, ?)',
                                     (descriptor['device']['id'], json.dumps(descriptor)))
                assert invoke(handle, 'prepare', prepare)['error']['code'] == 'CONNECTION_LINK_UNSUPPORTED'
                assert invoke(handle, 'confirm', examples['confirm'])['error']['code'] == 'CONNECTION_LINK_INVALID'
                assert not library.homeverb_discard_connection_link(handle, 1)
                for method in ['prepare_connection_link', 'confirm_connection_link', 'discard_connection_link']:
                    request = {'schema_version': '1.0', 'operation_id': 'ordinary-link', 'timeout_ms': 1000,
                               'request': {'method': method}}
                    assert not validator('native-request').is_valid(request)
                    assert decode(library.homeverb_call(handle, json.dumps(request).encode()))['error']['code'] == 'INVALID_ARGUMENT'
                with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                    assert json.loads(database.execute('SELECT descriptor FROM devices').fetchone()[0]) == descriptor
                library.homeverb_set_foreground(handle, False)
                assert invoke(handle, 'prepare', prepare)['error']['code'] == 'HOST_BACKGROUND'
            finally:
                library.homeverb_close(handle)
            for name in ['prepare', 'confirm']:
                assert invoke(handle, name, examples[name])['error']['code'] == 'HOST_CLOSED'
            assert not library.homeverb_discard_connection_link(handle, 1)
    print('PASS: three connection-link C exports, closed metadata, ordinary-dispatch and Mock/Meiju refusal. Successful links use the fictional opted-in Session adapter; no credentials or network traffic.')


if __name__ == '__main__':
    verify_connection_link()
