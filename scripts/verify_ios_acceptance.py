#!/usr/bin/env python3
"""Exercise a fictional native app on a newly created, disposable iOS Simulator."""

import json
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import uuid

import build_ios_native as package

ROOT = package.ROOT
BUNDLE = 'org.homeverb.test'
GROUP = 'SYNTHETIC.' + BUNDLE
STORE_CHECKS = {'write': 10, 'read': 4}


def command(*args, timeout=60, **kwargs):
    return subprocess.check_output(list(map(str, args)), text=True, timeout=timeout, **kwargs).strip()


def select_runtime(runtimes, device_types):
    available = [runtime for runtime in runtimes if runtime.get('isAvailable') is True
                 and runtime.get('identifier', '').startswith('com.apple.CoreSimulator.SimRuntime.iOS-')]
    for runtime in sorted(available, key=lambda item: package.version(item['version']), reverse=True):
        major, minor, patch = package.version(runtime['version'])
        numeric = (major << 16) | (minor << 8) | patch
        devices = [device for device in device_types if device.get('productFamily') == 'iPhone'
                   and device.get('identifier', '').startswith('com.apple.CoreSimulator.SimDeviceType.')
                   and device.get('minRuntimeVersion', 0) <= numeric <= device.get('maxRuntimeVersion', 0)]
        if devices and major >= 14:
            return runtime['identifier'], sorted(devices, key=lambda item: item['identifier'])[-1]['identifier']
    raise ValueError('No installed iOS 14+ Simulator runtime with a compatible iPhone type.')


def launched_pid(output):
    match = re.fullmatch(re.escape(BUNDLE) + r': ([1-9][0-9]*)', output.strip())
    if not match:
        raise ValueError('Simulator launch did not return the exact test app PID.')
    return int(match[1])


def validate_report(report, phase, run_id, pid, writer_pid):
    expected = {'run_id': run_id, 'phase': phase, 'pid': pid, 'writer_pid': writer_pid,
                'status': 'passed', 'stage': 'complete', 'store_code': 0,
                'store_checks': STORE_CHECKS[phase]}
    if (not isinstance(report, dict) or set(report) != set(expected) | {'core_checks'}
            or any(type(report.get(key)) is not type(value) or report[key] != value for key, value in expected.items())
            or type(report['core_checks']) is not int or report['core_checks'] < 40
            or (phase == 'read' and pid == writer_pid)):
        # Reports are a closed test format; do not echo arbitrary JSON or app logs.
        stage = report.get('stage') if isinstance(report, dict) else None
        code = report.get('store_code') if isinstance(report, dict) else None
        count = report.get('store_checks') if isinstance(report, dict) else None
        safe_stage = stage if stage in {'startup', 'process_identity', 'core_open', 'core_api', 'core_discover',
                                       'core_execute', 'core_read', 'core_persistence', 'core_lifecycle',
                                       'core_closed', 'keychain', 'complete'} else 'invalid'
        safe_code = code if type(code) is int and 0 <= code <= 7 else -1
        safe_count = count if type(count) is int and 0 <= count <= 10 else -1
        shape = report.get('metadata_shape', {}) if isinstance(report, dict) else {}
        safe_shape = {key: value for key, value in shape.items()
                      if key in {'array', 'dictionary', 'scope', 'service', 'account', 'passcode', 'sync_present', 'sync_boolean',
                                 'sync_number', 'sync_zero', 'access_control', 'persistent_ref'}
                      and type(value) is bool} if isinstance(shape, dict) else {}
        raise ValueError(f'Acceptance evidence refused: phase={phase}, stage={safe_stage}, store_code={safe_code}, store_checks={safe_count}, metadata_shape={safe_shape}.')


def wait_report(path, phase, run_id, pid, writer_pid, timeout=60):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.is_file():
            report = json.loads(path.read_text())
            validate_report(report, phase, run_id, pid, writer_pid)
            return report
        time.sleep(0.25)
    raise ValueError(f'Missing acceptance report for {phase}; no app restart or write retry was attempted.')


def validate_entitlements(entitlements):
    if entitlements != {'application-identifier': GROUP, 'keychain-access-groups': [GROUP]}:
        raise ValueError('The isolated app must have only its exact fictional private Keychain scope.')


