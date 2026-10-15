#!/usr/bin/env python3
"""Build a test-only, network-disabled Android APK from a verified HomeVerb AAR."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import zipfile

from build_android_native import ABIS, ROOT, inspect_library


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--aar', required=True, type=Path)
    parser.add_argument('--abi', choices=ABIS, default='x86_64')
    parser.add_argument('--ndk', required=True, type=Path)
    parser.add_argument('--android-jar', required=True, type=Path)
    parser.add_argument('--build-tools', required=True, type=Path)
    parser.add_argument('--java-home', required=True, type=Path)
    args = parser.parse_args()
    tools = args.ndk / 'toolchains/llvm/prebuilt/linux-x86_64/bin'
    clang = tools / f'{ABIS[args.abi][0]}28-clang'
    java = args.java_home / 'bin'
    required = [args.aar, args.android_jar, clang, tools / 'llvm-readelf']
    required += [java / name for name in ['javac', 'jar', 'keytool']]
    required += [args.build_tools / name for name in ['aapt2', 'd8', 'zipalign', 'apksigner']]
    for path in required:
        if not path.is_file():
            parser.error(f'Missing build dependency: {path}')
    source = ROOT / 'packages/android/acceptance'
    output = ROOT / 'target/android' / f'homeverb-native-acceptance-{args.abi}.apk'
    output.parent.mkdir(parents=True, exist_ok=True)
    environment = {**os.environ, 'JAVA_HOME': str(args.java_home),
                   'PATH': str(java) + os.pathsep + os.environ.get('PATH', '')}

    def run(*command):
        subprocess.run(list(map(str, command)), check=True, cwd=ROOT, env=environment)

    with tempfile.TemporaryDirectory(prefix='.homeverb-acceptance-', dir=output.parent) as directory:
        work = Path(directory)
        production = work / 'production.jar'
        library = work / 'libhomeverb_ffi.so'
        with zipfile.ZipFile(args.aar) as aar:
            production.write_bytes(aar.read('classes.jar'))
            library.write_bytes(aar.read(f'jni/{args.abi}/libhomeverb_ffi.so'))
        inspect_library(tools / 'llvm-readelf', library, args.abi)
        with zipfile.ZipFile(production) as jar:
            if any('Test' in name or 'Acceptance' in name for name in jar.namelist()):
                raise ValueError('The input AAR must contain production classes only.')
        classes = work / 'classes'
        classes.mkdir()
        run(java / 'javac', '--release', '8', '-Xlint:all,-options', '-Werror',
            '-cp', os.pathsep.join(map(str, [args.android_jar, production])), '-d', classes,
            *sorted((source / 'java').rglob('*.java')))
        test_jar = work / 'acceptance.jar'
        run(java / 'jar', '--create', '--file', test_jar, '-C', classes, '.')
        dex = work / 'dex'
        dex.mkdir()
        run(args.build_tools / 'd8', '--min-api', '28', '--lib', args.android_jar,
            '--output', dex, production, test_jar)
        shim = work / 'libhomeverb_acceptance.so'
        run(clang, '-shared', '-fPIC', '-Wall', '-Wextra', '-Werror',
            '-I', ROOT / 'crates/ffi/include', source / 'probe.c', '-L', work,
            '-lhomeverb_ffi', '-Wl,-z,max-page-size=16384', '-o', shim)
        unsigned = work / 'unsigned.apk'
        run(args.build_tools / 'aapt2', 'link', '--manifest', source / 'AndroidManifest.xml',
            '-I', args.android_jar, '--version-code', '1', '--version-name', '0.1', '-o', unsigned)
        with zipfile.ZipFile(unsigned, 'a') as apk:
            apk.write(dex / 'classes.dex', 'classes.dex')
            apk.write(library, f'lib/{args.abi}/libhomeverb_ffi.so')
            apk.write(shim, f'lib/{args.abi}/libhomeverb_acceptance.so')
        aligned = work / 'aligned.apk'
        run(args.build_tools / 'zipalign', '-P', '16', '-f', '4', unsigned, aligned)
        # Ephemeral, publicly known development signing material; never a release key.
        key = work / 'debug.jks'
        run(java / 'keytool', '-genkeypair', '-keystore', key, '-storepass', 'android',
            '-keypass', 'android', '-alias', 'acceptance', '-keyalg', 'RSA', '-validity', '2',
            '-dname', 'CN=HomeVerb Native Acceptance', '-noprompt')
        signed = work / 'signed.apk'
        run(args.build_tools / 'apksigner', 'sign', '--ks', key, '--ks-pass', 'pass:android',
            '--out', signed, aligned)
        run(args.build_tools / 'apksigner', 'verify', signed)
        run(args.build_tools / 'zipalign', '-c', '-P', '16', '4', signed)
        permissions = subprocess.check_output(
            [str(args.build_tools / 'aapt2'), 'dump', 'permissions', str(signed)], text=True)
        if 'uses-permission' in permissions:
            raise ValueError('The acceptance APK must not request permissions.')
        with zipfile.ZipFile(signed) as apk:
            native = {name for name in apk.namelist() if name.startswith('lib/')}
            if apk.testzip() or native != {f'lib/{args.abi}/{name}' for name in
                                          ['libhomeverb_ffi.so', 'libhomeverb_acceptance.so']}:
                raise ValueError('Unexpected acceptance APK contents.')
        shutil.copyfile(signed, work / output.name)
        (work / output.name).replace(output)
    print(f'PASS: signed test-only {args.abi} APK, no permissions, 16 KiB ZIP alignment: {output}')
    print('Building this APK does not prove Android runtime, Keystore or phone acceptance.')


if __name__ == '__main__':
    main()
