"""Acceptance must require an emulator and positive instrumentation evidence."""

import contextlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    'android_acceptance', Path(__file__).resolve().parents[1] / 'scripts/verify_android_acceptance.py')
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
ARGV = ['verify', '--adb-port', '5047', '--serial', 'emulator-5588', '--phase', 'write']


class AndroidAcceptanceTests(unittest.TestCase):
    def test_network_failure_exports_only_fixed_codes(self):
        for value, expected in [('LAN_NETWORK_UNAVAILABLE', 'LAN_NETWORK_UNAVAILABLE'),
                                ('PRIVATE_NETWORK_DETAIL', 'unavailable')]:
            result = f'INSTRUMENTATION_RESULT: network_error={value}\n'
            code, output, _ = self.invoke([subprocess.CompletedProcess([], 0, '1\n', ''),
                                          subprocess.CompletedProcess([], 0, result, '')], 'networks')
            self.assertEqual(code, 1)
            self.assertIn(f'network error: {expected}', output)
            self.assertNotIn('PRIVATE', output)

    def invoke(self, replies, phase='write'):
        output = io.StringIO()
        with patch('sys.argv', ARGV[:-1] + [phase]), patch.object(MODULE.subprocess, 'run', side_effect=replies) as run:
            with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
                try:
                    MODULE.main()
                    code = 0
                except SystemExit as error:
                    code = error.code
            return code, output.getvalue(), run.call_args_list

    def test_physical_device_is_refused_before_instrumentation(self):
        for phase in ['write', 'force-stop', 'networks']:
            with self.subTest(phase=phase), patch.object(MODULE.subprocess, 'Popen') as start:
                code, _, calls = self.invoke([subprocess.CompletedProcess([], 0, '0\n', '')], phase)
                self.assertNotEqual(code, 0)
                self.assertEqual(len(calls), 1)
                start.assert_not_called()

    def test_exit_zero_without_assertion_success_is_not_a_pass(self):
        for text in ['', 'INSTRUMENTATION_CODE: 0\n',
                     'INSTRUMENTATION_RESULT: homeverb_result=PASS\nINSTRUMENTATION_CODE: -1\n',
                     'INSTRUMENTATION_RESULT: completed_assertions=4\nPRIVATE-DETAIL\nINSTRUMENTATION_CODE: -1\n',
                     'INSTRUMENTATION_RESULT: assertions=0\nINSTRUMENTATION_RESULT: phase=write\nINSTRUMENTATION_RESULT: homeverb_result=PASS\nINSTRUMENTATION_CODE: -1\n']:
            with self.subTest(output=text):
                code, output, _ = self.invoke([subprocess.CompletedProcess([], 0, '1\n', ''),
                                               subprocess.CompletedProcess([], 0, text, 'PRIVATE-ERROR')])
                self.assertEqual(code, 1)
                self.assertNotIn('PRIVATE-', output)

    def test_success_requires_matching_phase_and_positive_completion(self):
        result = 'INSTRUMENTATION_RESULT: assertions=12\nINSTRUMENTATION_RESULT: phase=write\nINSTRUMENTATION_RESULT: homeverb_result=PASS\nINSTRUMENTATION_CODE: -1\n'
        code, output, calls = self.invoke([subprocess.CompletedProcess([], 0, '1\n', ''),
                                           subprocess.CompletedProcess([], 0, result, '')])
        self.assertEqual(code, 0)
        self.assertIn('12 assertions', output)
        self.assertEqual(calls[1].args[0][-3:], ['phase', 'write', MODULE.COMPONENT])
        self.assertEqual(calls[1].args[0][1:5], ['-P', '5047', '-s', 'emulator-5588'])

    def test_adb_failure_overrides_success_text(self):
        result = 'INSTRUMENTATION_RESULT: assertions=12\nINSTRUMENTATION_RESULT: phase=write\nINSTRUMENTATION_RESULT: homeverb_result=PASS\nINSTRUMENTATION_CODE: -1\n'
        code, _, _ = self.invoke([subprocess.CompletedProcess([], 0, '1\n', ''),
                                  subprocess.CompletedProcess([], 1, result, '')])
        self.assertEqual(code, 1)


class ForceStopEvidenceTests(unittest.TestCase):
    adb = ['adb', '-P', '5047', '-s', 'emulator-5588']

    class Observer:
        def __init__(self, ended=False):
            self.returncode = 0 if ended else None

        def poll(self):
            return self.returncode

        def wait(self, timeout):
            self.returncode = 0
            return 0

        def terminate(self):
            self.returncode = -15

    @staticmethod
    def reply(text='', code=0):
        return subprocess.CompletedProcess([], code, text, '')

    def invoke(self, replies, *, ended=False, ticks=None):
        observer = self.Observer(ended)
        with patch.object(MODULE.subprocess, 'Popen', return_value=observer), \
             patch.object(MODULE.subprocess, 'run', side_effect=replies) as calls, \
             patch.object(MODULE.time, 'monotonic', side_effect=ticks or [0, 0, 0, 0]), \
             patch.object(MODULE.time, 'sleep'):
            MODULE.interrupt_write(self.adb, 'current-run')
        return calls.call_args_list

    def test_live_matching_write_is_stopped_and_absence_verified(self):
        ready = json.dumps({'run_id': 'current-run', 'pid': 1234})
        calls = self.invoke([self.reply(ready), self.reply('1234\n'), self.reply(), self.reply(code=1)])
        self.assertEqual(calls[1].args[0], self.adb + ['shell', 'pidof', MODULE.PACKAGE])
        self.assertEqual(calls[2].args[0], self.adb + ['shell', 'am', 'force-stop', MODULE.PACKAGE])
        self.assertEqual(calls[3].args[0], self.adb + ['shell', 'pidof', MODULE.PACKAGE])

    def test_stale_or_malformed_ready_files_never_establish_live_write_evidence(self):
        for marker in [{'run_id': 'old-run', 'pid': 1234}, {'run_id': 'current-run', 'pid': True},
                       {'run_id': 'current-run', 'pid': 1234, 'approved': True}]:
            with self.subTest(marker=marker), self.assertRaises(RuntimeError):
                self.invoke([self.reply(json.dumps(marker)), self.reply()], ticks=[0, 0, 16])

    def test_successful_observer_exit_is_not_a_forced_interruption(self):
        with self.assertRaises(RuntimeError):
            self.invoke([self.reply()], ended=True)

    def test_missing_process_before_stop_refuses_the_claim(self):
        ready = json.dumps({'run_id': 'current-run', 'pid': 1234})
        for live in [self.reply(code=1), self.reply('5678\n')]:
            with self.subTest(pid=live.stdout), self.assertRaises(RuntimeError):
                self.invoke([self.reply(ready), live, self.reply()])

    def test_successful_force_stop_command_requires_process_absence(self):
        ready = json.dumps({'run_id': 'current-run', 'pid': 1234})
        with self.assertRaises(RuntimeError):
            self.invoke([self.reply(ready), self.reply('1234\n'), self.reply(),
                         self.reply('1234\n'), self.reply()], ticks=[0, 0, 0, 0, 6])

    def test_observer_completion_waits_for_actual_process_disappearance(self):
        ready = json.dumps({'run_id': 'current-run', 'pid': 1234})
        calls = self.invoke([self.reply(ready), self.reply('1234\n'), self.reply(),
                             self.reply('1234\n'), self.reply(code=1)], ticks=[0, 0, 0, 0, 1])
        self.assertEqual(sum(call.args[0][-3:] == ['am', 'force-stop', MODULE.PACKAGE] for call in calls), 1)
