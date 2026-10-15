#!/usr/bin/env python3
"""Build and inspect an arm64 Android Core AAR from a Linux host."""

import argparse
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
TARGET = 'aarch64-linux-android'
SYMBOLS = ['homeverb_open', 'homeverb_call', 'homeverb_connect_account',
           'homeverb_cancel', 'homeverb_set_foreground', 'homeverb_close',
           'homeverb_string_free', 'Java_org_homeverb_NativeCredentials_nativeInstall']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ndk', required=True, type=Path)
    parser.add_argument('--android-jar', required=True, type=Path)
    parser.add_argument('--java-home', required=True, type=Path)
    args = parser.parse_args()
    tools = args.ndk / 'toolchains/llvm/prebuilt/linux-x86_64/bin'
    clang = tools / 'aarch64-linux-android28-clang'
    readelf = tools / 'llvm-readelf'
    for path in [clang, readelf, tools / 'llvm-ar', args.android_jar,
                 args.java_home / 'bin/javac', args.java_home / 'bin/jar']:
        if not path.is_file():
            parser.error(f'Missing build dependency: {path}')
    environment = {**os.environ,
                   'CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER': str(clang),
                   'CC_aarch64_linux_android': str(clang),
                   'AR_aarch64_linux_android': str(tools / 'llvm-ar'),
                   'RUSTFLAGS': os.environ.get('RUSTFLAGS', '') + ' -C link-arg=-Wl,-z,max-page-size=16384'}
    subprocess.run(['cargo', 'build', '-p', 'homeverb-ffi', '--target', TARGET,
                    '--target-dir', str(ROOT / 'target'), '--release', '--offline', '--locked'],
                   check=True, cwd=ROOT, env=environment)
    library = ROOT / 'target' / TARGET / 'release/libhomeverb_ffi.so'
    header = subprocess.check_output([str(readelf), '--file-header', str(library)], text=True)
    assert 'AArch64' in header, 'The output must be an arm64 library.'
    symbols = subprocess.check_output([str(readelf), '--dyn-symbols', '--wide', str(library)], text=True)
    for symbol in SYMBOLS:
        assert any(line.split()[-1:] == [symbol] and ' UND ' not in line for line in symbols.splitlines()), symbol
    segments = subprocess.check_output([str(readelf), '--program-headers', '--wide', str(library)], text=True)
    loads = [line for line in segments.splitlines() if line.strip().startswith('LOAD ')]
    assert loads and all(int(line.split()[-1], 16) >= 16384 for line in loads), 'Require 16 KiB ELF load alignment.'
    dynamic = subprocess.check_output([str(readelf), '--dynamic', str(library)], text=True)
    assert 'libc.so.6' not in dynamic and 'libjvm' not in dynamic, 'Desktop runtime dependencies must not enter Android.'
    output = ROOT / 'target/android/homeverb-core-arm64.aar'
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='homeverb-android-aar-') as directory:
        classes = Path(directory) / 'classes'
        classes.mkdir()
        sources = sorted((ROOT / 'packages/android/src/main/java').rglob('*.java'))
        subprocess.run([str(args.java_home / 'bin/javac'), '--release', '8',
                        '-Xlint:all,-options', '-Werror', '-cp', str(args.android_jar),
                        '-d', str(classes), *map(str, sources)], check=True, cwd=ROOT)
        jar = Path(directory) / 'classes.jar'
        subprocess.run([str(args.java_home / 'bin/jar'), '--create', '--file', str(jar),
                        '-C', str(classes), '.'], check=True)
        with zipfile.ZipFile(jar) as contents:
            assert 'org/homeverb/NativeCredentials.class' in contents.namelist()
            assert not any('Test' in name for name in contents.namelist())
        with zipfile.ZipFile(output, 'w', zipfile.ZIP_DEFLATED) as aar:
            aar.write(ROOT / 'packages/android/AndroidManifest.xml', 'AndroidManifest.xml')
            aar.write(ROOT / 'packages/android/consumer-rules.pro', 'proguard.txt')
            aar.write(jar, 'classes.jar')
            aar.write(library, 'jni/arm64-v8a/libhomeverb_ffi.so')
            aar.writestr('R.txt', '')
        with zipfile.ZipFile(output) as aar:
            assert aar.testzip() is None
            assert set(aar.namelist()) == {'AndroidManifest.xml', 'proguard.txt', 'classes.jar',
                                          'jni/arm64-v8a/libhomeverb_ffi.so', 'R.txt'}
    print(f'PASS: arm64 Android AAR, eight native exports, 16 KiB ELF alignment: {output}')
    print('This is a library build, not APK, emulator, Keystore or physical-device acceptance.')


if __name__ == '__main__':
    main()