def validate_simulator_entitlements(xml, der, expected_der, signed):
    validate_entitlements(plistlib.loads(xml))
    if not expected_der or der != expected_der:
        raise ValueError('The embedded Simulator DER entitlements differ from the generated scope.')
    if signed.strip() and plistlib.loads(signed) != {}:
        raise ValueError('iOS Simulator entitlements belong in Mach-O sections, not the host code signature.')


def launch_failure_messages(log):
    records = json.loads(log)
    messages = [record.get('eventMessage', '') for record in records if isinstance(record, dict)]
    failures = [message for message in messages if isinstance(message, str)
                and re.search(r'fail|denied|error|unsupported|invalid|incompatible|refus', message, re.IGNORECASE)]
    return '\n'.join(message[:4000] for message in failures[-12:]) or 'No matching failure messages.'


def build_app(directory):
    architecture = platform.machine()
    if architecture not in package.DESTINATIONS['simulator']['targets']:
        raise ValueError('Unsupported macOS runner architecture.')
    target = package.DESTINATIONS['simulator']['targets'][architecture]
    environment = {**os.environ, 'IPHONEOS_DEPLOYMENT_TARGET': '14.0'}
    command('cargo', 'build', '-p', 'homeverb-secrets', '--example', 'ios_store_probe',
            '--target', target, '--target-dir', ROOT / 'target', '--release', '--offline', '--locked',
            cwd=ROOT, env=environment, timeout=600)
    archive = ROOT / 'target/ios/homeverb-core.xcframework.zip'
    command('ditto', '-x', '-k', archive, directory / 'extracted')
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
    framework = package.inspect_package(directory / 'extracted/HomeVerb.xcframework', version)['simulator']
    symbols = command('xcrun', 'nm', '-arch', architecture, '-gU', framework / 'HomeVerb')
    if 'homeverb_test_' in symbols:
        raise ValueError('Test-only exports must not enter the production framework.')
    app = directory / 'HomeVerbAcceptance.app'
    (app / 'Frameworks').mkdir(parents=True)
    shutil.copytree(framework, app / 'Frameworks/HomeVerb.framework')
    info = {
        'CFBundleIdentifier': BUNDLE, 'CFBundleName': 'HomeVerbAcceptance',
        'CFBundleExecutable': 'HomeVerbAcceptance', 'CFBundlePackageType': 'APPL',
        'CFBundleVersion': '1', 'CFBundleShortVersionString': '1.0',
        'MinimumOSVersion': '14.0', 'CFBundleSupportedPlatforms': ['iPhoneSimulator'],
        'UIDeviceFamily': [1], 'LSRequiresIPhoneOS': True, 'UILaunchScreen': {},
    }
    (app / 'Info.plist').write_bytes(plistlib.dumps(info))
    entitlements = directory / 'entitlements.plist'
    entitlements.write_bytes(plistlib.dumps({'application-identifier': GROUP, 'keychain-access-groups': [GROUP]}))
    der = directory / 'entitlements.der'
    command('/usr/bin/derq', 'query', '--raw', '-f', 'xml', '-i', entitlements, '-o', der)
    sdk = command('xcrun', '--sdk', 'iphonesimulator', '--show-sdk-path')
    command('xcrun', '--sdk', 'iphonesimulator', 'clang', '-target', f'{architecture}-apple-ios14.0-simulator',
            '-isysroot', sdk, '-Wall', '-Wextra', '-Werror', '-fobjc-arc', '-fmodules',
            '-F', app / 'Frameworks', ROOT / 'packages/ios/acceptance/main.m',
            ROOT / f'target/{target}/release/examples/libios_store_probe.a',
            '-framework', 'HomeVerb', '-framework', 'UIKit', '-framework', 'Foundation',
            '-framework', 'Security', '-framework', 'CoreFoundation', '-lresolv', '-liconv',
            '-Xlinker', '-sectcreate', '-Xlinker', '__TEXT', '-Xlinker', '__entitlements', '-Xlinker', entitlements,
            '-Xlinker', '-sectcreate', '-Xlinker', '__TEXT', '-Xlinker', '__ents_der', '-Xlinker', der,
            '-Wl,-rpath,@executable_path/Frameworks', '-o', app / 'HomeVerbAcceptance', timeout=120)
    # Simulator entitlements are interpreted by iOS from these embedded sections.
    # The macOS kernel must not receive iOS-only entitlements in the code signature.
    command('codesign', '--force', '--sign', '-', '--timestamp=none', app)
    command('codesign', '--verify', '--deep', '--strict', app)
    xml_copy, der_copy = directory / 'embedded.plist', directory / 'embedded.der'
    command('xcrun', 'segedit', app / 'HomeVerbAcceptance',
            '-extract', '__TEXT', '__entitlements', xml_copy, '-extract', '__TEXT', '__ents_der', der_copy)
    signed = command('codesign', '--display', '--entitlements', '-', '--xml', app)
    validate_simulator_entitlements(xml_copy.read_bytes(), der_copy.read_bytes(), der.read_bytes(), signed.encode())
    return app


