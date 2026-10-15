#!/usr/bin/env python3
"""Verify built CLI, MCP, native ABI, restart behavior, and runtime JSON contracts."""

from __future__ import annotations

import ctypes
import copy
from contextlib import closing
from datetime import datetime, timedelta, timezone
import json
import os
from pathlib import Path
import selectors
import signal
import sqlite3
import subprocess
import tempfile
import threading
import time
import unittest

from jsonschema import Draft202012Validator
from validate_contracts import checked_formats

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / 'target/debug/homeverb'
LIBRARY = ROOT / 'target/debug/libhomeverb_ffi.so'


class RuntimeAcceptance(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.formats = checked_formats()

    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix='homeverb-acceptance-')
        self.addCleanup(self.directory.cleanup)
        self.base = [str(BINARY), '--mock', '--data-dir', self.directory.name, '--profile', 'acceptance']

    @staticmethod
    def close_process(process):
        if process.poll() is None:
            process.kill()
        process.wait(timeout=2)
        for stream in [process.stdin, process.stdout, process.stderr]:
            if stream and not stream.closed:
                stream.close()

    def cli(self, *arguments, expected=0):
        result = subprocess.run([*self.base, *arguments], capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        self.assertEqual(result.stderr, '')
        return json.loads(result.stdout)

    def schema(self, name, value):
        schema = json.loads((ROOT / f'schemas/{name}.schema.json').read_text())
        Draft202012Validator(schema, format_checker=self.formats).validate(value)

    def test_cli_discovery_state_validation_and_persistence(self):
        self.assertEqual(self.cli('device', 'list')['data'], [])
        device = self.cli('device', 'discover')['data'][0]
        self.schema('device', device)
        description = self.cli('device', 'describe', 'mock-ac')['data']
        for capability in description['capabilities']:
            self.schema('capability', capability)
        initial = self.cli('device', 'state', 'mock-ac', '--fresh')['data']
        self.schema('state', initial)
        self.assertFalse(initial['observations']['power.state']['value'])
        result = self.cli('power', 'set', 'mock-ac', '--on')
        self.schema('command-result', result)
        self.assertEqual(result['status'], 'verified')
        current = self.cli('device', 'state', 'mock-ac', '--fresh')['data']
        self.assertTrue(current['observations']['power.state']['value'])
        invalid = self.cli('climate', 'set', 'mock-ac', '--temperature', '60', expected=2)
        self.schema('command-result', invalid)
        self.assertEqual(invalid['attempts'], [])
        self.cli('--dry-run', 'power', 'set', 'mock-ac', '--off')
        self.assertTrue(self.cli('device', 'state', 'mock-ac', '--fresh')['data']['observations']['power.state']['value'])

    def test_correlated_completion_receipts_preserve_cache_and_survive_restart(self):
        self.cli('--scenario', 'correlated-completion', 'device', 'discover')
        description = self.cli('device', 'describe', 'mock-ac')['data']
        for capability in description['capabilities']:
            self.schema('capability', capability)
            self.assertEqual(capability['actions']['set']['verification'], 'correlated_completion')
        before = self.cli('device', 'state', 'mock-ac', '--fresh')['data']
        result = self.cli('--scenario', 'correlated-completion', '--request-id', 'completion-one',
                          'power', 'set', 'mock-ac', '--on')
        self.schema('command-result', result)
        self.assertEqual(result['status'], 'verified')
        self.assertEqual(result['verification']['kind'], 'correlated_completion')
        self.assertNotIn('observed_at', result['verification'])
        self.assertGreaterEqual(result['verification']['received_at'], result['started_at'])
        cached = self.cli('device', 'state', 'mock-ac', '--cached')['data']
        self.assertEqual(cached['observations'], before['observations'])
        self.assertFalse(cached['observations']['power.state']['value'])
        self.assertEqual(self.cli('execution', 'completion-one')['data'], result)
        replay = self.cli('--scenario', 'offline', '--request-id', 'completion-one',
                          'power', 'set', 'mock-ac', '--on')
        self.assertEqual(replay, result)

        composite = self.cli('--scenario', 'correlated-completion', '--request-id', 'completion-parts',
                             'climate', 'set', 'mock-ac', '--mode', 'heat', '--temperature', '24')
        self.schema('command-result', composite)
        self.assertEqual(composite['status'], 'verified')
        self.assertEqual(len(composite['items']), 2)
        for item in composite['items']:
            self.assertEqual(item['verification']['kind'], 'correlated_completion')
            self.assertEqual(self.cli('execution', item['request_id'])['data'], item)

        acknowledgement = self.cli('power', 'set', 'mock-ac', '--off', expected=12)
        self.schema('command-result', acknowledgement)
        self.assertEqual(acknowledgement['status'], 'accepted')
        self.assertEqual(acknowledgement['error']['code'], 'VERIFICATION_PENDING')
        self.assertIsNone(acknowledgement['verification'])
        accepted = self.cli('--scenario', 'correlated-completion', '--wait', 'accepted',
                            'power', 'set', 'mock-ac', '--on')
        self.assertEqual(accepted['status'], 'accepted')
        self.assertIsNone(accepted['verification'])
        self.assertIsNone(accepted['error'])
        uncertain = self.cli('--scenario', 'lost-ack', 'power', 'set', 'mock-ac', '--off', expected=11)
        self.assertEqual(uncertain['status'], 'unknown')
        self.assertIsNone(uncertain['verification'])
        self.cli('device', 'discover')
        restored = self.cli('power', 'set', 'mock-ac', '--on')
        self.assertEqual(restored['verification']['kind'], 'state_match')

    def test_cli_batch_preflight_outcomes_and_individual_receipt_replay(self):
        self.cli('device', 'discover')
        path = Path(self.directory.name) / 'batch.json'
        def save(commands):
            batch = {'schema_version': '1.0', 'commands': commands}
            path.write_text(json.dumps(batch))
            return batch
        command = {'schema_version': '1.0', 'request_id': 'batch-dry-run', 'device_id': 'dev-mock-ac',
                   'component_id': 'main', 'capability': 'power', 'action': 'set', 'arguments': {'state': True},
                   'connection': 'auto', 'timeout_ms': 10000, 'wait_for': 'verified'}
        save([command])
        self.assertEqual(len(self.cli('--dry-run', 'batch', '--file', str(path))['data']['items']), 1)
        self.assertEqual(self.cli('executions')['data'], [])
        for extra in [command | {'request_id': 'another'},
                      command | {'request_id': 'another', 'device_id': 'missing'}]:
            save([command, extra])
            error = self.cli('batch', '--file', str(path), expected=2 if extra['device_id'] == 'dev-mock-ac' else 4)
            self.assertIsNotNone(error['error'])
            self.assertEqual(self.cli('executions')['data'], [])
        save([command | {'grants': []}])
        self.assertEqual(self.cli('batch', '--file', str(path), expected=2)['error']['code'], 'INVALID_ARGUMENT')
        for index, (scenario, arguments, wait, status, code) in enumerate([
                ('normal', {'state': True}, 'verified', 'verified', 0),
                ('accept-only', {'state': False}, 'verified', 'accepted', 12),
                ('lost-ack', {'state': False}, 'verified', 'unknown', 11),
                ('reject', {'state': True}, 'verified', 'failed', 10),
                ('normal', {'state': True}, 'accepted', 'accepted', 0),
                ('partial', {'mode': 'heat', 'target_temperature': 24}, 'verified', 'partial', 13)]):
            item = command | {'request_id': f'batch-{index}', 'arguments': arguments, 'wait_for': wait,
                              'capability': 'climate' if scenario == 'partial' else 'power'}
            self.schema('batch-command', save([item]))
            result = self.cli('--scenario', scenario, 'batch', '--file', str(path), expected=code)
            self.schema('batch-result', result)
            self.assertEqual(result['status'], status)
            self.assertEqual(result['items'][0], self.cli('execution', item['request_id'])['data'])
            replay = self.cli('--scenario', 'offline', 'batch', '--file', str(path), expected=code)
            self.assertEqual(replay['items'], result['items'])
        self.assertEqual(self.cli('--connection', 'cloud', 'batch', '--file', str(path), expected=2)['error']['code'],
                         'INVALID_ARGUMENT')
        self.assertEqual(self.cli('batch', '--file', str(path.with_suffix('.absent')), expected=2)['error']['code'],
                         'INVALID_ARGUMENT')

    def test_native_batch_planning_execution_and_foreground_cancellation(self):
        library = ctypes.CDLL(str(LIBRARY))
        library.homeverb_open.argtypes = [ctypes.c_char_p]
        library.homeverb_open.restype = ctypes.c_void_p
        library.homeverb_call.argtypes = [ctypes.c_uint64, ctypes.c_char_p]
        library.homeverb_call.restype = ctypes.c_void_p
        library.homeverb_string_free.argtypes = [ctypes.c_void_p]
        library.homeverb_set_foreground.argtypes = [ctypes.c_uint64, ctypes.c_bool]
        library.homeverb_close.argtypes = [ctypes.c_uint64]
        def decode(pointer):
            self.assertTrue(pointer)
            try:
                return json.loads(ctypes.string_at(pointer))
            finally:
                library.homeverb_string_free(pointer)
        directory = str(Path(self.directory.name) / 'native-batch')
        def open_host(allow, scenario):
            result = decode(library.homeverb_open(json.dumps({'schema_version': '1.0', 'data_dir': directory,
                            'allow_mock_writes': allow, 'scenario': scenario}).encode()))
            handle = result['data']['handle']
            self.addCleanup(library.homeverb_close, handle)
            return handle
        def call(request, operation='batch-test'):
            return decode(library.homeverb_call(handle, json.dumps({'schema_version': '1.0',
                          'operation_id': operation, 'timeout_ms': 10000, 'request': request}).encode()))
        command = {'schema_version': '1.0', 'request_id': 'native-batch-1', 'device_id': 'dev-mock-ac',
                   'component_id': 'main', 'capability': 'power', 'action': 'set', 'arguments': {'state': True},
                   'connection': 'auto', 'timeout_ms': 10000, 'wait_for': 'verified'}
        batch = {'schema_version': '1.0', 'commands': [command]}
        handle = open_host(False, 'normal')
        call({'method': 'discover'})
        self.assertEqual(call({'method': 'execute_batch', 'batch': batch})['error']['code'], 'POLICY_DENIED')
        self.assertEqual(call({'method': 'recent_executions'})['data'], [])
        library.homeverb_close(handle)
        handle = open_host(True, 'normal')
        self.assertEqual(len(call({'method': 'plan_batch', 'batch': batch})['data']['items']), 1)
        result = call({'method': 'execute_batch', 'batch': batch})
        self.schema('batch-result', result)
        self.assertEqual(result['status'], 'verified')
        self.assertEqual(call({'method': 'get_execution', 'request_id': command['request_id']})['data'], result['items'][0])
        library.homeverb_close(handle)
        handle = open_host(True, 'slow-after-send')
        command['request_id'] = 'native-batch-interrupted'
        command['arguments'] = {'state': False}
        replies = []
        worker = threading.Thread(target=lambda: replies.append(call({'method': 'execute_batch', 'batch': batch}, 'active-batch')))
        worker.start()
        try:
            deadline = time.monotonic() + 3
            while time.monotonic() < deadline:
                snapshot = call({'method': 'read_state', 'device': 'dev-mock-ac', 'component': 'main',
                                 'connection': 'auto', 'cached': False}, 'inspect-batch')['data']
                if not snapshot['observations']['power.state']['value']:
                    break
                time.sleep(0.01)
            self.assertFalse(snapshot['observations']['power.state']['value'])
        finally:
            library.homeverb_set_foreground(handle, False)
            worker.join(timeout=2)
        self.assertFalse(worker.is_alive())
        self.schema('batch-result', replies[0])
        self.assertEqual(replies[0]['status'], 'unknown')
        self.assertEqual(call({'method': 'execute_batch', 'batch': batch})['error']['code'], 'HOST_BACKGROUND')
        library.homeverb_set_foreground(handle, True)
        self.assertEqual(call({'method': 'get_execution', 'request_id': command['request_id']})['data'], replies[0]['items'][0])

    def test_cached_observation_contract_and_clock_changes(self):
        self.cli('device', 'discover')
        original = self.cli('device', 'state', 'mock-ac', '--fresh')['data']
        database = Path(self.directory.name) / 'acceptance/homeverb.sqlite3'

        def store(snapshot):
            with closing(sqlite3.connect(database)) as connection:
                connection.execute('UPDATE snapshots SET snapshot=? WHERE device=? AND component=?',
                                   (json.dumps(snapshot), 'dev-mock-ac', 'main'))
                connection.commit()

        invalid_snapshots = []
        for field, value in [('schema_version', '9.0'), ('device_id', 'foreign-device'), ('component_id', 'foreign-component')]:
            invalid = copy.deepcopy(original)
            invalid[field] = value
            invalid_snapshots.append(invalid)
        for name, value in [('power.state', 'on'), ('power.state', None),
                            ('climate.target_temperature', 60), ('climate.target_temperature', 24.25),
                            ('climate.mode', 'unsupported'), ('unknown.value', False)]:
            invalid = copy.deepcopy(original)
            invalid['observations'][name] = {**original['observations']['power.state'], 'value': value}
            invalid_snapshots.append(invalid)
        for invalid in invalid_snapshots:
            store(invalid)
            failure = self.cli('--scenario', 'offline', 'device', 'state', 'mock-ac', '--cached', expected=10)
            self.assertEqual(failure['error']['code'], 'INVALID_OBSERVATION')
            self.assertIsNone(failure['data'])
            with closing(sqlite3.connect(database)) as connection:
                saved = json.loads(connection.execute('SELECT snapshot FROM snapshots').fetchone()[0])
            self.assertEqual(saved, invalid)

        old = copy.deepcopy(original)
        old['observations']['power.state']['observed_at'] = '2000-01-01T00:00:00Z'
        store(old)
        cached = self.cli('--scenario', 'offline', 'device', 'state', 'mock-ac', '--cached')['data']
        self.schema('state', cached)
        self.assertTrue(cached['observations']['power.state']['stale'])
        self.assertIn('One or more observations reached their property freshness limit.', cached['warnings'])

        future = copy.deepcopy(original)
        for observation in future['observations'].values():
            observation['source'] = 'cloud'
            observation['observed_at'] = '2100-01-01T00:00:00Z'
            observation['received_at'] = '2100-01-01T00:00:00Z'
        store(future)
        cached = self.cli('--scenario', 'offline', '--connection', 'lan', 'device', 'state', 'mock-ac', '--cached')['data']
        self.schema('state', cached)
        self.assertTrue(cached['from_cache'])
        self.assertTrue(all(observation['stale'] for observation in cached['observations'].values()))
        self.assertTrue(all(observation['source'] == 'cloud' for observation in cached['observations'].values()))
        self.assertIn('One or more receipt timestamps are ahead of the local clock.', cached['warnings'])
        self.assertEqual(self.cli('device', 'state', 'mock-ac', '--fresh')['data']['schema_version'], '1.0')

    def test_cached_property_freshness_uses_current_metadata_without_rewriting_observations(self):
        self.cli('device', 'discover')
        descriptor = self.cli('device', 'describe', 'mock-ac')['data']
        descriptor['capabilities'][0]['properties']['state']['max_age_ms'] = 1000.0
        descriptor['capabilities'][1]['properties']['mode']['max_age_ms'] = 120000
        snapshot = self.cli('device', 'state', 'mock-ac', '--fresh')['data']
        old_source = (datetime.now(timezone.utc) - timedelta(seconds=60)).isoformat()
        for observation in snapshot['observations'].values():
            observation['observed_at'] = old_source
        database = Path(self.directory.name) / 'acceptance/homeverb.sqlite3'
        with closing(sqlite3.connect(database)) as connection:
            connection.execute('UPDATE devices SET descriptor=?', (json.dumps(descriptor),))
            connection.execute('UPDATE snapshots SET snapshot=?', (json.dumps(snapshot),))
            connection.commit()
        cached = self.cli('--scenario', 'offline', 'device', 'state', 'mock-ac', '--cached')['data']
        self.schema('state', cached)
        self.assertTrue(cached['observations']['power.state']['stale'])
        self.assertTrue(cached['observations']['climate.target_temperature']['stale'])
        self.assertFalse(cached['observations']['climate.mode']['stale'])
        with closing(sqlite3.connect(database)) as connection:
            self.assertEqual(json.loads(connection.execute('SELECT snapshot FROM snapshots').fetchone()[0]), snapshot)
            for capability in descriptor['capabilities']:
                for prop in capability['properties'].values():
                    prop['max_age_ms'] = 1000000000
            snapshot['observations']['climate.mode']['stale'] = True
            connection.execute('UPDATE devices SET descriptor=?', (json.dumps(descriptor),))
            connection.execute('UPDATE snapshots SET snapshot=?', (json.dumps(snapshot),))
            connection.commit()
        relaxed = self.cli('--scenario', 'offline', 'device', 'state', 'mock-ac', '--cached')['data']
        self.assertFalse(relaxed['observations']['power.state']['stale'])
        self.assertFalse(relaxed['observations']['climate.target_temperature']['stale'])
        self.assertTrue(relaxed['observations']['climate.mode']['stale'])
        refreshed = self.cli('device', 'state', 'mock-ac', '--fresh')['data']
        self.assertTrue(all(not observation['stale'] for observation in refreshed['observations'].values()))

    def test_numeric_verification_preserves_observed_values_and_request_identity(self):
        self.cli('device', 'discover')
        # The simulator starts at 26.0; accept-only leaves that observed value unchanged.
        result = self.cli('--scenario', 'accept-only', '--request-id', 'numeric-match',
                          'device', 'execute', 'mock-ac', '--capability', 'climate', '--action', 'set',
                          '--arguments', '{"target_temperature":26}')
        self.schema('command-result', result)
        self.assertEqual(result['status'], 'verified')
        self.assertIs(type(result['data']['target_temperature']), int)
        cached = self.cli('device', 'state', 'mock-ac', '--cached')['data']
        self.assertIs(type(cached['observations']['climate.target_temperature']['value']), float)

        # Numeric state equality does not change the conservative request replay contract.
        conflict = self.cli('--request-id', 'numeric-match', 'device', 'execute', 'mock-ac',
                            '--capability', 'climate', '--action', 'set',
                            '--arguments', '{"target_temperature":26.0}', expected=7)
        self.assertEqual(conflict['error']['code'], 'REQUEST_CONFLICT')
        self.assertEqual(conflict['attempts'], [])
        pending = self.cli('--scenario', 'accept-only', 'climate', 'set', 'mock-ac', '--temperature', '26.5', expected=12)
        self.assertEqual(pending['status'], 'accepted')
        self.assertEqual(pending['error']['code'], 'VERIFICATION_PENDING')
        self.assertIsNone(pending['verification'])
        invalid = self.cli('device', 'execute', 'mock-ac', '--capability', 'climate', '--action', 'set',
                           '--arguments', '{"target_temperature":"26"}', expected=2)
        self.assertEqual(invalid['attempts'], [])

        self.cli('device', 'execute', 'mock-ac', '--capability', 'climate', '--action', 'set',
                 '--arguments', '{"target_temperature":25}')
        result = self.cli('--scenario', 'accept-only', 'climate', 'set', 'mock-ac', '--temperature', '25')
        self.schema('command-result', result)
        self.assertEqual(result['status'], 'verified')
        self.assertIs(type(result['data']['target_temperature']), float)
        cached = self.cli('device', 'state', 'mock-ac', '--cached')['data']
        self.assertIs(type(cached['observations']['climate.target_temperature']['value']), int)

    def test_component_capability_scopes_preserve_distinct_cached_constraints(self):
        self.cli('device', 'discover')
        original = self.cli('device', 'describe', 'mock-ac')['data']
        descriptor = copy.deepcopy(original)
        descriptor['device']['components'] = [{'id': component, 'capabilities': ['climate']}
                                               for component in ['fridge', 'freezer']]
        descriptor['capabilities'] = [json.loads((ROOT / f'examples/capability.{component}.json').read_text())
                                      for component in ['fridge', 'freezer']]
        database = Path(self.directory.name) / 'acceptance/homeverb.sqlite3'
        observed = (datetime.now(timezone.utc) - timedelta(seconds=30)).isoformat()
        snapshots = {}
        for component, temperature in [('fridge', 4), ('freezer', -18)]:
            snapshots[component] = {'schema_version': '1.0', 'device_id': 'dev-mock-ac', 'component_id': component,
                'from_cache': False, 'warnings': [], 'observations': {'climate.target_temperature': {
                    'value': temperature, 'source': 'lan', 'observed_at': None, 'received_at': observed, 'stale': False}}}
        with closing(sqlite3.connect(database)) as connection:
            connection.execute('UPDATE devices SET descriptor=? WHERE id=?', (json.dumps(descriptor), 'dev-mock-ac'))
            for component, snapshot in snapshots.items():
                connection.execute('INSERT INTO snapshots(device,component,snapshot) VALUES(?,?,?)',
                                   ('dev-mock-ac', component, json.dumps(snapshot)))
            connection.commit()
        described = self.cli('device', 'describe', 'mock-ac')['data']
        for capability in described['capabilities']:
            self.schema('capability', capability)
        self.assertEqual([capability['component_id'] for capability in described['capabilities']], ['fridge', 'freezer'])
        for component, temperature, stale in [('fridge', 4, True), ('freezer', -18, False)]:
            state = self.cli('device', 'state', 'mock-ac', '--component', component, '--cached')['data']
            self.schema('state', state)
            self.assertEqual(state['observations']['climate.target_temperature']['value'], temperature)
            self.assertEqual(state['observations']['climate.target_temperature']['stale'], stale)
            denied = self.cli('device', 'execute', 'mock-ac', '--component', component, '--capability', 'climate',
                              '--action', 'set', '--arguments', json.dumps({'target_temperature': temperature}), expected=6)
            self.assertEqual(denied['error']['code'], 'POLICY_DENIED')
            self.assertEqual(denied['attempts'], [])
        invalid = self.cli('device', 'execute', 'mock-ac', '--component', 'freezer', '--capability', 'climate',
                           '--action', 'set', '--arguments', '{"target_temperature":4}', expected=2)
        self.assertEqual(invalid['error']['code'], 'INVALID_ARGUMENT')
        self.assertEqual(self.cli('executions')['data'], [])
        with closing(sqlite3.connect(database)) as connection:
            for component, snapshot in snapshots.items():
                stored = connection.execute('SELECT snapshot FROM snapshots WHERE device=? AND component=?',
                                            ('dev-mock-ac', component)).fetchone()[0]
                self.assertEqual(json.loads(stored), snapshot)
            descriptor['capabilities'][1].pop('component_id')
            connection.execute('UPDATE devices SET descriptor=? WHERE id=?', (json.dumps(descriptor), 'dev-mock-ac'))
            connection.commit()
        self.assertEqual(self.cli('device', 'describe', 'mock-ac', expected=4)['error']['code'], 'CAPABILITY_UNSUPPORTED')
        self.cli('device', 'discover')
        self.assertEqual(self.cli('device', 'describe', 'mock-ac')['data'], original)

    def test_invalid_device_metadata_blocks_use_until_valid_rediscovery(self):
        self.cli('device', 'discover')
        original = self.cli('device', 'describe', 'mock-ac')['data']
        snapshot = self.cli('device', 'state', 'mock-ac', '--fresh')['data']
        database = Path(self.directory.name) / 'acceptance/homeverb.sqlite3'
        for mutation in ['version', 'duplicate-capability', 'malformed-property', 'invalid-decomposition']:
            descriptor = copy.deepcopy(original)
            if mutation == 'version':
                descriptor['device']['schema_version'] = '9.0'
            elif mutation == 'duplicate-capability':
                descriptor['capabilities'].append(copy.deepcopy(descriptor['capabilities'][0]))
            elif mutation == 'malformed-property':
                descriptor['capabilities'][0]['properties']['state']['type'] = 'unknown'
            else:
                descriptor['capabilities'][1]['actions']['set']['idempotency'] = 'non_idempotent'
            with closing(sqlite3.connect(database)) as connection:
                connection.execute('UPDATE devices SET descriptor=? WHERE id=?',
                                   (json.dumps(descriptor), 'dev-mock-ac'))
                connection.commit()
            for arguments in [('device', 'describe', 'mock-ac'),
                              ('--dry-run', 'power', 'set', 'mock-ac', '--on'),
                              ('power', 'set', 'mock-ac', '--on'),
                              ('device', 'state', 'mock-ac', '--fresh'),
                              ('device', 'state', 'mock-ac', '--cached')]:
                rejected = self.cli(*arguments, expected=4)
                self.assertEqual(rejected['error']['code'], 'CAPABILITY_UNSUPPORTED')
                self.assertIsNone(rejected['data'])
            self.assertEqual(self.cli('executions')['data'], [])
            self.assertEqual(self.cli('executions', '--device', 'dev-mock-ac')['data'], [])
            self.assertEqual(self.cli('executions', '--device', 'mock-ac')['data'], [])
            preferences = self.cli('device', 'preferences', 'mock-ac', '--settings',
                                   '{"alias":null,"room_id":null,"favorite":true}')['data']
            self.assertTrue(preferences['favorite'])
            with closing(sqlite3.connect(database)) as connection:
                self.assertEqual(json.loads(connection.execute('SELECT snapshot FROM snapshots').fetchone()[0]), snapshot)
                self.assertEqual(json.loads(connection.execute('SELECT descriptor FROM devices').fetchone()[0]), descriptor)
        self.cli('device', 'discover')
        self.assertEqual(self.cli('device', 'describe', 'mock-ac')['data'], original)
        current = self.cli('device', 'state', 'mock-ac', '--fresh')['data']
        self.assertFalse(current['observations']['power.state']['value'])

    def test_exact_receipt_lookup_preserves_uncertainty_without_valid_device_metadata(self):
        self.cli('device', 'discover')
        original = self.cli('--scenario', 'lost-ack', '--request-id', 'inspect-unknown',
                            'power', 'set', 'mock-ac', '--on', expected=11)
        current = self.cli('device', 'state', 'mock-ac', '--fresh')['data']
        self.assertTrue(current['observations']['power.state']['value'])
        database = Path(self.directory.name) / 'acceptance/homeverb.sqlite3'
        with closing(sqlite3.connect(database)) as connection:
            descriptor = json.loads(connection.execute('SELECT descriptor FROM devices').fetchone()[0])
            descriptor['device']['schema_version'] = 'unsupported'
            connection.execute('UPDATE devices SET descriptor=?', (json.dumps(descriptor),))
            connection.execute('INSERT INTO retired_requests VALUES (?)', ('retired-receipt',))
            connection.commit()
            before = {table: connection.execute(f'SELECT * FROM {table}').fetchall()
                      for table in ('devices', 'snapshots', 'executions', 'retired_requests')}
        query = self.cli('--scenario', 'offline', 'execution', 'inspect-unknown')
        self.assertEqual(set(query), {'schema_version', 'data', 'warnings', 'error'})
        self.assertIsNone(query['error'])
        self.schema('command-result', query['data'])
        self.assertEqual(query['data'], original)
        self.assertEqual(query['data']['status'], 'unknown')
        for request_id, exit_code, code in [('missing', 4, 'REQUEST_NOT_FOUND'),
                                           ('retired-receipt', 7, 'REQUEST_RETIRED'),
                                           (' ', 2, 'INVALID_ARGUMENT')]:
            rejected = self.cli('--scenario', 'offline', 'execution', request_id, expected=exit_code)
            self.assertEqual(rejected['error']['code'], code)
            self.assertFalse(rejected['error']['retryable'])
            self.assertIsNone(rejected['data'])
        with closing(sqlite3.connect(database)) as connection:
            after = {table: connection.execute(f'SELECT * FROM {table}').fetchall() for table in before}
        self.assertEqual(after, before)

    def test_discovery_conflicts_preserve_existing_identity_and_local_records(self):
        self.cli('device', 'discover')
        self.cli('power', 'set', 'mock-ac', '--on')
        self.cli('device', 'preferences', 'mock-ac', '--settings',
                 '{"alias":null,"room_id":null,"favorite":true}')
        descriptor = self.cli('device', 'describe', 'mock-ac')['data']
        descriptor['device']['connections'][0]['identity_ref'] = 'identity:conflicting-appliance'
        database = Path(self.directory.name) / 'acceptance/homeverb.sqlite3'
        with closing(sqlite3.connect(database)) as connection:
            connection.execute('UPDATE devices SET descriptor=? WHERE id=?',
                               (json.dumps(descriptor), 'dev-mock-ac'))
            connection.commit()
            before = {table: connection.execute(f'SELECT * FROM {table}').fetchall()
                      for table in ('devices', 'device_preferences', 'snapshots', 'executions')}
        rejected = self.cli('device', 'discover', expected=4)
        self.assertEqual(rejected['error']['code'], 'DEVICE_IDENTITY_CONFLICT')
        self.assertFalse(rejected['error']['retryable'])
        self.assertIsNone(rejected['data'])
        self.assertNotIn('conflicting-appliance', json.dumps(rejected))
        with closing(sqlite3.connect(database)) as connection:
            after = {table: connection.execute(f'SELECT * FROM {table}').fetchall() for table in before}
        self.assertEqual(after, before)

    def test_legacy_action_metadata_keeps_arguments_intact_until_rediscovery(self):
        self.cli('device', 'discover')
        descriptor = self.cli('device', 'describe', 'mock-ac')['data']
        climate = descriptor['capabilities'][1]
        self.assertTrue(climate['actions']['set'].pop('independent_fields'))
        climate['revision'] = 1
        database = Path(self.directory.name) / 'acceptance/homeverb.sqlite3'
        with closing(sqlite3.connect(database)) as connection:
            connection.execute('UPDATE devices SET descriptor=? WHERE id=?',
                               (json.dumps(descriptor), 'dev-mock-ac'))
            connection.commit()
        arguments = ('--request-id', 'legacy-intact', 'climate', 'set', 'mock-ac',
                     '--mode', 'heat', '--temperature', '24')
        self.assertNotIn('items', self.cli('--dry-run', *arguments)['data'])
        result = self.cli(*arguments)
        self.schema('command-result', result)
        self.assertEqual(result['status'], 'verified')
        self.assertEqual(result['data'], {'mode': 'heat', 'target_temperature': 24})
        self.assertIsNone(result.get('items'))
        self.assertEqual(len(result['attempts']), 1)
        self.assertEqual(self.cli('--scenario', 'offline', *arguments), result)

        self.cli('device', 'discover')
        updated = self.cli('device', 'describe', 'mock-ac')['data']['capabilities'][1]
        self.assertEqual(updated['revision'], 2)
        self.assertTrue(updated['actions']['set']['independent_fields'])
        self.assertEqual(self.cli('--scenario', 'offline', *arguments), result)
        composite = self.cli('--request-id', 'explicit-independent', 'climate', 'set', 'mock-ac',
                             '--mode', 'cool', '--temperature', '25')
        self.schema('command-result', composite)
        self.assertEqual(composite['status'], 'verified')
        self.assertEqual(len(composite['items']), 2)

    def test_maximum_request_ids_support_composites_across_processes(self):
        self.cli('device', 'discover')
        for request_id in ['r' * 256, '\u00e9' * 128]:
            self.assertEqual(len(request_id.encode('utf-8')), 256)
            arguments = ('--request-id', request_id, 'climate', 'set', 'mock-ac', '--mode', 'heat', '--temperature', '24')
            plan = self.cli('--dry-run', *arguments)['data']
            self.assertEqual([item['command']['request_id'] for item in plan['items']],
                             [request_id + ':0', request_id + ':1'])
            result = self.cli(*arguments)
            self.schema('command-result', result)
            self.assertEqual(result['status'], 'verified')
            self.assertEqual([item['request_id'] for item in result['items']], [request_id + ':0', request_id + ':1'])
            for item in result['items']:
                self.assertEqual(self.cli('--scenario', 'offline', 'execution', item['request_id'])['data'], item)
            # Each invocation is a new process; offline replay must return the stored receipt.
            self.assertEqual(self.cli('--scenario', 'offline', *arguments), result)
            rejected = self.cli('--request-id', request_id + 'x', 'climate', 'set', 'mock-ac',
                                '--mode', 'heat', '--temperature', '24', expected=2)
            self.assertEqual(rejected['error']['code'], 'INVALID_ARGUMENT')
            self.assertEqual(rejected['attempts'], [])

    def test_cli_diagnostics_is_bounded_local_and_redacted(self):
        empty = self.cli('diagnostics')['data']
        self.schema('diagnostics', empty)
        self.assertEqual(empty['device_count'], 0)
        self.assertEqual(empty['recent_receipts'], [])
        self.cli('device', 'discover')
        marker = 'diagnostic-private-marker'
        room = self.cli('room', 'create', marker)['data']
        self.cli('device', 'preferences', 'mock-ac', '--settings', json.dumps({
            'alias': marker, 'room_id': room['id'], 'favorite': True}))
        self.cli('--request-id', marker, 'power', 'set', 'dev-mock-ac', '--on')
        database = Path(self.directory.name) / 'acceptance/homeverb.sqlite3'
        with closing(sqlite3.connect(database)) as connection:
            result = json.loads(connection.execute('SELECT result FROM executions WHERE request_id=?', (marker,)).fetchone()[0])
            result['error'] = {'code': marker, 'message': marker, 'retryable': False}
            result['data'] = {'private': marker}
            result['warnings'] = [marker]
            result['verification']['note'] = marker
            connection.execute('UPDATE executions SET result=? WHERE request_id=?', (json.dumps(result), marker))
            connection.commit()
        report = self.cli('--scenario', 'offline', 'diagnostics')['data']
        self.schema('diagnostics', report)
        self.assertEqual(report['device_count'], 1)
        self.assertEqual(report['room_count'], 1)
        self.assertEqual(report['connections'], {'lan': 1, 'cloud': 1})
        receipt = report['recent_receipts'][0]
        self.assertEqual(receipt['error_code'], 'OTHER')
        self.assertEqual(receipt['warning_count'], 1)
        self.assertEqual(receipt['verification_kind'], 'state_match')
        for private in (marker, 'dev-mock-ac', room['id'], self.directory.name):
            self.assertNotIn(private, json.dumps(report))
        self.cli('diagnostics', '--include-secrets', expected=2)
        self.assertEqual(self.cli('executions')['data'][0]['warnings'], [marker])

    def test_outcomes_routes_and_composite_receipts(self):
        self.cli('device', 'discover')
        plan = self.cli('--dry-run', 'climate', 'set', 'mock-ac', '--temperature', '25', '--mode', 'heat')['data']
        self.assertEqual(len(plan['items']), 2)
        for item in plan['items']:
            self.schema('command', item['command'])
            self.assertEqual(len(item['command']['arguments']), 1)
            self.assertEqual(item['transports'], ['lan', 'cloud'])
        self.assertEqual(self.cli('executions')['data'], [])
        accepted = self.cli('--scenario', 'accept-only', 'power', 'set', 'mock-ac', '--on', expected=12)
        self.schema('command-result', accepted)
        self.assertEqual(accepted['status'], 'accepted')
        rejected = self.cli('--scenario', 'reject', 'power', 'set', 'mock-ac', '--on', expected=10)
        self.schema('command-result', rejected)
        self.cli('--scenario', 'lan-offline', '--connection', 'lan', 'power', 'set', 'mock-ac', '--on', expected=5)
        unknown = self.cli('--scenario', 'lost-ack', '--request-id', 'uncertain-1', 'power', 'set', 'mock-ac', '--on', expected=11)
        self.schema('command-result', unknown)
        replay = self.cli('--request-id', 'uncertain-1', 'power', 'set', 'mock-ac', '--on', expected=11)
        self.assertEqual(replay, unknown)
        composite = self.cli('climate', 'set', 'mock-ac', '--temperature', '25', '--mode', 'heat')
        self.schema('command-result', composite)
        self.assertEqual(composite['status'], 'verified')
        self.assertEqual(len(composite['items']), 2)
        partial = self.cli('--scenario', 'partial', 'climate', 'set', 'mock-ac', '--temperature', '24', '--mode', 'cool', expected=13)
        self.schema('command-result', partial)
        self.assertEqual([item['status'] for item in partial['items']], ['verified', 'failed'])

    def test_cli_room_preferences_survive_restart_and_preserve_device_routes(self):
        self.cli('device', 'discover')
        original = self.cli('device', 'describe', 'mock-ac')['data']['device']
        room = self.cli('room', 'create', 'Bedroom')['data']
        self.schema('room', room)
        other = self.cli('room', 'create', 'Office')['data']
        ordered = self.cli('room', 'reorder', other['id'], room['id'])['data']
        self.assertEqual([entry['id'] for entry in ordered], [other['id'], room['id']])
        self.cli('room', 'create', 'bedroom', expected=2)
        preferences = self.cli('device', 'preferences', 'mock-ac', '--settings', json.dumps({
            'alias': 'Quiet AC', 'room_id': room['id'], 'favorite': True}))['data']
        self.schema('device-preferences', preferences)
        self.cli('device', 'discover')
        card = self.cli('device', 'cards')['data'][0]
        self.assertEqual(card['preferences'], preferences)
        self.assertEqual(card['device']['id'], original['id'])
        self.assertEqual(card['device']['connections'], original['connections'])
        self.assertEqual(card['device']['alias'], 'Quiet AC')
        self.cli('device', 'preferences', 'Quiet AC', '--settings', '{"favorite": false}', expected=2)
        self.assertTrue(self.cli('device', 'cards')['data'][0]['preferences']['favorite'])
        self.cli('room', 'rename', room['id'], 'Primary Bedroom')
        self.cli('room', 'delete', room['id'])
        after = self.cli('device', 'cards')['data'][0]
        self.assertIsNone(after['preferences']['room_id'])
        self.assertEqual(after['device']['alias'], 'Quiet AC')
        self.assertEqual(self.cli('executions')['data'], [])

    def test_batch_room_assignment_rolls_back_all_devices_on_storage_failure(self):
        self.cli('device', 'discover')
        descriptor = self.cli('device', 'describe', 'mock-ac')['data']
        descriptor['device']['id'] = 'dev-mock-second'
        descriptor['device']['alias'] = 'second-ac'
        database = Path(self.directory.name) / 'acceptance/homeverb.sqlite3'
        with closing(sqlite3.connect(database)) as connection:
            connection.execute('INSERT INTO devices VALUES (?, ?)', ('dev-mock-second', json.dumps(descriptor)))
            connection.execute('INSERT INTO device_preferences(device_id, position) VALUES (?, ?)', ('dev-mock-second', 1))
            connection.commit()
        room = self.cli('room', 'create', 'Destination')['data']
        self.cli('device', 'preferences', 'mock-ac', '--settings', '{"alias":"Keep this name","room_id":null,"favorite":true}')
        before = self.cli('device', 'cards')['data']
        arguments = ('room', 'assign', 'dev-mock-ac', 'dev-mock-second', '--room', room['id'])
        with closing(sqlite3.connect(database)) as connection:
            connection.executescript('''
                CREATE TRIGGER reject_second_assignment BEFORE UPDATE OF room_id ON device_preferences
                WHEN NEW.device_id='dev-mock-second'
                BEGIN SELECT RAISE(FAIL, 'Fictional assignment failure'); END;
            ''')
        failure = self.cli(*arguments, expected=10)
        self.assertEqual(failure['error']['code'], 'STORAGE_ERROR')
        self.assertEqual(self.cli('device', 'cards')['data'], before)
        with closing(sqlite3.connect(database)) as connection:
            connection.execute('DROP TRIGGER reject_second_assignment')
        assigned = self.cli(*arguments)['data']
        self.assertEqual([item['device_id'] for item in assigned], ['dev-mock-ac', 'dev-mock-second'])
        for item in assigned:
            self.schema('device-preferences', item)
            self.assertEqual(item['room_id'], room['id'])
        self.assertEqual(assigned[0]['alias'], 'Keep this name')
        self.assertTrue(assigned[0]['favorite'])
        self.cli('room', 'assign', 'dev-mock-ac', expected=2)
        self.cli('room', 'assign', 'dev-mock-ac', '--room', room['id'], '--unassigned', expected=2)
        self.cli('room', 'assign', 'dev-mock-ac', 'dev-mock-ac', '--unassigned', expected=2)
        unassigned = self.cli('room', 'assign', 'dev-mock-second', 'dev-mock-ac', '--unassigned')['data']
        self.assertEqual([item['device_id'] for item in unassigned], ['dev-mock-second', 'dev-mock-ac'])
        self.assertTrue(all(item['room_id'] is None for item in unassigned))
        self.assertEqual(self.cli('executions')['data'], [])
        self.cli('power', 'set', 'dev-mock-ac', '--on')
        history = self.cli('executions', '--device', 'Keep this name')['data']
        self.assertEqual(len(history), 1)
        self.assertEqual(history[0]['device_id'], 'dev-mock-ac')
        self.assertEqual(self.cli('executions', '--device', 'second-ac')['data'], [])
        self.cli('executions', '--device', 'missing', expected=4)

    def test_composite_preserves_verified_item_when_journal_finalization_fails(self):
        self.cli('device', 'discover')
        database = Path(self.directory.name) / 'acceptance/homeverb.sqlite3'
        with closing(sqlite3.connect(database)) as connection:
            connection.executescript('''
                CREATE TRIGGER reject_item_finalization BEFORE UPDATE ON executions
                WHEN NEW.request_id='journal-fault:0' AND NEW.finalized=1
                BEGIN SELECT RAISE(FAIL, 'Fictional journal failure'); END;
            ''')
        arguments = ('--request-id', 'journal-fault', 'climate', 'set', 'mock-ac',
                     '--mode', 'heat', '--temperature', '25')
        result = self.cli(*arguments, expected=13)
        self.schema('command-result', result)
        self.assertEqual(result['status'], 'partial')
        verified, skipped = result['items']
        self.assertEqual(verified['status'], 'verified')
        self.assertIsNone(verified['error'])
        self.assertIsNotNone(verified['verification'])
        self.assertTrue(verified['warnings'])
        self.assertEqual(skipped['error']['code'], 'NOT_EXECUTED')
        self.assertEqual(skipped['attempts'], [])
        # Restore storage before reopening; recovery must not replay the earlier write.
        with closing(sqlite3.connect(database)) as connection:
            connection.execute('DROP TRIGGER reject_item_finalization')
        self.assertEqual(self.cli(*arguments, expected=13), result)
        state = self.cli('device', 'state', 'mock-ac', '--fresh')['data']['observations']
        self.assertEqual(state['climate.mode']['value'], 'heat')
        self.assertEqual(state['climate.target_temperature']['value'], 26.0)

    def test_version_one_profile_upgrades_with_backup_through_actual_cli(self):
        directory = Path(self.directory.name) / 'acceptance'
        directory.mkdir()
        descriptor = {'device': json.loads((ROOT / 'examples/device.json').read_text()),
                      'capabilities': [json.loads((ROOT / 'examples/capability.climate.json').read_text())]}
        with closing(sqlite3.connect(directory / 'homeverb.sqlite3')) as database:
            database.executescript('''
                CREATE TABLE devices (id TEXT PRIMARY KEY, descriptor TEXT NOT NULL);
                CREATE TABLE snapshots (device TEXT NOT NULL, component TEXT NOT NULL, snapshot TEXT NOT NULL, PRIMARY KEY(device, component));
                CREATE TABLE executions (request_id TEXT PRIMARY KEY, command TEXT NOT NULL, result TEXT NOT NULL);
                PRAGMA user_version=1;
            ''')
            database.execute('INSERT INTO devices VALUES (?, ?)', (descriptor['device']['id'], json.dumps(descriptor)))
            database.commit()
        cards = self.cli('device', 'cards')['data']
        self.assertEqual(cards[0]['device']['id'], descriptor['device']['id'])
        self.assertIsNone(cards[0]['preferences']['room_id'])
        backups = list((directory / 'backups').glob('schema-1-*.sqlite3'))
        self.assertEqual(len(backups), 1)
        with closing(sqlite3.connect(backups[0])) as backup:
            self.assertEqual(backup.execute('PRAGMA user_version').fetchone()[0], 1)
            self.assertEqual(backup.execute('SELECT COUNT(*) FROM devices').fetchone()[0], 1)

    def test_forced_termination_never_replays_pending_write(self):
        self.cli('device', 'discover')
        process = subprocess.Popen([*self.base, '--scenario', 'slow-after-send', '--request-id', 'crash-1',
                                    'power', 'set', 'mock-ac', '--on'], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        self.addCleanup(self.close_process, process)
        database = Path(self.directory.name) / 'acceptance/mock-appliance.sqlite3'
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            with closing(sqlite3.connect(database)) as connection:
                state = json.loads(connection.execute('SELECT state FROM appliance WHERE id=1').fetchone()[0])
            if state['power.state']:
                break
            time.sleep(0.01)
        else:
            self.fail('The simulated write did not reach transmission')
        process.kill()
        process.communicate(timeout=2)
        recovered = self.cli('--request-id', 'crash-1', 'power', 'set', 'mock-ac', '--on', expected=11)
        self.assertEqual(recovered['status'], 'unknown')
        self.assertEqual(len(recovered['attempts']), 1)

    def test_sigint_terminates_after_sent_write_with_unknown_result(self):
        self.cli('device', 'discover')
        process = subprocess.Popen([*self.base, '--scenario', 'slow-after-send', 'power', 'set', 'mock-ac', '--on'],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.addCleanup(self.close_process, process)
        database = Path(self.directory.name) / 'acceptance/mock-appliance.sqlite3'
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            with closing(sqlite3.connect(database)) as connection:
                state = json.loads(connection.execute('SELECT state FROM appliance WHERE id=1').fetchone()[0])
            if state['power.state']:
                break
            time.sleep(0.01)
        self.assertTrue(state['power.state'])
        process.send_signal(signal.SIGINT)
        stdout, stderr = process.communicate(timeout=2)
        self.assertEqual(process.returncode, 130)
        self.assertEqual(stderr, '')
        self.assertEqual(json.loads(stdout)['status'], 'unknown')

    def test_mcp_protocol_read_tools_policy_and_eof_exit(self):
        self.cli('device', 'discover')
        receipt = self.cli('--scenario', 'lost-ack', '--request-id', 'mcp-receipt',
                           'power', 'set', 'mock-ac', '--on', expected=11)
        process = subprocess.Popen([*self.base, 'mcp', '--stdio'], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, text=True, bufsize=1)
        self.addCleanup(self.close_process, process)

        def send(value):
            process.stdin.write(json.dumps(value) + '\n')
            process.stdin.flush()

        def response():
            with selectors.DefaultSelector() as selector:
                selector.register(process.stdout, selectors.EVENT_READ)
                self.assertTrue(selector.select(timeout=3), 'MCP response timed out')
            return json.loads(process.stdout.readline())

        send({'jsonrpc': '2.0', 'id': 1, 'method': 'initialize', 'params': {
            'protocolVersion': '2025-06-18', 'capabilities': {}, 'clientInfo': {'name': 'acceptance', 'version': '1'}}})
        self.assertEqual(response()['result']['protocolVersion'], '2025-06-18')
        send({'jsonrpc': '2.0', 'method': 'notifications/initialized'})
        send({'jsonrpc': '2.0', 'id': 2, 'method': 'tools/list'})
        tools = response()['result']['tools']
        self.assertEqual({tool['name'] for tool in tools},
                         {'device_list', 'device_describe', 'device_get_state', 'execution_get'})
        self.assertTrue(all(tool['annotations']['readOnlyHint'] for tool in tools))
        send({'jsonrpc': '2.0', 'id': 3, 'method': 'tools/call', 'params': {'name': 'device_list', 'arguments': {}}})
        listed = response()['result']
        self.assertFalse(listed['isError'])
        self.assertEqual(listed['structuredContent']['data'][0]['id'], 'dev-mock-ac')
        for index, name, arguments in [(4, 'power_set', {'device': 'dev-mock-ac', 'state': True}),
                                       (5, 'device_get_state', {'device': 'dev-mock-ac', 'principal': 'admin'}),
                                       (6, 'create_room', {'name': 'Untrusted room'}),
                                       (7, 'execution_get', {'request_id': 'mcp-receipt', 'principal': 'admin'})]:
            send({'jsonrpc': '2.0', 'id': index, 'method': 'tools/call', 'params': {'name': name, 'arguments': arguments}})
            self.assertTrue(response()['result']['isError'])
        send({'jsonrpc': '2.0', 'id': 8, 'method': 'tools/call',
              'params': {'name': 'execution_get', 'arguments': {'request_id': 'mcp-receipt'}}})
        looked_up = response()['result']
        self.assertFalse(looked_up['isError'])
        self.assertEqual(looked_up['structuredContent']['data'], receipt)
        send({'jsonrpc': '2.0', 'id': 9, 'method': 'tools/call',
              'params': {'name': 'execution_get', 'arguments': {'request_id': 'missing'}}})
        missing = response()['result']
        self.assertTrue(missing['isError'])
        self.assertEqual(missing['structuredContent']['error']['code'], 'REQUEST_NOT_FOUND')
        # Tokio may use Unix socket pairs for signals; check Internet sockets only.
        if Path(f'/proc/{process.pid}/fd').exists():
            socket_inodes = set()
            for descriptor in Path(f'/proc/{process.pid}/fd').iterdir():
                try:
                    target = os.readlink(descriptor)
                    if target.startswith('socket:['):
                        socket_inodes.add(target[8:-1])
                except FileNotFoundError:
                    pass
            for table in ['tcp', 'tcp6', 'udp', 'udp6']:
                for row in Path(f'/proc/{process.pid}/net/{table}').read_text().splitlines()[1:]:
                    self.assertNotIn(row.split()[9], socket_inodes)
        process.stdin.close()
        process.wait(timeout=2)
        self.assertEqual(process.returncode, 0)
        self.assertEqual(process.stderr.read(), '')
        process.stdout.close()
        process.stderr.close()

    def test_native_bridge_independence_and_foreground_lifecycle(self):
        library = ctypes.CDLL(str(LIBRARY))
        library.homeverb_open.argtypes = [ctypes.c_char_p]
        library.homeverb_open.restype = ctypes.c_void_p
        library.homeverb_call.argtypes = [ctypes.c_uint64, ctypes.c_char_p]
        library.homeverb_call.restype = ctypes.c_void_p
        library.homeverb_string_free.argtypes = [ctypes.c_void_p]
        library.homeverb_set_foreground.argtypes = [ctypes.c_uint64, ctypes.c_bool]
        library.homeverb_close.argtypes = [ctypes.c_uint64]

        def decode(pointer):
            self.assertTrue(pointer)
            try:
                return json.loads(ctypes.string_at(pointer))
            finally:
                library.homeverb_string_free(pointer)

        directory = Path(self.directory.name) / 'native'
        opened = decode(library.homeverb_open(json.dumps({'schema_version': '1.0', 'data_dir': str(directory),
                                                         'allow_mock_writes': False, 'scenario': 'slow-after-send'}).encode()))
        handle = opened['data']['handle']
        self.addCleanup(library.homeverb_close, handle)

        def call(operation, request):
            return decode(library.homeverb_call(handle, json.dumps({'schema_version': '1.0', 'operation_id': operation,
                          'timeout_ms': 10000, 'request': request}).encode()))

        self.assertEqual(call('list-1', {'method': 'list_devices'})['data'], [])
        self.assertEqual(call('discover-1', {'method': 'discover'})['data'][0]['id'], 'dev-mock-ac')
        room = call('room-1', {'method': 'create_room', 'name': 'Bedroom'})['data']
        self.schema('room', room)
        preferences = {'alias': 'Bedroom AC', 'room_id': room['id'], 'favorite': True}
        updated = call('preferences-1', {'method': 'set_device_preferences', 'device_id': 'dev-mock-ac',
                                       'preferences': preferences})['data']
        self.schema('device-preferences', updated)
        self.assertEqual(call('cards-1', {'method': 'list_device_cards'})['data'][0]['preferences'], updated)
        assignment = {'device_ids': ['dev-mock-ac'], 'room_id': None}
        self.schema('room-assignment', assignment)
        moved = call('room-unassign', {'method': 'assign_devices_to_room', 'assignment': assignment})['data']
        self.assertIsNone(moved[0]['room_id'])
        self.assertEqual(moved[0]['alias'], preferences['alias'])
        self.assertTrue(moved[0]['favorite'])
        invalid = call('room-omitted', {'method': 'assign_devices_to_room', 'assignment': {'device_ids': ['dev-mock-ac']}})
        self.assertEqual(invalid['error']['code'], 'INVALID_ARGUMENT')
        self.assertIsNone(call('room-delete', {'method': 'delete_room', 'room_id': room['id']})['error'])
        self.assertIsNone(call('cards-2', {'method': 'list_device_cards'})['data'][0]['preferences']['room_id'])
        command = {'schema_version': '1.0', 'request_id': 'native-write', 'device_id': 'dev-mock-ac', 'component_id': 'main',
                   'capability': 'power', 'action': 'set', 'arguments': {'state': True}, 'connection': 'auto',
                   'timeout_ms': 10000, 'wait_for': 'verified'}
        denied = call('write-denied', {'method': 'execute', 'command': command})
        self.schema('command-result', denied)
        self.assertEqual(denied['error']['code'], 'POLICY_DENIED')
        self.assertEqual(denied['attempts'], [])
        for index, extra in enumerate([{'principal': 'admin'}, {'grants': [{'transports': ['cloud']}]}, {'confirmed': True}]):
            forged = call(f'forged-{index}', {'method': 'execute', 'command': command | extra})
            self.assertEqual(forged['error']['code'], 'INVALID_ARGUMENT')
        self.assertEqual(call('no-denied-journal', {'method': 'recent_executions'})['data'], [])
        library.homeverb_close(handle)
        opened = decode(library.homeverb_open(json.dumps({'schema_version': '1.0', 'data_dir': str(directory),
                                                         'allow_mock_writes': True, 'scenario': 'slow-after-send'}).encode()))
        handle = opened['data']['handle']
        self.addCleanup(library.homeverb_close, handle)
        replies = []
        worker = threading.Thread(target=lambda: replies.append(call('write-1', {'method': 'execute', 'command': command})))
        worker.start()
        deadline = time.monotonic() + 3
        observation_index = 0
        while time.monotonic() < deadline:
            # Read through the ABI; two SQLite builds in one process must not share file locks.
            observation_index += 1
            snapshot = call(f'observe-{observation_index}', {'method': 'read_state', 'device': 'dev-mock-ac',
                            'component': 'main', 'connection': 'auto', 'cached': False})['data']
            if snapshot['observations']['power.state']['value']:
                break
            time.sleep(0.01)
        self.assertTrue(snapshot['observations']['power.state']['value'])
        pending = call('inspect-pending', {'method': 'get_execution', 'request_id': command['request_id']})['data']
        self.schema('command-result', pending)
        self.assertEqual(pending['status'], 'unknown')
        library.homeverb_set_foreground(handle, False)
        worker.join(timeout=2)
        self.assertFalse(worker.is_alive())
        self.assertEqual(replies[0]['status'], 'unknown')
        self.assertEqual(call('list-2', {'method': 'list_devices'})['error']['code'], 'HOST_BACKGROUND')
        self.assertEqual(call('inspect-background', {'method': 'get_execution',
                         'request_id': command['request_id']})['error']['code'], 'HOST_BACKGROUND')
        library.homeverb_set_foreground(handle, True)
        self.assertEqual(len(call('list-3', {'method': 'list_devices'})['data']), 1)
        history = call('history-device', {'method': 'device_executions', 'device_id': 'dev-mock-ac'})['data']
        self.assertEqual(len(history), 1)
        self.assertEqual(history[0]['status'], 'unknown')
        self.assertEqual(history[0]['request_id'], command['request_id'])
        looked_up = call('inspect-recorded', {'method': 'get_execution', 'request_id': command['request_id']})
        self.assertIsNone(looked_up['error'])
        self.assertEqual(looked_up['data'], history[0])
        self.assertEqual(call('inspect-missing', {'method': 'get_execution', 'request_id': 'missing'})['error']['code'],
                         'REQUEST_NOT_FOUND')
        self.assertEqual(call('inspect-forged', {'method': 'get_execution', 'request_id': command['request_id'],
                         'confirmed': True})['error']['code'], 'INVALID_ARGUMENT')
        report = call('diagnostics-1', {'method': 'diagnostics'})['data']
        self.schema('diagnostics', report)
        self.assertEqual(report['device_count'], 1)
        self.assertEqual(report['recent_receipts'][0]['status'], 'unknown')
        self.assertEqual(report['recent_receipts'][0]['error_code'], 'WRITE_OUTCOME_UNKNOWN')
        self.assertNotIn(command['request_id'], json.dumps(report))
        self.assertNotIn('dev-mock-ac', json.dumps(report))
        rejected_report = call('diagnostics-forged', {'method': 'diagnostics', 'include_secrets': True})
        self.assertEqual(rejected_report['error']['code'], 'INVALID_ARGUMENT')
        library.homeverb_close(handle)
        self.assertEqual(call('list-4', {'method': 'list_devices'})['error']['code'], 'HOST_CLOSED')

    def test_mcp_sigint_exits_even_when_host_stdin_remains_open(self):
        process = subprocess.Popen([*self.base, 'mcp', '--stdio'], stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        self.addCleanup(self.close_process, process)
        process.stdin.write(json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'ping'}) + '\n')
        process.stdin.flush()
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            self.assertTrue(selector.select(timeout=3))
        self.assertEqual(json.loads(process.stdout.readline())['result'], {})
        process.send_signal(signal.SIGINT)
        process.wait(timeout=2)
        self.assertEqual(process.returncode, 0)
        self.assertEqual(process.stderr.read(), '')


if __name__ == '__main__':
    if not BINARY.is_file() or not LIBRARY.is_file():
        raise SystemExit('Run cargo build --workspace before runtime acceptance (Linux native library required).')
    unittest.main(verbosity=2)
