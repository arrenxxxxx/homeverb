#!/usr/bin/env python3
"""Verify trusted approval exports with fictional appliances and temporary profiles."""

import ctypes
import json
import sqlite3
import tempfile
import uuid
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from native_contract import ROOT, load, validator


def verify_approvals():
    library = ctypes.CDLL(str(ROOT / 'target/debug/libhomeverb_ffi.so'))
    for name in ['homeverb_call', 'homeverb_prepare_approval', 'homeverb_execute_approved']:
        function = getattr(library, name)
        function.argtypes = [ctypes.c_uint64, ctypes.c_char_p]
        function.restype = ctypes.c_void_p
    library.homeverb_open.argtypes = [ctypes.c_char_p]
    library.homeverb_open.restype = ctypes.c_void_p
    library.homeverb_string_free.argtypes = [ctypes.c_void_p]
    library.homeverb_string_free.restype = None
    library.homeverb_discard_approval.argtypes = [ctypes.c_uint64, ctypes.c_uint64]
    library.homeverb_discard_approval.restype = ctypes.c_bool
    library.homeverb_set_foreground.argtypes = [ctypes.c_uint64, ctypes.c_bool]
    library.homeverb_set_foreground.restype = None
    library.homeverb_close.argtypes = [ctypes.c_uint64]
    library.homeverb_close.restype = None
    shapes = {name: validator('native-approval', name) for name in
              ['prepare', 'execute', 'prepare_result', 'execute_result']}
    contract = load('packages/native-contract/approval.json')
    assert contract['api_version'] == load('packages/native-contract/api.json')['api_version']
    assert contract['limits'] == {'max_pending_approvals': 8, 'approval_ttl_ms': 60000}
    for name in contract['functions']:
        assert getattr(library, name)

    def decode(pointer):
        assert pointer
        try:
            return json.loads(ctypes.string_at(pointer))
        finally:
            library.homeverb_string_free(pointer)

    def invoke(name, handle, request):
        raw = json.dumps(request).encode() if request is not None else None
        result = decode(getattr(library, name)(handle, raw))
        shapes['prepare_result' if name == 'homeverb_prepare_approval' else 'execute_result'].validate(result)
        return result

    def call(handle, method, **fields):
        request = {'schema_version': '1.0', 'operation_id': str(uuid.uuid4()),
                   'timeout_ms': 10000, 'request': {'method': method, **fields}}
        return decode(library.homeverb_call(handle, json.dumps(request).encode()))

    def command():
        value = load('examples/command.approved-cycle.json')
        value.update(device_id='dev-mock-ac', request_id=str(uuid.uuid4()))
        return value

    def prepare_input():
        return {'schema_version': '1.0', 'operation_id': str(uuid.uuid4()), 'command': command()}

    def execute_input(identifier):
        return {'schema_version': '1.0', 'operation_id': str(uuid.uuid4()), 'approval_id': identifier}

    def prepare(handle):
        request = prepare_input()
        shapes['prepare'].validate(request)
        result = invoke('homeverb_prepare_approval', handle, request)
        assert result['error'] is None, result
        assert result['data']['command'] == request['command']
        assert result['data']['action']['risk'] == 'elevated'
        assert result['data']['connection']['transport'] == 'cloud'
        return result['data']

    def execute(handle, identifier):
        request = execute_input(identifier)
        shapes['execute'].validate(request)
        return invoke('homeverb_execute_approved', handle, request)

    def error(result, code):
        assert result['error']['code'] == code, result

    with tempfile.TemporaryDirectory(prefix='homeverb-native-approval-') as root:
        handles = []

        def open_host(name, scenario='approval-required', allow=True):
            config = {'schema_version': '1.0', 'data_dir': str(Path(root) / name),
                      'scenario': scenario, 'allow_mock_writes': allow}
            validator('native-open').validate(config)
            opened = decode(library.homeverb_open(json.dumps(config).encode()))
            assert opened['error'] is None, opened
            handle = opened['data']['handle']
            handles.append(handle)
            assert call(handle, 'discover')['error'] is None
            return handle

        try:
            forbidden_path = Path(root) / 'meiju-scenario-refusal'
            forbidden = {'schema_version': '1.0', 'data_dir': str(forbidden_path),
                         'adapter': 'midea.meiju', 'scenario': 'approval-required',
                         'allow_mock_writes': False}
            assert not validator('native-open').is_valid(forbidden)
            error(decode(library.homeverb_open(json.dumps(forbidden).encode())), 'INVALID_ARGUMENT')
            assert not forbidden_path.exists()
            handle = open_host('first')
            other = open_host('other')
            for method in ['plan', 'execute']:
                error(call(handle, method, command=command()), 'POLICY_DENIED')
            preview = prepare(handle)
            error(execute(other, preview['approval_id']), 'APPROVAL_INVALID')
            assert not library.homeverb_discard_approval(other, preview['approval_id'])
            assert call(handle, 'get_execution', request_id=preview['command']['request_id'])['data'] is None
            result = execute(handle, preview['approval_id'])
            assert result['status'] == 'accepted', result
            error(result, 'VERIFICATION_PENDING')
            assert result['warnings'] == ['User approval was supplied for this exact command.']
            assert call(handle, 'get_execution', request_id=preview['command']['request_id'])['data'] == result
            error(execute(handle, preview['approval_id']), 'APPROVAL_INVALID')

            identifier = prepare(handle)['approval_id']
            with ThreadPoolExecutor(max_workers=2) as pool:
                results = list(pool.map(lambda _: execute(handle, identifier), range(2)))
            assert sum(value.get('status') == 'accepted' for value in results) == 1
            assert sum(value['error']['code'] == 'APPROVAL_INVALID' for value in results) == 1

            identifier = prepare(handle)['approval_id']
            assert library.homeverb_discard_approval(handle, identifier)
            assert not library.homeverb_discard_approval(handle, identifier)
            error(execute(handle, identifier), 'APPROVAL_INVALID')
            ids = [prepare(handle)['approval_id'] for _ in range(8)]
            error(invoke('homeverb_prepare_approval', handle, prepare_input()), 'RESOURCE_BUSY')
            assert library.homeverb_discard_approval(handle, ids.pop())
            ids.append(prepare(handle)['approval_id'])
            for identifier in ids:
                assert library.homeverb_discard_approval(handle, identifier)

            for field, value in [('schema_version', '2.0'), ('operation_id', '')]:
                identifier = prepare(handle)['approval_id']
                bad = execute_input(identifier)
                bad[field] = value
                assert not shapes['execute'].is_valid(bad)
                error(invoke('homeverb_execute_approved', handle, bad), 'INVALID_ARGUMENT')
                error(execute(handle, identifier), 'APPROVAL_INVALID')
            identifier = prepare(handle)['approval_id']
            for field, value in [('approved', True), ('command', command()), ('principal', 'admin'),
                                 ('approval_id', 0), ('approval_id', 2**64), ('method', 'execute')]:
                bad = execute_input(identifier)
                bad[field] = value
                assert not shapes['execute'].is_valid(bad)
                error(invoke('homeverb_execute_approved', handle, bad), 'INVALID_ARGUMENT')
            error(invoke('homeverb_execute_approved', handle, None), 'INVALID_ARGUMENT')
            assert execute(handle, identifier)['status'] == 'accepted'
            for name in ['homeverb_prepare_approval', 'homeverb_execute_approved']:
                error(invoke(name, handle, None), 'INVALID_ARGUMENT')
                error(call(handle, name), 'INVALID_ARGUMENT')
            bad = prepare_input()
            bad['approved'] = True
            assert not shapes['prepare'].is_valid(bad)
            error(invoke('homeverb_prepare_approval', handle, bad), 'INVALID_ARGUMENT')

            identifier = prepare(handle)['approval_id']
            library.homeverb_set_foreground(handle, False)
            error(invoke('homeverb_prepare_approval', handle, prepare_input()), 'HOST_BACKGROUND')
            error(execute(handle, identifier), 'HOST_BACKGROUND')
            library.homeverb_set_foreground(handle, True)
            error(execute(handle, identifier), 'APPROVAL_INVALID')

            for name, scenario, allow in [('read-only', 'approval-required', False), ('normal', 'normal', True)]:
                denied = open_host(name, scenario, allow)
                result = invoke('homeverb_prepare_approval', denied, prepare_input())
                assert result['data'] is None and result['error'] is not None, result

            # Mutate fictional cached metadata only; the immutable preview must fail closed.
            preview = prepare(handle)
            with sqlite3.connect(Path(root) / 'first' / 'homeverb.sqlite3') as database:
                row = database.execute('SELECT descriptor FROM devices WHERE id = ?', ('dev-mock-ac',)).fetchone()
                descriptor = json.loads(row[0])
                cycle = next(c for c in descriptor['capabilities'] if c['id'] == 'vendor.example.cycle')
                cycle['revision'] += 1
                database.execute('UPDATE devices SET descriptor = ? WHERE id = ?', (json.dumps(descriptor), 'dev-mock-ac'))
            result = execute(handle, preview['approval_id'])
            error(result, 'APPROVAL_INVALID')
            assert result['attempts'] == [] and result['status'] == 'failed'
            with sqlite3.connect(Path(root) / 'first' / 'homeverb.sqlite3') as database:
                cycle['actions']['start']['risk'] = 'low'
                database.execute('UPDATE devices SET descriptor = ? WHERE id = ?', (json.dumps(descriptor), 'dev-mock-ac'))
            result = call(handle, 'execute', command=command())
            error(result, 'POLICY_DENIED')
            assert result['attempts'][0]['phase'] == 'not_sent'

            identifier = prepare(other)['approval_id']
            library.homeverb_close(other)
            error(execute(other, identifier), 'HOST_CLOSED')
            assert not library.homeverb_discard_approval(other, identifier)
        finally:
            for handle in handles:
                library.homeverb_close(handle)
    print('PASS: three trusted approval C exports; scope, single consumption, concurrency, capacity, lifecycle, metadata and authority guards; fictional data only.')


if __name__ == '__main__':
    verify_approvals()
