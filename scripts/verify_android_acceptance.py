#!/usr/bin/env python3
"""Run one native acceptance phase on an explicitly selected disposable emulator."""

import argparse
import json
import re
import subprocess
import tempfile
import time
import uuid

PACKAGE = 'org.homeverb.nativeacceptance'
COMPONENT = PACKAGE + '/org.homeverb.AcceptanceRunner'


def interrupt_write(adb, run_id):
    """Stop only a live test process that proved its matching write reached the simulator."""
    def shell(*command):
        return subprocess.run(adb + ['shell', *command], capture_output=True, text=True, timeout=15)

    with tempfile.TemporaryFile(mode='w+') as output:
        process = subprocess.Popen(adb + ['shell', 'am', 'instrument', '-w', '-r',
                                         '-e', 'phase', 'crash_prepare', '-e', 'run_id', run_id,
                                         COMPONENT], stdout=output, stderr=output)
        stopped = False
        try:
            deadline = time.monotonic() + 15
            marker = None
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    output.seek(0)
                    completed = re.search(r'^INSTRUMENTATION_RESULT: completed_assertions=(\d+)$', output.read(), re.M)
                    count = completed.group(1) if completed else 'unavailable'
                    raise RuntimeError(f'Preparation ended before the forced interruption; completed assertions: {count}.')
                saved = shell('run-as', PACKAGE, 'cat', 'files/interrupted-control-ready.json')
                try:
                    value = json.loads(saved.stdout) if saved.returncode == 0 else None
                except json.JSONDecodeError:
                    value = None
                if (isinstance(value, dict) and set(value) == {'run_id', 'pid'}
                        and value['run_id'] == run_id and type(value['pid']) is int and value['pid'] > 1):
                    marker = value
                    break
                time.sleep(0.05)
            if marker is None:
                raise RuntimeError('No matching pending-write evidence arrived within the budget.')
            live = shell('pidof', PACKAGE)
            if process.poll() is not None or live.returncode or str(marker['pid']) not in live.stdout.split():
                raise RuntimeError('The prepared process is no longer live.')
            if shell('am', 'force-stop', PACKAGE).returncode:
                raise RuntimeError('The isolated test package could not be stopped.')
            process.wait(timeout=10)
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline:
                remaining = shell('pidof', PACKAGE)
                if remaining.returncode == 1 and not remaining.stdout.strip():
                    stopped = True
                    break
                if remaining.returncode not in (0, 1):
                    raise RuntimeError('The test process state could not be inspected.')
                time.sleep(0.05)
            if not stopped:
                raise RuntimeError('The test package still has a process after force-stop.')
        finally:
            try:
                if not stopped:
                    # This exact test package was started above; never stop another application.
                    shell('am', 'force-stop', PACKAGE)
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--adb', default='adb')
    parser.add_argument('--adb-port', required=True, type=int)
    parser.add_argument('--serial', required=True)
    parser.add_argument('--phase', required=True, choices=['unsecured', 'write', 'locked', 'read', 'controls', 'force-stop'])
    args = parser.parse_args()
    if not 1 <= args.adb_port <= 65535:
        parser.error('Require a valid port for the isolated ADB server.')
    adb = [args.adb, '-P', str(args.adb_port), '-s', args.serial]
    emulator = subprocess.run(adb + ['shell', 'getprop', 'ro.kernel.qemu'],
                              check=True, capture_output=True, text=True, timeout=15)
    if emulator.stdout.strip() != '1':
        parser.error('This harness refuses physical devices; select a disposable emulator.')
    extra = []
    if args.phase == 'force-stop':
        run_id = str(uuid.uuid4())
        try:
            interrupt_write(adb, run_id)
        except RuntimeError as error:
            # interrupt_write raises fixed diagnostics only, never Android output.
            parser.exit(1, f'FAIL: Android force-stop. {error}\n')
        except (OSError, subprocess.SubprocessError):
            parser.exit(1, 'FAIL: Android force-stop preparation or process termination was not verified.\n')
        extra = ['-e', 'run_id', run_id]
    result = subprocess.run(adb + ['shell', 'am', 'instrument', '-w', '-r', '-e', 'phase', args.phase,
                                   *extra, COMPONENT], capture_output=True, text=True, timeout=180)
    assertions = re.search(r'^INSTRUMENTATION_RESULT: assertions=(\d+)$', result.stdout, re.M)
    lines = set(result.stdout.splitlines())
    passed = ('INSTRUMENTATION_RESULT: homeverb_result=PASS' in lines
              and f'INSTRUMENTATION_RESULT: phase={args.phase}' in lines
              and 'INSTRUMENTATION_CODE: -1' in lines and assertions
              and int(assertions.group(1)) > 0)
    if result.returncode or not passed:
        # Never relay arbitrary Android logs or exception messages into test reports.
        completed = re.search(r'^INSTRUMENTATION_RESULT: completed_assertions=(\d+)$', result.stdout, re.M)
        parser.exit(1, f'FAIL: Android {args.phase}; completed assertions: '
                    f'{completed.group(1) if completed else "unavailable"}.\n')
    print(f'PASS: Android {args.phase}; {assertions.group(1)} assertions.')


if __name__ == '__main__':
    main()
