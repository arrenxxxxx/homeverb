"""Refuse stale/incomplete runtime evidence and protect existing simulators."""

from pathlib import Path
import json
import sys
import subprocess
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import verify_ios_acceptance as ios

DEVICE = '00000000-0000-4000-8000-000000000001'
RUN = '00000000-0000-4000-8000-000000000002'


def report(phase='write'):
    return {'run_id': RUN, 'phase': phase, 'pid': 101 if phase == 'write' else 102,
            'writer_pid': 101, 'status': 'passed', 'stage': 'complete',
            'core_checks': 40, 'store_code': 0, 'store_checks': ios.STORE_CHECKS[phase]}


class IosAcceptanceTests(unittest.TestCase):
    def test_only_available_compatible_iphone_runtime_is_selected(self):
        runtimes = [
            {'identifier': 'com.apple.CoreSimulator.SimRuntime.iOS-18-0', 'version': '18.0', 'isAvailable': True},
            {'identifier': 'com.apple.CoreSimulator.SimRuntime.iOS-26-0', 'version': '26.0', 'isAvailable': False},
            {'identifier': 'com.apple.CoreSimulator.SimRuntime.tvOS-26-0', 'version': '26.0', 'isAvailable': True},
        ]
        devices = [
            {'identifier': 'com.apple.CoreSimulator.SimDeviceType.iPhone-Test', 'productFamily': 'iPhone',
             'minRuntimeVersion': 17 << 16, 'maxRuntimeVersion': (19 << 16) - 1},
            {'identifier': 'com.apple.CoreSimulator.SimDeviceType.iPad-Test', 'productFamily': 'iPad',
             'minRuntimeVersion': 17 << 16, 'maxRuntimeVersion': 99 << 16},
        ]
        self.assertEqual(ios.select_runtime(runtimes, devices), (runtimes[0]['identifier'], devices[0]['identifier']))
        for unavailable in [[], [dict(runtimes[0], version='16.0')], [runtimes[1]], [runtimes[2]]]:
            with self.assertRaises(ValueError):
                ios.select_runtime(unavailable, devices)

    def test_launch_requires_an_exact_positive_test_app_pid(self):
        self.assertEqual(ios.launched_pid(ios.BUNDLE + ': 101\n'), 101)
        for value in ['other: 101', ios.BUNDLE + ': 0', ios.BUNDLE + ': -1', '101', ios.BUNDLE + ': 101\nother: 102']:
            with self.assertRaises(ValueError):
                ios.launched_pid(value)

    def test_matching_reports_require_all_assertions_and_distinct_processes(self):
        for phase in ['write', 'read']:
            valid = report(phase)
            ios.validate_report(valid, phase, RUN, valid['pid'], 101)
            for key, value in [('status', 'failed'), ('stage', 'keychain'), ('store_code', 4),
                               ('store_checks', 0), ('core_checks', 39), ('run_id', DEVICE),
                               ('phase', 'other'), ('pid', 999), ('writer_pid', 99), ('store_code', False)]:
                with self.subTest(phase=phase, key=key), self.assertRaises(ValueError):
                    ios.validate_report({**valid, key: value}, phase, RUN, valid['pid'], 101)
            for invalid in [None, [], {}, {**valid, 'extra': 'fictional unapproved output'}]:
                with self.assertRaises(ValueError):
                    ios.validate_report(invalid, phase, RUN, valid['pid'], 101)
        with self.assertRaises(ValueError):
            ios.validate_report(dict(report('read'), pid=101), 'read', RUN, 101, 101)

    def test_missing_report_times_out_without_relaunching(self):
        with tempfile.TemporaryDirectory() as directory, self.assertRaisesRegex(ValueError, 'no app restart'):
            ios.wait_report(Path(directory) / 'missing.json', 'write', RUN, 101, 101, timeout=0)

    def test_report_failures_do_not_echo_arbitrary_strings(self):
        with self.assertRaises(ValueError) as failure:
            ios.validate_report(dict(report(), stage='fictional-sensitive-text', store_code='fictional-secret'),
                                'write', RUN, 101, 101)
        self.assertNotIn('fictional', str(failure.exception))

    def test_signed_entitlements_must_match_exact_private_scope(self):
        valid = {'application-identifier': ios.GROUP, 'keychain-access-groups': [ios.GROUP]}
        ios.validate_entitlements(valid)
        for invalid in [{}, {**valid, 'get-task-allow': True},
                        {**valid, 'keychain-access-groups': [ios.GROUP, 'SYNTHETIC.shared']},
                        {**valid, 'application-identifier': 'SYNTHETIC.other'}]:
            with self.assertRaises(ValueError):
                ios.validate_entitlements(invalid)

    def test_failure_cleans_only_the_new_simulator_and_never_retries_launch(self):
        for fail_at in ['boot', 'bootstatus', 'install', 'launch', 'report', None]:
            with self.subTest(fail_at=fail_at), tempfile.TemporaryDirectory() as directory:
                calls = []

                def command(*args, **kwargs):
                    calls.append(args)
                    action = args[2]
                    if action == 'create':
                        return DEVICE
                    self.assertEqual(args[3], DEVICE)
                    if action == fail_at:
                        raise ValueError('Synthetic command failure.')
                    if action == 'get_app_container':
                        return directory
                    if action == 'launch':
                        return ios.BUNDLE + (': 101' if args[5] == 'write' else ': 102')
                    return ''

                def wait(path, phase, run_id, pid, writer_pid):
                    if fail_at == 'report':
                        raise ValueError('Synthetic evidence failure.')
                    return report(phase)

                with patch.object(ios, 'command', side_effect=command), patch.object(ios, 'wait_report', side_effect=wait):
                    if fail_at:
                        with self.assertRaises(ValueError):
                            ios.exercise(Path(directory) / 'app', 'runtime', 'type')
                    else:
                        self.assertEqual(len(ios.exercise(Path(directory) / 'app', 'runtime', 'type')), 2)
                self.assertEqual([call[2] for call in calls[-2:]], ['shutdown', 'delete'])
                launches = [call for call in calls if call[2] == 'launch']
                self.assertEqual(len(launches), 2 if fail_at is None else int(fail_at in ['launch', 'report']))

    def test_invalid_create_output_never_targets_existing_devices(self):
        for output in ['booted', 'all', '', 'existing-device']:
            with patch.object(ios, 'command', return_value=output) as command, self.assertRaises(ValueError):
                ios.exercise(Path('/fictional/app'), 'runtime', 'type')
            self.assertEqual(command.call_count, 1)

    def test_launch_failure_diagnostics_stay_on_the_created_device_without_retry(self):
        with tempfile.TemporaryDirectory() as directory:
            calls = []

            def command(*args, **kwargs):
                calls.append(args)
                action = args[2]
                if action == 'create':
                    return DEVICE
                self.assertEqual(args[3], DEVICE)
                if action == 'get_app_container':
                    return directory
                if action in ['launch', 'spawn']:
                    raise subprocess.CalledProcessError(1, args)
                return ''

            with patch.object(ios, 'command', side_effect=command), self.assertRaises(subprocess.CalledProcessError):
                ios.exercise(Path(directory) / 'app', 'runtime', 'type')
            self.assertEqual([call[2] for call in calls].count('launch'), 1)
            diagnostics = [call for call in calls if call[2] == 'spawn']
            self.assertEqual(len(diagnostics), 1)
            self.assertIn('org.homeverb.test', diagnostics[0][-1])
            self.assertEqual([call[2] for call in calls[-2:]], ['shutdown', 'delete'])

    def test_launch_failure_is_not_hidden_by_later_ui_noise(self):
        records = [{'eventMessage': 'Launch failed: fictional incompatible executable.'}]
        records.extend({'eventMessage': 'Fictional AppSwitcher transition.'} for _ in range(1000))
        result = ios.launch_failure_messages(json.dumps(records))
        self.assertIn('incompatible executable', result)
        self.assertNotIn('AppSwitcher', result)
        self.assertLess(len(result), 100)


if __name__ == '__main__':
    unittest.main()
