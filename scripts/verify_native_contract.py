#!/usr/bin/env python3
"""Exercise every published UI method through the built C ABI using fictional data."""

from __future__ import annotations

import argparse
import copy
import ctypes
import json
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
    info_example['data']['api_version'] = '1.3.0'
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
            assert call(handle, 'refresh_account', account_id='00000000-0000-4000-8000-000000000001')['error']['code'] == 'AUTH_FLOW_UNSUPPORTED'
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
            assert not library.homeverb_cancel(handle, b'absent-operation')
            library.homeverb_set_foreground(handle, False)
            assert call(handle, 'get_api_info')['error']['code'] == 'HOST_BACKGROUND'
            library.homeverb_set_foreground(handle, True)
            assert call(handle, 'get_api_info')['data'] == api
        finally:
            library.homeverb_close(handle)
        assert call(handle, 'list_devices')['error']['code'] == 'HOST_CLOSED'

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
            library.homeverb_set_foreground(handle, False)
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
            connect_check('HOST_CLOSED', target=0)
            connect_check('MIDEA_LOGIN_INPUT_UNSUPPORTED', login_length=0)
            connect_check('MIDEA_LOGIN_INPUT_UNSUPPORTED', login_length=257)
        finally:
            library.homeverb_close(handle)
        assert raw(handle, envelope('list_devices'))['error']['code'] == 'HOST_CLOSED'

    with tempfile.TemporaryDirectory(prefix='homeverb-ui-mock-account-') as directory:
        handle = open_host(directory, allow=False)
        try:
            connect_check('AUTH_FLOW_UNSUPPORTED', target=handle)
        finally:
            library.homeverb_close(handle)

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
    for case in account_examples['cases']:
        validator('native-response', case['response_definition']).validate(case['response'])
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
    print(f"PASS: {len(api['methods'])} native methods; {len(examples)} live examples; closed-field, error, outcome, and lifecycle checks.")
    print('Mock, empty Meiju startup and protected account-buffer guards verified on Linux; no live accounts, Flutter/mobile or physical-device verification in this suite.')


if __name__ == '__main__':
    main()
