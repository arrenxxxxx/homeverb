#!/usr/bin/env python3
"""Exercise every published UI method through the built C ABI using fictional data."""

from __future__ import annotations

import argparse
import copy
import ctypes
import json
import sqlite3
from pathlib import Path
import tempfile
import threading
import time

from native_contract import ROOT, load, validator


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write-examples', action='store_true',
                        help='Replace fictional UI examples with this verified native run.')
    args = parser.parse_args()
    api = load('packages/native-contract/api.json')
    request_shapes = load('schemas/native-request.schema.json')['properties']['request']['oneOf']
    assert {shape['properties']['method']['const'] for shape in request_shapes} == set(api['methods'])
    request_schema = validator('native-request')
    open_schema = validator('native-open')
    responses = {name: validator('native-response', name) for name in [*api['methods'], 'open']}
    query_error = validator('native-response', 'query_error')
    info_example = {'schema_version': '1.0', 'data': copy.deepcopy(api), 'warnings': [], 'error': None}
    info_example['data']['api_version'] = '1.16.0'
    info_example['data']['methods']['future_query'] = {'response_kind': 'query'}
    responses['get_api_info'].validate(info_example)
    info_example['data']['api_version'] = '2.0.0'
    assert not responses['get_api_info'].is_valid(info_example)
    library = ctypes.CDLL(str(ROOT / 'target/debug/libhomeverb_ffi.so'))
    library.homeverb_open.argtypes = [ctypes.c_char_p]
    library.homeverb_open.restype = ctypes.c_void_p
    library.homeverb_call.argtypes = [ctypes.c_uint64, ctypes.c_char_p]
    library.homeverb_call.restype = ctypes.c_void_p
    library.homeverb_string_free.argtypes = [ctypes.c_void_p]
    library.homeverb_string_free.restype = None
    library.homeverb_cancel.argtypes = [ctypes.c_uint64, ctypes.c_char_p]
    library.homeverb_cancel.restype = ctypes.c_bool
    library.homeverb_set_foreground.argtypes = [ctypes.c_uint64, ctypes.c_bool]
    library.homeverb_set_foreground.restype = None
    library.homeverb_close.argtypes = [ctypes.c_uint64]
    library.homeverb_close.restype = None
    library.homeverb_connect_account.argtypes = [ctypes.c_uint64, ctypes.c_char_p,
                                                ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
                                                ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
    library.homeverb_connect_account.restype = ctypes.c_void_p
    connect_schema = validator('native-connect')
    connect_response = validator('native-response', 'connect_account')
    examples = []
    sequence = 0

    def decode(pointer):
        assert pointer, 'The native library returned a null output pointer.'
        try:
            return json.loads(ctypes.string_at(pointer))
        finally:
            library.homeverb_string_free(pointer)

    def raw(handle, request):
        return decode(library.homeverb_call(handle, json.dumps(request).encode()))

    def envelope(method, **fields):
        nonlocal sequence
        sequence += 1
        return {'schema_version': '1.0', 'operation_id': f'ui-{sequence}',
                'timeout_ms': 10000, 'request': {'method': method, **fields}}

    def call(handle, method, **fields):
        request = envelope(method, **fields)
        request_schema.validate(request)
        result = raw(handle, request)
        responses[method].validate(result)
        examples.append({'request': request, 'response': result})
        return result

    def open_host(directory, scenario='normal', allow=True, adapter=None):
        config = {'schema_version': '1.0', 'data_dir': directory,
                  'allow_mock_writes': allow, 'scenario': scenario}
        if adapter is not None:
            config['adapter'] = adapter
        open_schema.validate(config)
        result = decode(library.homeverb_open(json.dumps(config).encode()))
        responses['open'].validate(result)
        assert result['error'] is None, result
        assert result['data']['adapter'] == (adapter or 'homeverb.mock')
        return result['data']['handle']

    def command(identifier, **changes):
        value = {'schema_version': '1.0', 'request_id': identifier,
                 'device_id': 'dev-mock-ac', 'component_id': 'main',
                 'capability': 'power', 'action': 'set', 'arguments': {'state': True},
                 'connection': 'auto', 'timeout_ms': 10000, 'wait_for': 'verified'}
        value.update(changes)
        return value

    with tempfile.TemporaryDirectory(prefix='homeverb-ui-contract-') as directory:
        handle = open_host(directory)
        try:
            assert call(handle, 'get_api_info')['data'] == api
            assert call(handle, 'list_devices')['data'] == []
            assert call(handle, 'list_accounts')['data'] == []
            assert call(handle, 'get_account', account_id='00000000-0000-4000-8000-000000000001')['error']['code'] == 'AUTH_FLOW_UNSUPPORTED'
            assert call(handle, 'inspect_account_inventory', account_id='00000000-0000-4000-8000-000000000001')['error']['code'] == 'AUTH_FLOW_UNSUPPORTED'
            assert call(handle, 'rename_account', account_id='00000000-0000-4000-8000-000000000001', label='Family appliances')['error']['code'] == 'AUTH_FLOW_UNSUPPORTED'
            assert call(handle, 'remove_account', account_id='00000000-0000-4000-8000-000000000001', expected_device_ids=[])['error']['code'] == 'AUTH_FLOW_UNSUPPORTED'
            assert call(handle, 'refresh_account', account_id='00000000-0000-4000-8000-000000000001')['error']['code'] == 'AUTH_FLOW_UNSUPPORTED'
            assert call(handle, 'recover_account', account_id='00000000-0000-4000-8000-000000000001')['error']['code'] == 'AUTH_FLOW_UNSUPPORTED'
            assert call(handle, 'list_device_cards')['data'] == []
            assert call(handle, 'list_rooms')['data'] == []
            devices = call(handle, 'discover')['data']
            assert len(devices) == 1
            call(handle, 'describe_device', device=devices[0]['id'])
            call(handle, 'read_state', device='dev-mock-ac', component='main', connection='auto', cached=False)
            call(handle, 'read_state', device='dev-mock-ac', component='main', connection='auto', cached=True)
            room = call(handle, 'create_room', name='Bedroom')['data']
            call(handle, 'rename_room', room_id=room['id'], name='Primary Bedroom')
            call(handle, 'reorder_rooms', room_ids=[room['id']])
            call(handle, 'set_device_preferences', device_id='dev-mock-ac',
                 preferences={'alias': 'Quiet AC', 'room_id': room['id'], 'favorite': True})
            call(handle, 'assign_devices_to_room', assignment={'device_ids': ['dev-mock-ac'], 'room_id': None})
            call(handle, 'reorder_devices', device_ids=['dev-mock-ac'])
            assert call(handle, 'list_device_cards')['data'][0]['preferences']['alias'] == 'Quiet AC'
            deleted = call(handle, 'delete_room', room_id=room['id'])
            assert deleted['data'] is None and deleted['error'] is None
            call(handle, 'plan', command=command('plan-example'))
            composite = command('compound-example', capability='climate', arguments={'mode': 'heat', 'target_temperature': 25})
            assert len(call(handle, 'plan', command=composite)['data']['items']) == 2
            assert call(handle, 'execute', command=composite)['status'] == 'verified'
            batch = {'schema_version': '1.0', 'commands': [command('batch-example')]}
            call(handle, 'plan_batch', batch=batch)
            assert call(handle, 'execute_batch', batch=batch)['status'] == 'verified'
            call(handle, 'get_execution', request_id='compound-example')
            call(handle, 'recent_executions')
            call(handle, 'device_executions', device_id='dev-mock-ac')
            call(handle, 'diagnostics')
            assert call(handle, 'get_execution', request_id='missing')['error']['code'] == 'REQUEST_NOT_FOUND'

            invalid = command('bad-budget', timeout_ms=9999)
            assert call(handle, 'execute', command=invalid)['error']['code'] == 'INVALID_ARGUMENT'
            forbidden = command('unsupported-example', capability='vendor.example.unsupported')
            result = call(handle, 'execute', command=forbidden)
            assert result['status'] == 'failed' and result['attempts'] == []
            bad_batch = {'schema_version': '1.0', 'commands': [forbidden]}
            query_error.validate(call(handle, 'execute_batch', batch=bad_batch))

            assert call(handle, 'clear_cached_state')['data'] == {'removed_snapshots': 1}
            assert raw(handle, envelope('read_state', device='dev-mock-ac', component='main', connection='auto', cached=True))['error']['code'] == 'CACHE_MISS'
            assert call(handle, 'clear_execution_history')['data'] == {'removed_receipts': 4}
            for request_id in ['compound-example', 'compound-example:0', 'compound-example:1', 'batch-example']:
                assert raw(handle, envelope('get_execution', request_id=request_id))['error']['code'] == 'REQUEST_RETIRED'
            assert raw(handle, envelope('execute', command=composite))['error']['code'] == 'REQUEST_RETIRED'

            # Reuse the validated input shapes, not a second handwritten method list.
            by_method = {item['request']['request']['method']: item['request'] for item in examples}
            assert set(by_method) == set(api['methods'])
            for method, valid in by_method.items():
                bad = copy.deepcopy(valid)
                bad['request']['principal'] = 'administrator'
                assert not request_schema.is_valid(bad), method
                rejected = raw(handle, bad)
                query_error.validate(rejected)
                assert rejected['error']['code'] == 'INVALID_ARGUMENT', method
            for field, value in [('schema_version', '9.0'), ('timeout_ms', 0), ('operation_id', ''), ('principal', 'administrator')]:
                bad = envelope('list_devices')
                bad[field] = value
                assert not request_schema.is_valid(bad)
                assert raw(handle, bad)['error']['code'] == 'INVALID_ARGUMENT'
            bad = envelope('set_device_preferences', device_id='dev-mock-ac', preferences={'favorite': False})
            assert not request_schema.is_valid(bad)
            assert raw(handle, bad)['error']['code'] == 'INVALID_ARGUMENT'
            assert raw(handle, envelope('start_authentication'))['error']['code'] == 'INVALID_ARGUMENT'
            # Typed trusted approvals must never become an ordinary JSON privilege path.
            for name in ['prepare_user_approval', 'execute_user_approved']:
                forged = envelope(name, command=load('examples/command.approved-cycle.json'))
                assert not request_schema.is_valid(forged)
                assert raw(handle, forged)['error']['code'] == 'INVALID_ARGUMENT'
            for field in ['approved', 'user_approved', 'approval_id']:
                forged = copy.deepcopy(by_method['execute'])
                forged['request']['command'][field] = True
                assert not request_schema.is_valid(forged)
                assert raw(handle, forged)['error']['code'] == 'INVALID_ARGUMENT'
            assert not library.homeverb_cancel(handle, b'absent-operation')
            library.homeverb_set_foreground(handle, False)
            assert call(handle, 'get_api_info')['error']['code'] == 'HOST_BACKGROUND'
            for method in ['clear_cached_state', 'clear_execution_history']:
                assert raw(handle, envelope(method))['error']['code'] == 'HOST_BACKGROUND'
            assert raw(handle, envelope('recover_account', account_id='00000000-0000-4000-8000-000000000001'))['error']['code'] == 'HOST_BACKGROUND'
            library.homeverb_set_foreground(handle, True)
            assert call(handle, 'get_api_info')['data'] == api
        finally:
            library.homeverb_close(handle)
        assert call(handle, 'list_devices')['error']['code'] == 'HOST_CLOSED'
        for method in ['clear_cached_state', 'clear_execution_history']:
            assert raw(handle, envelope(method))['error']['code'] == 'HOST_CLOSED'
        assert raw(handle, envelope('recover_account', account_id='00000000-0000-4000-8000-000000000001'))['error']['code'] == 'HOST_CLOSED'

    with tempfile.TemporaryDirectory(prefix='homeverb-ui-clear-history-') as directory:
        handle = open_host(directory, 'lost-ack')
        try:
            raw(handle, envelope('discover'))
            item = command('uncertain-cleared')
            assert raw(handle, envelope('execute', command=item))['status'] == 'unknown'
            with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                database.execute('UPDATE executions SET finalized=0')
            busy = raw(handle, envelope('clear_execution_history'))
            responses['clear_execution_history'].validate(busy)
            assert busy['error']['code'] == 'RESOURCE_BUSY'
            assert raw(handle, envelope('get_execution', request_id=item['request_id']))['data']['status'] == 'unknown'
            with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                database.execute('UPDATE executions SET finalized=1')
            cleared = raw(handle, envelope('clear_execution_history'))
            responses['clear_execution_history'].validate(cleared)
            assert cleared['data'] == {'removed_receipts': 1}
            for count in [-1, None, '1']:
                invalid = copy.deepcopy(cleared)
                invalid['data']['removed_receipts'] = count
                assert not responses['clear_execution_history'].is_valid(invalid)
        finally:
            library.homeverb_close(handle)
        handle = open_host(directory)
        try:
            replay = raw(handle, envelope('execute', command=item))
            responses['execute'].validate(replay)
            assert replay['error']['code'] == 'REQUEST_RETIRED' and replay['attempts'] == []
            assert raw(handle, envelope('recent_executions'))['data'] == []
        finally:
            library.homeverb_close(handle)

    # Obtain uncertainty examples from the real simulator path, not UI-invented success.
    for scenario, outcome in [('accept-only', 'accepted'), ('lost-ack', 'unknown'), ('reject', 'failed'), ('partial', 'partial')]:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-outcome-') as directory:
            handle = open_host(directory, scenario)
            try:
                call(handle, 'discover')
                item = command('scenario-' + scenario)
                if scenario == 'partial':
                    item.update(capability='climate', arguments={'mode': 'heat', 'target_temperature': 25})
                result = call(handle, 'execute', command=item)
                assert result['status'] == outcome, (scenario, result)
            finally:
                library.homeverb_close(handle)

    with tempfile.TemporaryDirectory(prefix='homeverb-ui-cancel-') as directory:
        handle = open_host(directory, 'slow-before-send')
        try:
            call(handle, 'discover')
            pending = envelope('execute', command=command('cancel-before-send'))
            request_schema.validate(pending)
            result = []
            thread = threading.Thread(target=lambda: result.append(raw(handle, pending)))
            thread.start()
            try:
                deadline = time.monotonic() + 2
                while time.monotonic() < deadline:
                    if library.homeverb_cancel(handle, pending['operation_id'].encode()):
                        break
                    time.sleep(0.005)
                else:
                    raise AssertionError('The operation was not registered for cancellation.')
            finally:
                thread.join(timeout=2)
            assert not thread.is_alive(), 'Cancellation did not finish within the acceptance budget.'
            responses['execute'].validate(result[0])
            assert result[0]['status'] == 'failed'
            assert result[0]['error']['code'] == 'CANCELLED'
            examples.append({'request': pending, 'response': result[0]})
        finally:
            library.homeverb_close(handle)

    # Empty real-host startup/discovery needs neither a credential nor vendor traffic.
    # Keep these cases out of fictional Mock device examples.
    with tempfile.TemporaryDirectory(prefix='homeverb-ui-meiju-') as directory:
        handle = open_host(directory, allow=False, adapter='midea.meiju')
        try:
            assert not (Path(directory) / 'mock-appliance.sqlite3').exists()
            for method in ['list_devices', 'list_device_cards', 'discover', 'list_accounts']:
                result = raw(handle, envelope(method))
                responses[method].validate(result)
                assert result['error'] is None and result['data'] == []
            inspection = envelope('inspect_account_inventory', account_id='00000000-0000-4000-8000-000000000001')
            result = raw(handle, inspection)
            responses['inspect_account_inventory'].validate(result)
            assert result['error']['code'] == 'ACCOUNT_NOT_FOUND'
            for field, value in [('listed', True), ('refresh', True), ('credential_ref', 'FORBIDDEN')]:
                bad = copy.deepcopy(inspection)
                bad['request'][field] = value
                assert not request_schema.is_valid(bad)
                assert raw(handle, bad)['error']['code'] == 'INVALID_ARGUMENT'
            library.homeverb_set_foreground(handle, False)
            assert raw(handle, inspection)['error']['code'] == 'HOST_BACKGROUND'
            result = raw(handle, envelope('discover'))
            query_error.validate(result)
            assert result['error']['code'] == 'HOST_BACKGROUND'
            metadata = {'schema_version': '1.0', 'operation_id': 'account-connect',
                        'timeout_ms': 1000, 'label': 'Synthetic account', 'account_id': None}
            connect_schema.validate(metadata)
            # A foreground gate ensures these fictional credentials can never reach a provider.
            def connect_check(expected, request=metadata, target=handle, login_length=None):
                login = bytearray(b'owner@example.test')
                password = bytearray(b'FICTIONAL-PASSWORD')
                login_buffer = (ctypes.c_uint8 * len(login)).from_buffer(login)
                password_buffer = (ctypes.c_uint8 * len(password)).from_buffer(password)
                try:
                    result = decode(library.homeverb_connect_account(
                        target, json.dumps(request).encode(), login_buffer,
                        len(login) if login_length is None else login_length,
                        password_buffer, len(password)))
                    connect_response.validate(result)
                    assert result['error']['code'] == expected, result
                    assert not any(password), 'The valid password buffer was not cleared.'
                    if login_length is None:
                        assert not any(login), 'The valid account buffer was not cleared.'
                    assert 'FICTIONAL' not in json.dumps(result)
                finally:
                    login[:] = bytes(len(login))
                    password[:] = bytes(len(password))
            connect_check('HOST_BACKGROUND')
            bad = {**metadata, 'password': 'FICTIONAL-REFUSED'}
            assert not connect_schema.is_valid(bad)
            connect_check('INVALID_ARGUMENT', request=bad)
            for field, value in [('confirmed', True), ('recover_pending', True),
                                 ('subject', 'FICTIONAL'), ('session', 'FICTIONAL')]:
                bad = {**metadata, 'account_id': '40000000-0000-4000-8000-000000000001',
                       field: value}
                assert not connect_schema.is_valid(bad)
                connect_check('INVALID_ARGUMENT', request=bad)
            connect_check('HOST_CLOSED', target=0)
            connect_check('MIDEA_LOGIN_INPUT_UNSUPPORTED', login_length=0)
            connect_check('MIDEA_LOGIN_INPUT_UNSUPPORTED', login_length=257)
        finally:
            library.homeverb_close(handle)
        assert raw(handle, envelope('list_devices'))['error']['code'] == 'HOST_CLOSED'
        assert raw(handle, inspection)['error']['code'] == 'HOST_CLOSED'

    with tempfile.TemporaryDirectory(prefix='homeverb-ui-mock-account-') as directory:
        handle = open_host(directory, allow=False)
        try:
            connect_check('AUTH_FLOW_UNSUPPORTED', target=handle)
        finally:
            library.homeverb_close(handle)

    # Local account metadata remains usable without creating any protected session.
    with tempfile.TemporaryDirectory(prefix='homeverb-ui-local-accounts-') as directory:
        account_id = '40000000-0000-4000-8000-000000000001'
        other_id = '40000000-0000-4000-8000-000000000002'
        reference = '50000000-0000-4000-8000-000000000001'
        handle = open_host(directory, allow=False, adapter='midea.meiju')
        try:
            with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                database.execute('INSERT INTO accounts VALUES (?, ?, ?, ?, ?, ?)',
                                 (account_id, 'midea.meiju', 'CN', 'Original', reference, 'pending'))
                for device_id, accounts in [('linked', [account_id, account_id]), ('other', [other_id])]:
                    device = load('examples/device.json')
                    device.update(id=device_id, support_status='unsupported', connections=[
                        {'id': f'route-{index}', 'adapter': 'midea.meiju', 'transport': 'cloud',
                         'account_id': selected, 'identity_ref': 'fictional-binding'}
                        for index, selected in enumerate(accounts)])
                    database.execute('INSERT INTO devices VALUES (?, ?)',
                                     (device_id, json.dumps({'device': device, 'capabilities': []})))
            details = raw(handle, envelope('get_account', account_id=account_id))
            responses['get_account'].validate(details)
            assert details['data']['device_ids'] == ['linked']
            assert details['data']['account']['authorization'] == 'not_checked'
            assert reference not in json.dumps(details)
            renamed = raw(handle, envelope('rename_account', account_id=account_id, label='  Family appliances  '))
            responses['rename_account'].validate(renamed)
            assert renamed['data']['label'] == 'Family appliances'
            assert renamed['data']['credential_state'] == 'pending'
            # Removing and foreign accounts refuse recovery before any keyring access.
            with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                database.execute("UPDATE accounts SET credential_state='removing' WHERE id=?", (account_id,))
            refused = raw(handle, envelope('recover_account', account_id=account_id))
            responses['recover_account'].validate(refused)
            assert refused['error']['code'] == 'ACCOUNT_REMOVAL_PENDING'
            with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                database.execute("UPDATE accounts SET credential_state='pending', region='US' WHERE id=?", (account_id,))
            assert raw(handle, envelope('recover_account', account_id=account_id))['error']['code'] == 'ACCOUNT_NOT_FOUND'
            with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                database.execute("UPDATE accounts SET region='CN' WHERE id=?", (account_id,))
            for field, value in [('confirmed', True), ('validated', True), ('session', 'FICTIONAL'), ('provider', 'midea.meiju')]:
                forged = envelope('recover_account', account_id=account_id, **{field: value})
                assert not request_schema.is_valid(forged)
                assert raw(handle, forged)['error']['code'] == 'INVALID_ARGUMENT'
            for fields in [{'label': ''}, {'label': ' '}, {'label': 'a' * 81}, {'label': 'a\nb'},
                           {'label': 'Denied', 'provider': 'other'}, {'label': 'Denied', 'confirmed': True}]:
                request = envelope('rename_account', account_id=account_id, **fields)
                assert raw(handle, request)['error']['code'] == 'INVALID_ARGUMENT'
            for method in ['get_account', 'rename_account']:
                fields = {'label': 'Denied'} if method == 'rename_account' else {}
                assert raw(handle, envelope(method, account_id=other_id, **fields))['error']['code'] == 'ACCOUNT_NOT_FOUND'
                assert raw(handle, envelope(method, account_id='not-a-uuid', **fields))['error']['code'] == 'INVALID_ARGUMENT'
            with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                database.execute("CREATE TRIGGER refuse_account_rename BEFORE UPDATE OF label ON accounts BEGIN SELECT RAISE(ABORT, 'synthetic'); END")
            failed = raw(handle, envelope('rename_account', account_id=account_id, label='Must not persist'))
            responses['rename_account'].validate(failed)
            assert failed['error']['code'] == 'STORAGE_ERROR'
            assert raw(handle, envelope('get_account', account_id=account_id))['data']['account']['label'] == 'Family appliances'
            assert raw(handle, envelope('remove_account', account_id=account_id, expected_device_ids=[]))['error']['code'] == 'ACCOUNT_SCOPE_CHANGED'
            with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                # Abort before intent is committed so the ABI test never opens a keyring.
                database.execute("CREATE TRIGGER refuse_removal_intent BEFORE UPDATE OF credential_state ON accounts BEGIN SELECT RAISE(ABORT, 'synthetic'); END")
            failed = raw(handle, envelope('remove_account', account_id=account_id, expected_device_ids=['linked']))
            responses['remove_account'].validate(failed)
            assert failed['error']['code'] == 'STORAGE_ERROR'
            with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                assert database.execute('SELECT COUNT(*) FROM account_removals').fetchone()[0] == 0
                assert database.execute('SELECT credential_state FROM accounts').fetchone()[0] == 'pending'
        finally:
            library.homeverb_close(handle)
        handle = open_host(directory, allow=False, adapter='midea.meiju')
        try:
            persisted = raw(handle, envelope('get_account', account_id=account_id))
            assert persisted['data']['account'] == renamed['data']
            assert persisted['data']['device_ids'] == ['linked']
        finally:
            library.homeverb_close(handle)

    with tempfile.TemporaryDirectory(prefix='homeverb-ui-removed-account-') as directory:
        handle = open_host(directory, allow=False, adapter='midea.meiju')
        try:
            account_id = '40000000-0000-4000-8000-000000000001'
            # A completed fictional ledger tests actual ABI replay without any credential access.
            record = {'account_id': account_id, 'provider': 'midea.meiju', 'region': 'CN',
                      'device_ids': ['removed-card'], 'completed': True}
            with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                database.execute('INSERT INTO account_removals VALUES (?, ?)', (account_id, json.dumps(record)))
            request = envelope('remove_account', account_id=account_id, expected_device_ids=['removed-card'])
            result = raw(handle, request)
            responses['remove_account'].validate(result)
            assert result['data']['status'] == 'removed'
            assert result['data']['remote_revocation'] == 'not_attempted'
            assert result['data']['error'] is None
            for field, value in [('expected_device_ids', ['removed-card', 'removed-card']),
                                 ('expected_device_ids', ['']), ('expected_device_ids', None),
                                 ('confirmed', True), ('revoke_remote', True)]:
                bad = copy.deepcopy(request)
                bad['request'][field] = value
                assert not request_schema.is_valid(bad)
                assert raw(handle, bad)['error']['code'] == 'INVALID_ARGUMENT'
            assert raw(handle, envelope('remove_account', account_id=account_id, expected_device_ids=[]))['error']['code'] == 'ACCOUNT_SCOPE_CHANGED'
            library.homeverb_set_foreground(handle, False)
            assert raw(handle, request)['error']['code'] == 'HOST_BACKGROUND'
        finally:
            library.homeverb_close(handle)
        assert raw(handle, request)['error']['code'] == 'HOST_CLOSED'

    for fields in [
        {'adapter': 'unknown'}, {'adapter': None},
        {'adapter': 'midea.meiju', 'allow_mock_writes': True},
        {'adapter': 'midea.meiju', 'scenario': 'lost-ack'},
        {'adapter': 'midea.meiju', 'password': 'FICTIONAL-REFUSED'},
    ]:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-invalid-start-') as directory:
            private = Path(directory) / 'must-not-create'
            config = {'schema_version': '1.0', 'data_dir': str(private),
                      'allow_mock_writes': False, **fields}
            assert not open_schema.is_valid(config), config
            result = decode(library.homeverb_open(json.dumps(config).encode()))
            query_error.validate(result)
            assert result['error']['code'] == 'INVALID_ARGUMENT'
            assert not private.exists(), 'Invalid startup must fail before profile creation.'

    grant_id = '30000000-0000-4000-8000-000000000001'
    grant = {'device_id': grant_id, 'state': False}
    for grants in [[], [grant], [grant, {'device_id': grant_id, 'state': True}]]:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-power-grants-') as directory:
            config = {'schema_version': '1.0', 'data_dir': directory, 'adapter': 'midea.meiju',
                      'allow_mock_writes': False, 'meiju_power_grants': grants}
            open_schema.validate(config)
            result = decode(library.homeverb_open(json.dumps(config).encode()))
            responses['open'].validate(result)
            assert result['error'] is None, result
            handle = result['data']['handle']
            try:
                info = raw(handle, envelope('get_api_info'))
                assert info['data']['api_version'] == api['api_version']
                assert raw(handle, envelope('list_accounts'))['data'] == []
                assert raw(handle, envelope('discover'))['data'] == []
                # Fictional metadata makes the actual ABI policy observable without secrets or sends.
                device = load('examples/device.json')
                device.update(id=grant_id, model='KFR-35G/WF2', support_status='experimental',
                              components=[{'id': 'main', 'capabilities': ['power']}],
                              connections=[{'id': 'fictional-cloud', 'adapter': 'midea.meiju',
                                            'transport': 'cloud', 'account_id': '40000000-0000-4000-8000-000000000001',
                                            'identity_ref': 'fictional-binding'}])
                power = {'schema_version': '1.0', 'id': 'power', 'component_id': 'main', 'revision': 2,
                         'properties': {'state': {'type': 'boolean', 'readable': True, 'writable': True}},
                         'actions': {'set': {'parameters': {'type': 'object', 'properties': {'state': {'type': 'boolean'}},
                                                           'required': ['state'], 'additionalProperties': False},
                                             'transports': ['cloud'], 'idempotency': 'set_value', 'risk': 'low',
                                             'atomic': False, 'verification': 'unavailable'}}, 'events': {}}
                with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                    database.execute('INSERT INTO devices (id, descriptor) VALUES (?, ?)',
                                     (grant_id, json.dumps({'device': device, 'capabilities': [power]})))
                for state in [False, True]:
                    plan = raw(handle, envelope('plan', command=command('native-power-plan', device_id=grant_id,
                                                                        arguments={'state': state}, connection='cloud')))
                    responses['plan'].validate(plan)
                    permitted = any(item['state'] == state for item in grants)
                    assert (plan['error'] is None) == permitted
                    if permitted:
                        assert plan['data']['transports'] == ['cloud']
                    else:
                        assert plan['error']['code'] == 'POLICY_DENIED'
                forged = envelope('list_devices')
                forged['request']['meiju_power_grants'] = [grant]
                assert not request_schema.is_valid(forged)
                assert raw(handle, forged)['error']['code'] == 'INVALID_ARGUMENT'
            finally:
                library.homeverb_close(handle)

    for grants in [None, True, {}, [grant, grant], [grant] * 65,
                   [{'device_id': '*', 'state': False}],
                   [{'device_id': grant_id}], [{'device_id': grant_id, 'state': 'false'}],
                   [{'device_id': grant_id, 'state': False, 'confirmed': True}],
                   [{'device_id': '00000000-0000-0000-0000-000000000000', 'state': False}],
                   [{'device_id': grant_id.replace('-', ''), 'state': False}]]:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-bad-grants-') as directory:
            private = Path(directory) / 'must-not-create'
            config = {'schema_version': '1.0', 'data_dir': str(private), 'adapter': 'midea.meiju',
                      'allow_mock_writes': False, 'meiju_power_grants': grants}
            assert not open_schema.is_valid(config)
            assert decode(library.homeverb_open(json.dumps(config).encode()))['error']['code'] == 'INVALID_ARGUMENT'
            assert not private.exists()
    for adapter in [None, 'homeverb.mock']:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-mock-grants-') as directory:
            private = Path(directory) / 'must-not-create'
            config = {'schema_version': '1.0', 'data_dir': str(private),
                      'allow_mock_writes': False, 'meiju_power_grants': [grant]}
            if adapter:
                config['adapter'] = adapter
            assert not open_schema.is_valid(config)
            assert decode(library.homeverb_open(json.dumps(config).encode()))['error']['code'] == 'INVALID_ARGUMENT'
            assert not private.exists()

    fan_grant = {'device_id': grant_id, 'wind_speed': 40}
    for grants in [[], [fan_grant], [fan_grant, {'device_id': grant_id, 'wind_speed': 20.0}]]:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-fan-grants-') as directory:
            config = {'schema_version': '1.0', 'data_dir': directory, 'adapter': 'midea.meiju',
                      'allow_mock_writes': False, 'meiju_fan_grants': grants, 'meiju_power_grants': [grant]}
            open_schema.validate(config)
            result = decode(library.homeverb_open(json.dumps(config).encode()))
            assert result['error'] is None, result
            handle = result['data']['handle']
            try:
                assert raw(handle, envelope('list_accounts'))['data'] == []
                assert raw(handle, envelope('discover'))['data'] == []
                device = load('examples/device.json')
                device.update(id=grant_id, model='KFR-35G/WF2', support_status='experimental',
                              components=[{'id': 'main', 'capabilities': ['power', 'vendor.midea.ac']}],
                              connections=[{'id': 'fictional-cloud', 'adapter': 'midea.meiju',
                                            'transport': 'cloud', 'account_id': '40000000-0000-4000-8000-000000000001',
                                            'identity_ref': 'fictional-binding'}])
                fan = load('examples/capability.meiju-fan.json')
                with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                    database.execute('INSERT INTO devices (id, descriptor) VALUES (?, ?)',
                                     (grant_id, json.dumps({'device': device, 'capabilities': [power, fan]})))
                for speed in [20, 40, 102]:
                    request = command('native-fan-plan', device_id=grant_id, capability='vendor.midea.ac',
                                      action='set_fan_speed', arguments={'wind_speed': speed}, connection='cloud')
                    plan = raw(handle, envelope('plan', command=request))
                    responses['plan'].validate(plan)
                    permitted = any(item['wind_speed'] == speed for item in grants)
                    assert (plan['error'] is None) == permitted
                    if permitted:
                        assert plan['data']['transports'] == ['cloud']
                    else:
                        assert plan['error']['code'] == 'POLICY_DENIED'
                for state in [False, True]:
                    plan = raw(handle, envelope('plan', command=command('independent-power', device_id=grant_id,
                                                                        arguments={'state': state})))
                    assert (plan['error'] is None) == (not state)
                forged = envelope('list_devices')
                forged['request']['meiju_fan_grants'] = [fan_grant]
                assert not request_schema.is_valid(forged)
                assert raw(handle, forged)['error']['code'] == 'INVALID_ARGUMENT'
            finally:
                library.homeverb_close(handle)

    bad_fan_grants = [None, True, {}, [fan_grant, fan_grant], [fan_grant] * 65,
                      [{'device_id': '*', 'wind_speed': 40}], [{'device_id': grant_id}],
                      [{'device_id': grant_id, 'wind_speed': 40, 'confirmed': True}]]
    bad_fan_grants.extend([{'device_id': grant_id, 'wind_speed': speed}]
                         for speed in [None, True, '40', -1, 0, 41, 40.5, 101, 103, 256])
    for grants in bad_fan_grants:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-bad-fan-') as directory:
            private = Path(directory) / 'must-not-create'
            config = {'schema_version': '1.0', 'data_dir': str(private), 'adapter': 'midea.meiju',
                      'allow_mock_writes': False, 'meiju_fan_grants': grants}
            assert not open_schema.is_valid(config), config
            assert decode(library.homeverb_open(json.dumps(config).encode()))['error']['code'] == 'INVALID_ARGUMENT'
            assert not private.exists()
    for adapter in [None, 'homeverb.mock']:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-mock-fan-') as directory:
            private = Path(directory) / 'must-not-create'
            config = {'schema_version': '1.0', 'data_dir': str(private), 'allow_mock_writes': False,
                      'meiju_fan_grants': [fan_grant]}
            if adapter:
                config['adapter'] = adapter
            assert not open_schema.is_valid(config)
            assert decode(library.homeverb_open(json.dumps(config).encode()))['error']['code'] == 'INVALID_ARGUMENT'
            assert not private.exists()

    mode_grant = {'device_id':grant_id, 'mode':'cool'}
    for grants in [[], [mode_grant]]:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-mode-grants-') as directory:
            config = {'schema_version':'1.0', 'data_dir':directory, 'adapter':'midea.meiju',
                      'allow_mock_writes':False, 'meiju_mode_grants':grants}
            open_schema.validate(config)
            result = decode(library.homeverb_open(json.dumps(config).encode()))
            assert result['error'] is None, result
            handle = result['data']['handle']
            try:
                assert raw(handle, envelope('list_accounts'))['data'] == []
                assert raw(handle, envelope('discover'))['data'] == []
                device = load('examples/device.json')
                device.update(id=grant_id, model='KFR-35G/WF2', support_status='experimental',
                              components=[{'id': 'main', 'capabilities': ['power', 'vendor.midea.ac']}],
                              connections=[{'id': 'fictional-cloud', 'adapter': 'midea.meiju',
                                            'transport': 'cloud', 'account_id': '40000000-0000-4000-8000-000000000001',
                                            'identity_ref': 'fictional-binding'}])
                mode_capability = load('examples/capability.meiju-mode.json')
                mode_capability['actions'].update(load('examples/capability.meiju-fan.json')['actions'])
                with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                    database.execute('INSERT INTO devices VALUES (?, ?)',
                                     (grant_id, json.dumps({'device':device, 'capabilities':[power, mode_capability]})))
                for mode in ['cool', 'heat', 'auto', 'dry', 'fan']:
                    request = command('native-mode-plan', device_id=grant_id, capability='vendor.midea.ac',
                                      action='activate_mode', arguments={'mode':mode}, connection='cloud')
                    plan = raw(handle, envelope('plan', command=request))
                    responses['plan'].validate(plan)
                    assert (plan['error'] is None) == (bool(grants) and mode == 'cool')
                for capability, action, arguments in [('power','set',{'state':True}),
                                                      ('vendor.midea.ac','set_fan_speed',{'wind_speed':40})]:
                    request = command('independent-mode', device_id=grant_id, capability=capability,
                                      action=action, arguments=arguments, connection='cloud')
                    assert raw(handle, envelope('plan', command=request))['error']['code'] == 'POLICY_DENIED'
                forged = envelope('list_devices')
                forged['request']['meiju_mode_grants'] = [mode_grant]
                assert not request_schema.is_valid(forged)
                assert raw(handle, forged)['error']['code'] == 'INVALID_ARGUMENT'
            finally:
                library.homeverb_close(handle)
    bad_modes = [None, True, {}, [mode_grant, mode_grant], [mode_grant]*65,
                 [{'device_id':'*','mode':'cool'}], [{'device_id':grant_id}],
                 [{**mode_grant,'confirmed':True}], [{**mode_grant,'power':'off'}]]
    bad_modes.extend([{**mode_grant,'mode':mode}] for mode in [None, True, 1, 'off', 'COOL', 'fan_only'])
    for grants, adapter in [(grants,'midea.meiju') for grants in bad_modes] + [([mode_grant],None), ([mode_grant],'homeverb.mock')]:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-bad-mode-') as directory:
            private = Path(directory) / 'must-not-create'
            config = {'schema_version':'1.0', 'data_dir':str(private), 'allow_mock_writes':False,
                      'meiju_mode_grants':grants}
            if adapter:
                config['adapter'] = adapter
            assert not open_schema.is_valid(config), config
            assert decode(library.homeverb_open(json.dumps(config).encode()))['error']['code'] == 'INVALID_ARGUMENT'
            assert not private.exists()

    temperature_grant = {'device_id':grant_id, 'target_temperature':26.5}
    fleet_ids = [f'30000000-0000-4000-8000-{index:012d}' for index in range(1, 5)]
    fleet_grants = [{'device_id':device_id, 'target_temperature':half / 2}
                    for device_id in fleet_ids for half in range(32, 61)]
    capacity_grants = [{'device_id':f'30000000-0000-4000-8000-{index:012d}', 'target_temperature':26.5}
                       for index in range(1, 258)]
    assert len(fleet_grants) == 116
    for grants in [[], [temperature_grant], fleet_grants, capacity_grants[:256]]:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-temperature-') as directory:
            config = {'schema_version':'1.0', 'data_dir':directory, 'adapter':'midea.meiju',
                      'allow_mock_writes':False, 'meiju_temperature_grants':grants}
            open_schema.validate(config)
            result = decode(library.homeverb_open(json.dumps(config).encode()))
            assert result['error'] is None, result
            handle = result['data']['handle']
            try:
                assert raw(handle, envelope('discover'))['data'] == []
                for index, device_id in enumerate(fleet_ids if grants == fleet_grants else [grant_id]):
                    device = load('examples/device.json')
                    device.update(id=device_id, model='KFR-35G/WF2', support_status='experimental',
                                  components=[{'id':'main','capabilities':['power','climate']}],
                                  connections=[{'id':f'fictional-cloud-{index}','adapter':'midea.meiju','transport':'cloud',
                                                'account_id':'40000000-0000-4000-8000-000000000001','identity_ref':f'fictional-binding-{index}'}])
                    climate = load('examples/capability.meiju-temperature.json')
                    limits = climate['actions']['set']['parameters']['properties']['target_temperature']
                    if index % 2:
                        limits.update(minimum=16, multipleOf=1)
                    with sqlite3.connect(Path(directory) / 'homeverb.sqlite3') as database:
                        database.execute('INSERT INTO devices VALUES (?, ?)',
                                         (device_id, json.dumps({'device':device, 'capabilities':[power,climate]})))
                    for value in ([half / 2 for half in range(32, 61)] if grants == fleet_grants else [26,26.5,27]):
                        request = command('temperature-plan',device_id=device_id,capability='climate',action='set',
                                          arguments={'target_temperature':value},connection='cloud')
                        plan = raw(handle,envelope('plan',command=request))
                        responses['plan'].validate(plan)
                        allowed = {'device_id':device_id, 'target_temperature':value} in grants
                        valid = value >= limits['minimum'] and value % limits['multipleOf'] == 0
                        assert (plan['error'] is None) == (allowed and valid), (device_id, value, plan)
                        if not valid:
                            assert plan['error']['code'] == 'INVALID_ARGUMENT'
                denied = command('independent-power',device_id=grant_id,arguments={'state':True},connection='cloud')
                assert raw(handle,envelope('plan',command=denied))['error']['code'] == 'POLICY_DENIED'
                forged = envelope('list_devices')
                forged['request']['meiju_temperature_grants'] = [temperature_grant]
                assert not request_schema.is_valid(forged)
                assert raw(handle,forged)['error']['code'] == 'INVALID_ARGUMENT'
            finally:
                library.homeverb_close(handle)
    bad_temperatures = [None,True,{},[temperature_grant]*2,capacity_grants,
                        [{**temperature_grant,'device_id':'*'}],[{**temperature_grant,'confirmed':True}],
                        [{'device_id':grant_id,'target_temperature':26},{'device_id':grant_id,'target_temperature':26.0}]]
    bad_temperatures.extend([{**temperature_grant,'target_temperature':value}] for value in [None,True,'26',15.5,30.5,26.25])
    for grants,adapter in [(grants,'midea.meiju') for grants in bad_temperatures] + [([temperature_grant],None),([temperature_grant],'homeverb.mock')]:
        with tempfile.TemporaryDirectory(prefix='homeverb-ui-bad-temperature-') as directory:
            private = Path(directory) / 'must-not-create'
            config = {'schema_version':'1.0','data_dir':str(private),'allow_mock_writes':False,
                      'meiju_temperature_grants':grants}
            if adapter:
                config['adapter'] = adapter
            assert not open_schema.is_valid(config),config
            assert decode(library.homeverb_open(json.dumps(config).encode()))['error']['code'] == 'INVALID_ARGUMENT'
            assert not private.exists()

    example_path = ROOT / 'packages/native-contract/examples.json'
    if args.write_examples:
        example_path.write_text(json.dumps({'api_version': api['api_version'], 'cases': examples}, indent=2) + '\n')
    saved = load('packages/native-contract/examples.json')
    assert saved['api_version'] == api['api_version']
    for case in saved['cases']:
        request_schema.validate(case['request'])
        responses[case['request']['request']['method']].validate(case['response'])
    assert {case['request']['request']['method'] for case in saved['cases']} == set(api['methods'])
    account_examples = load('packages/native-contract/account-examples.json')
    assert account_examples['api_version'] == api['api_version']
    connect_schema.validate(account_examples['connect_metadata'])
    connect_schema.validate(account_examples['reauthorize_metadata'])
    assert account_examples['reauthorize_metadata']['account_id'] is not None
    for case in account_examples['cases']:
        validator('native-response', case['response_definition']).validate(case['response'])
        if 'request' in case:
            request_schema.validate(case['request'])
        if case['response_definition'] == 'inspect_account_inventory':
            for field, value in [('membership', 'online'), ('membership', True),
                                 ('device_id', 'not-a-uuid'), ('connection_id', None),
                                 ('appliance_id', 'PRIVATE'), ('online', True)]:
                bad = copy.deepcopy(case['response'])
                bad['data']['devices'][0][field] = value
                assert not responses['inspect_account_inventory'].is_valid(bad), (field, value)
            bad = copy.deepcopy(case['response'])
            del bad['data']['devices'][0]['membership']
            assert not responses['inspect_account_inventory'].is_valid(bad)
    removal_cases = [case for case in account_examples['cases'] if case['response_definition'] == 'remove_account']
    for case in removal_cases:
        bad = copy.deepcopy(case['response'])
        bad['data']['status'] = 'pending' if bad['data']['status'] == 'removed' else 'removed'
        assert not responses['remove_account'].is_valid(bad)
        bad = copy.deepcopy(case['response'])
        bad['data']['remote_revocation'] = 'revoked'
        assert not responses['remove_account'].is_valid(bad)
    for path, value in [
        (('account', 'credential_ref'), 'FORBIDDEN'),
        (('account', 'authorization'), 'connected'),
        (('account', 'id'), 'not-a-uuid'),
        (('authentication',), 'not_attempted'),
        (('retrieval', 'status'), 'unknown'),
        (('retrieval', 'device_ids'), ['not-a-uuid']),
        (('retrieval', 'error'), {'code': 'FAILED', 'message': 'Synthetic', 'retryable': False}),
    ]:
        bad = copy.deepcopy(account_examples['cases'][0]['response'])
        target = bad['data']
        for field in path[:-1]:
            target = target[field]
        target[path[-1]] = value
        assert not connect_response.is_valid(bad), path
    from verify_native_approval import verify_approvals
    verify_approvals()
    from verify_native_dishwasher import verify_dishwasher
    verify_dishwasher()
    print(f"PASS: {len(api['methods'])} native methods; {len(examples)} live examples; closed-field, error, outcome, and lifecycle checks.")
    print('Mock, empty Meiju startup, independent AC and trusted dishwasher grant admission and protected account-buffer guards verified on Linux; no live accounts, Flutter/mobile or physical-device verification in this suite.')


if __name__ == '__main__':
    main()