def exercise(app, runtime, device_type):
    run_id = str(uuid.uuid4())
    device = command('xcrun', 'simctl', 'create', 'HomeVerbAcceptance-' + run_id, device_type, runtime)
    if not re.fullmatch(r'[0-9A-Fa-f]{8}(?:-[0-9A-Fa-f]{4}){3}-[0-9A-Fa-f]{12}', device):
        raise ValueError('Simulator creation did not return a UUID; refuse all device operations.')
    try:
        command('xcrun', 'simctl', 'boot', device)
        command('xcrun', 'simctl', 'bootstatus', device, '-b', timeout=240)
        command('xcrun', 'simctl', 'install', device, app)
        container = Path(command('xcrun', 'simctl', 'get_app_container', device, BUNDLE, 'data'))
        if not container.is_absolute() or not container.is_dir():
            raise ValueError('Simulator returned an invalid app data container.')
        writer_pid = None
        reports = []
        for phase in ['write', 'read']:
            path = container / 'Documents' / (phase + '.json')
            if path.exists():
                raise ValueError('Refuse a pre-existing acceptance report.')
            try:
                pid = launched_pid(command('xcrun', 'simctl', 'launch', device, BUNDLE, phase, run_id))
            except subprocess.SubprocessError:
                # Only this fresh fictional Simulator is queried; never collect host/user logs.
                predicate = f'eventMessage CONTAINS "{BUNDLE}" OR eventMessage CONTAINS "HomeVerbAcceptance"'
                try:
                    log = command('xcrun', 'simctl', 'spawn', device, 'log', 'show', '--last', '2m',
                                  '--style', 'json', '--predicate', predicate, timeout=30)
                    print('Isolated test-app launch diagnostics:\n' + launch_failure_messages(log), flush=True)
                except (ValueError, subprocess.SubprocessError):
                    print('Isolated test-app launch diagnostics unavailable.', flush=True)
                raise
            if phase == 'write':
                writer_pid = pid
            report = wait_report(path, phase, run_id, pid, writer_pid)
            reports.append(report)
            print(f'PASS: iOS Simulator {phase}: Core checks={report["core_checks"]}, Keychain checks={report["store_checks"]}.', flush=True)
            command('xcrun', 'simctl', 'terminate', device, BUNDLE)
        return reports
    finally:
        # Cleanup is scoped to the one UUID returned by this invocation's create.
        try:
            command('xcrun', 'simctl', 'shutdown', device)
        finally:
            command('xcrun', 'simctl', 'delete', device)


def main():
    try:
        if sys.platform != 'darwin':
            raise ValueError('The isolated iOS acceptance runner requires macOS and Xcode.')
        if package.require_group(os.environ) != GROUP:
            raise ValueError('Use the documented fictional group for this test-only Simulator build.')
        runtimes = json.loads(command('xcrun', 'simctl', 'list', 'runtimes', '--json'))['runtimes']
        device_types = json.loads(command('xcrun', 'simctl', 'list', 'devicetypes', '--json'))['devicetypes']
        runtime, device_type = select_runtime(runtimes, device_types)
        print(f'Isolated runtime: {runtime}; device type: {device_type}.', flush=True)
        with tempfile.TemporaryDirectory(prefix='homeverb-ios-acceptance-') as temporary:
            app = build_app(Path(temporary))
            exercise(app, runtime, device_type)
        print('PASS: production framework/Core and fictional Keychain persistence in separate Simulator processes.')
        print('No physical-device, passcode/lock policy, provider, Flutter or appliance acceptance.')
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f'FAIL: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
