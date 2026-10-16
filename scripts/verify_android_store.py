#!/usr/bin/env python3
"""Compile Android platform helpers and exercise synthetic network selection and JVM/JNI storage."""

import argparse
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--android-jar', type=Path, required=True)
    parser.add_argument('--java-home', type=Path, required=True)
    args = parser.parse_args()
    if not args.android_jar.is_file():
        parser.error('Provide official Android API 28 or newer stubs.')
    java = args.java_home / 'bin/java'
    javac = args.java_home / 'bin/javac'
    if not java.is_file() or not javac.is_file():
        parser.error('Provide a JDK with java and javac.')
    with tempfile.TemporaryDirectory(prefix='homeverb-android-classes-') as directory:
        sources = sorted((ROOT / 'packages/android/src').rglob('*.java'))
        subprocess.run([str(javac), '--release', '8', '-Xlint:all,-options', '-Werror',
                        '-cp', str(args.android_jar), '-d', directory,
                        *map(str, sources)], check=True, cwd=ROOT)
        subprocess.run([str(java), '-ea', '-cp', directory,
                        'org.homeverb.EncryptedEntriesTest'], check=True, cwd=ROOT)
        subprocess.run([str(java), '-ea', '-cp', os.pathsep.join([directory, str(args.android_jar)]),
                        'org.homeverb.NativeLanNetworksTest'], check=True, cwd=ROOT)
        environment = {**os.environ, 'JAVA_HOME': str(args.java_home),
                       'HOMEVERB_ANDROID_TEST_CLASSES': directory}
        subprocess.run(['cargo', 'test', '-p', 'homeverb-secrets', '--offline', '--locked',
                        'android::tests::jvm_bridge', '--', '--ignored', '--exact'],
                       check=True, cwd=ROOT, env=environment)
    print('PASS: Android Java compilation, synthetic network selection and JVM/JNI storage; no Android Keystore or phone acceptance.')


if __name__ == '__main__':
    main()
