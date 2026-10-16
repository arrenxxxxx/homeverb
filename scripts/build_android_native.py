#!/usr/bin/env python3
"""Build and inspect an arm64 and/or x86_64 Android Core AAR from a Linux host."""

import argparse
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
ABIS = {
    'arm64-v8a': ('aarch64-linux-android', 'AArch64', 'arm64'),
    'x86_64': ('x86_64-linux-android', 'Advanced Micro Devices X86-64', 'x86_64'),
}
SYMBOLS = ['homeverb_open', 'homeverb_call', 'homeverb_connect_account',
           'homeverb_cancel', 'homeverb_set_foreground', 'homeverb_close',
           'homeverb_string_free', 'homeverb_prepare_approval', 'homeverb_execute_approved',
           'homeverb_discard_approval', 'homeverb_prepare_lan_scan', 'homeverb_confirm_lan_scan',
           'homeverb_discard_lan_scan', 'Java_org_homeverb_NativeCredentials_nativeInstall']


def inspect_library(readelf, library, abi):
    header = subprocess.check_output([str(readelf), '--file-header', str(library)], text=True)
    if ABIS[abi][1] not in header:
        raise ValueError(f'The output must be a {abi} library.')
    symbols = subprocess.check_output([str(readelf), '--dyn-symbols', '--wide', str(library)], text=True)
    for symbol in SYMBOLS:
        if not any(line.split()[-1:] == [symbol] and ' UND ' not in line for line in symbols.splitlines()):
            raise ValueError(f'Missing native export: {symbol}')
    segments = subprocess.check_output([str(readelf), '--program-headers', '--wide', str(library)], text=True)
    loads = [line for line in segments.splitlines() if line.strip().startswith('LOAD ')]
    if not loads or any(int(line.split()[-1], 16) < 16384 for line in loads):
        raise ValueError('Require 16 KiB ELF load alignment.')
    dynamic = subprocess.check_output([str(readelf), '--dynamic', str(library)], text=True)
    if 'libc.so.6' in dynamic or 'libjvm' in dynamic:
        raise ValueError('Desktop runtime dependencies must not enter Android.')


def build_library(tools, abi):
    target = ABIS[abi][0]
    clang = tools / f'{target}28-clang'
    key = target.replace('-', '_')
    environment = {**os.environ,
                   f'CARGO_TARGET_{key.upper()}_LINKER': str(clang),
                   f'CC_{key}': str(clang),
                   f'AR_{key}': str(tools / 'llvm-ar'),
                   'RUSTFLAGS': os.environ.get('RUSTFLAGS', '') + ' -C link-arg=-Wl,-z,max-page-size=16384'}
    subprocess.run(['cargo', 'build', '-p', 'homeverb-ffi', '--target', target,
                    '--target-dir', str(ROOT / 'target'), '--release', '--offline', '--locked'],
                   check=True, cwd=ROOT, env=environment)
    library = ROOT / 'target' / target / 'release/libhomeverb_ffi.so'
    inspect_library(tools / 'llvm-readelf', library, abi)
    return library


def package_aar(output, libraries, jar):
    with zipfile.ZipFile(jar) as contents:
        if ('org/homeverb/NativeCredentials.class' not in contents.namelist()
                or any('Test' in name for name in contents.namelist())):
            raise ValueError('Require production-only credential classes.')
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.homeverb-aar-', dir=output.parent) as directory:
        pending = Path(directory) / output.name
        native_paths = {f'jni/{abi}/libhomeverb_ffi.so' for abi in libraries}
        with zipfile.ZipFile(pending, 'w', zipfile.ZIP_DEFLATED) as aar:
            aar.write(ROOT / 'packages/android/AndroidManifest.xml', 'AndroidManifest.xml')
            aar.write(ROOT / 'packages/android/consumer-rules.pro', 'proguard.txt')
            aar.write(jar, 'classes.jar')
            for abi, library in libraries.items():
                aar.write(library, f'jni/{abi}/libhomeverb_ffi.so')
            aar.writestr('R.txt', '')
        with zipfile.ZipFile(pending) as aar:
            expected = {'AndroidManifest.xml', 'proguard.txt', 'classes.jar', 'R.txt'} | native_paths
            if aar.testzip() is not None or set(aar.namelist()) != expected:
                raise ValueError('Invalid Android archive contents.')
        pending.replace(output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ndk', required=True, type=Path)
    parser.add_argument('--android-jar', required=True, type=Path)
    parser.add_argument('--java-home', required=True, type=Path)
    parser.add_argument('--abi', choices=ABIS, action='append',
                        help='Repeat for a combined AAR; default: arm64-v8a.')
    args = parser.parse_args()
    abis = args.abi or ['arm64-v8a']
    if len(set(abis)) != len(abis):
        parser.error('Select each ABI only once.')
    tools = args.ndk / 'toolchains/llvm/prebuilt/linux-x86_64/bin'
    dependencies = [tools / f'{ABIS[abi][0]}28-clang' for abi in abis]
    dependencies += [tools / 'llvm-readelf', tools / 'llvm-ar', args.android_jar,
                     args.java_home / 'bin/javac', args.java_home / 'bin/jar']
    for path in dependencies:
        if not path.is_file():
            parser.error(f'Missing build dependency: {path}')
    libraries = {abi: build_library(tools, abi) for abi in abis}
    suffix = ABIS[abis[0]][2] if len(abis) == 1 else 'universal'
    output = ROOT / 'target/android' / f'homeverb-core-{suffix}.aar'
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
        package_aar(output, libraries, jar)
    print(f'PASS: Android AAR ({", ".join(abis)}), {len(SYMBOLS)} exports per ABI, 16 KiB ELF alignment: {output}')
    print('This is a library build, not APK, emulator, Keystore or physical-device acceptance.')


if __name__ == '__main__':
    main()
