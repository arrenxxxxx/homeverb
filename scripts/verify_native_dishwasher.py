#!/usr/bin/env python3
"""Check native dishwasher startup and trusted approval admission without any account."""

import copy
import ctypes
import json
from pathlib import Path
import sqlite3
import tempfile
import uuid

from native_contract import ROOT, load, validator


def verify_dishwasher():
    library = ctypes.CDLL(str(ROOT / 'target/debug/libhomeverb_ffi.so'))
    library.homeverb_open.argtypes = [ctypes.c_char_p]
    library.homeverb_open.restype = ctypes.c_void_p
    for name in ['homeverb_call', 'homeverb_prepare_approval', 'homeverb_execute_approved']:
        getattr(library, name).argtypes = [ctypes.c_uint64, ctypes.c_char_p]
        getattr(library, name).restype = ctypes.c_void_p
    library.homeverb_string_free.argtypes = [ctypes.c_void_p]
    library.homeverb_string_free.restype = None
    library.homeverb_close.argtypes = [ctypes.c_uint64]
    library.homeverb_close.restype = None
    shape = validator('native-open')
    identifier = 'abcdef00-1234-4567-89ab-123456abcdef'
    arguments = {'program': 'eco_wash', 'additional': 0}
    grant = {'device_id': identifier, 'arguments': arguments}

    def decode(pointer):
        assert pointer
        try:
            return json.loads(ctypes.string_at(pointer))
        finally:
            library.homeverb_string_free(pointer)

    def invoke(name, handle, payload):
        return decode(getattr(library, name)(handle, json.dumps(payload).encode()))

    def error(value, code):
        assert value['error']['code'] == code, value

    def command():
        c = load('examples/command.meiju-dishwasher.json')
        c.update(device_id=identifier, request_id=str(uuid.uuid4()))
        return c

    def call(handle, method, **fields):
        return invoke('homeverb_call', handle, {'schema_version': '1.0',
                      'operation_id': str(uuid.uuid4()), 'timeout_ms': 10000,
                      'request': {'method': method, **fields}})

    with tempfile.TemporaryDirectory(prefix='homeverb-native-dishwasher-') as root:
        base = {'schema_version': '1.0', 'adapter': 'midea.meiju',
                'allow_mock_writes': False, 'meiju_dishwasher_grants': [grant]}
        invalid = []
        for args in [{}, None, {'program': 'neutral_gear'}, {'program': 'eco'},
                     {'program': 'eco_wash', 'additional': None},
                     {'program': 'eco_wash', 'additional': 1},
                     {'program': 'eco_wash', 'additional': True},
                     {'program': 'eco_wash', 'additional': 9.5},
                     {'program': 'eco_wash', 'approved': True}]:
            bad = copy.deepcopy(base)
            bad['meiju_dishwasher_grants'][0]['arguments'] = args
            invalid.append(bad)
        for grants in [[grant, grant], [grant] * 65, None, [{'device_id': '*', 'arguments': arguments}],
                       [{**grant, 'principal': 'admin'}]]:
            invalid.append({**base, 'meiju_dishwasher_grants': grants})
        invalid.append({**base, 'adapter': 'homeverb.mock'})
        omitted = dict(base)
        del omitted['adapter']
        invalid.append(omitted)
        for index, config in enumerate(invalid):
            path = Path(root) / f'refused-{index}'
            config['data_dir'] = str(path)
            assert not shape.is_valid(config), config
            error(decode(library.homeverb_open(json.dumps(config).encode())), 'INVALID_ARGUMENT')
            assert not path.exists()

        for granted in [False, True]:
            path = Path(root) / str(granted)
            config = {**base, 'data_dir': str(path), 'meiju_dishwasher_grants': [grant] if granted else []}
            shape.validate(config)
            opened = decode(library.homeverb_open(json.dumps(config).encode()))
            assert opened['error'] is None, opened
            handle = opened['data']['handle']
            try:
                assert call(handle, 'list_accounts')['data'] == []
                # Fictional public descriptor only. No account row or credential exists.
                descriptor = {'device': {'schema_version': '1.0', 'id': identifier,
                    'alias': 'Synthetic dishwasher', 'brand': 'Unknown',
                    'model': 'GX1200Max \u5c0a\u4eab\u7248', 'support_status': 'experimental',
                    'components': [{'id': 'main', 'capabilities': ['vendor.midea.dishwasher']}],
                    'connections': [{'id': str(uuid.uuid4()), 'adapter': 'midea.meiju',
                        'transport': 'cloud', 'account_id': str(uuid.uuid4()), 'identity_ref': str(uuid.uuid4())}]},
                    'capabilities': [load('examples/capability.meiju-dishwasher.json')]}
                with sqlite3.connect(path / 'homeverb.sqlite3') as database:
                    database.execute('INSERT INTO devices VALUES (?,?)', (identifier, json.dumps(descriptor)))
                for method in ['plan', 'execute']:
                    error(call(handle, method, command=command()), 'POLICY_DENIED')
                c = command()
                prepared = invoke('homeverb_prepare_approval', handle, {'schema_version': '1.0',
                    'operation_id': str(uuid.uuid4()), 'command': c})
                validator('native-approval', 'prepare_result').validate(prepared)
                if not granted:
                    error(prepared, 'POLICY_DENIED')
                    continue
                assert prepared['data']['command'] == c, prepared
                for change in [{'program': 'eco_wash', 'additional': 9}, {'program': 'strong_wash', 'additional': 0}]:
                    denied = command()
                    denied['arguments'] = change
                    error(invoke('homeverb_prepare_approval', handle, {'schema_version': '1.0',
                        'operation_id': str(uuid.uuid4()), 'command': denied}), 'POLICY_DENIED')
                result = invoke('homeverb_execute_approved', handle, {'schema_version': '1.0',
                    'operation_id': str(uuid.uuid4()), 'approval_id': prepared['data']['approval_id']})
                validator('native-approval', 'execute_result').validate(result)
                error(result, 'ACCOUNT_NOT_FOUND')
                assert result['status'] == 'failed' and result['attempts'][0]['phase'] == 'not_sent'
                assert call(handle, 'get_execution', request_id=c['request_id'])['data'] == result
            finally:
                library.homeverb_close(handle)
    print('PASS: native dishwasher startup, exact grants and separate confirmation admission; no accounts, credentials or provider traffic.')


if __name__ == '__main__':
    verify_dishwasher()
