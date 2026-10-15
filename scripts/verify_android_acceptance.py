#!/usr/bin/env python3
"""Run one native acceptance phase on an explicitly selected disposable emulator."""

import argparse
import re
import subprocess

COMPONENT = 'org.homeverb.nativeacceptance/org.homeverb.AcceptanceRunner'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--adb', default='adb')
    parser.add_argument('--adb-port', required=True, type=int)
    parser.add_argument('--serial', required=True)
    parser.add_argument('--phase', required=True, choices=['unsecured', 'write', 'locked', 'read'])
    args = parser.parse_args()
    if not 1 <= args.adb_port <= 65535:
        parser.error('Require a valid port for the isolated ADB server.')
    adb = [args.adb, '-P', str(args.adb_port), '-s', args.serial]
    emulator = subprocess.run(adb + ['shell', 'getprop', 'ro.kernel.qemu'],
                              check=True, capture_output=True, text=True, timeout=15)
    if emulator.stdout.strip() != '1':
        parser.error('This harness refuses physical devices; select a disposable emulator.')
    result = subprocess.run(adb + ['shell', 'am', 'instrument', '-w', '-r', '-e', 'phase', args.phase,
                                   COMPONENT], capture_output=True, text=True, timeout=180)
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
