"""Acceptance must require an emulator and positive instrumentation evidence."""

import contextlib
import importlib.util
import io
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
    def invoke(self, replies):
        output = io.StringIO()
        with patch('sys.argv', ARGV), patch.object(MODULE.subprocess, 'run', side_effect=replies) as run:
            with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
                try:
                    MODULE.main()
                    code = 0
                except SystemExit as error:
                    code = error.code
            return code, output.getvalue(), run.call_args_list

    def test_physical_device_is_refused_before_instrumentation(self):
        code, _, calls = self.invoke([subprocess.CompletedProcess([], 0, '0\n', '')])
        self.assertNotEqual(code, 0)
        self.assertEqual(len(calls), 1)

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
