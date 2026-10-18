#!/usr/bin/env python3
"""Inspect a development Flutter APK before local delivery; never install or upload it."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile

from build_android_native import ABIS, ROOT, inspect_library, ndk_tools


def inspect_contents(archive):
    names = archive.namelist()
    libraries = [name for name in names if name.startswith('lib/') and name.endswith('.so')]
    if {name.split('/')[1] for name in libraries} != set(ABIS):
        raise ValueError('The APK must advertise exactly arm64-v8a and x86_64.')
    for abi in ABIS:
        for library in ['libapp.so', 'libflutter.so', 'libhomeverb_ffi.so']:
            if names.count(f'lib/{abi}/{library}') != 1:
                raise ValueError(f'Missing or duplicate {abi} runtime: {library}')
    for name in names:
        if name.startswith('classes') and name.endswith('.dex'):
            if b'IntegrationTestPlugin' in archive.read(name):
                raise ValueError('The integration-test plugin must not enter a delivery APK.')
    if archive.testzip() is not None:
        raise ValueError('Corrupt APK archive.')


def match_core_library(packaged, rebuilt, strip):
    """Require current AAR code, allowing only AGP's normal symbol stripping."""
    if packaged == rebuilt:
        return 'exact AAR bytes'
    with tempfile.TemporaryDirectory(prefix='homeverb-core-match-') as directory:
        library = Path(directory) / 'libhomeverb_ffi.so'
        library.write_bytes(rebuilt)
        subprocess.run([str(strip), '--strip-unneeded', str(library)], check=True)
        if packaged != library.read_bytes():
            raise ValueError('APK Core does not match the rebuilt AAR, including after symbol stripping.')
    return 'exact AAR bytes after NDK --strip-unneeded'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apk', required=True, type=Path)
    parser.add_argument('--sdk', required=True, type=Path)
    parser.add_argument('--java-home', required=True, type=Path)
    parser.add_argument('--adapter', required=True, choices=['homeverb.mock', 'midea.meiju'])
    parser.add_argument('--report', required=True, type=Path)
    args = parser.parse_args()
    build_tools = args.sdk / 'build-tools/36.0.0'
    tools = ndk_tools(args.sdk / 'ndk/28.2.13676358')
    readelf = tools / 'llvm-readelf'
    environment = {**os.environ, 'JAVA_HOME': str(args.java_home)}
    subprocess.run([str(build_tools / 'apksigner'), 'verify', '--verbose', str(args.apk)],
                   env=environment, check=True)
    subprocess.run([str(build_tools / 'zipalign'), '-c', '-P', '16', '4', str(args.apk)], check=True)
    badging = subprocess.check_output([str(build_tools / 'aapt'), 'dump', 'badging', str(args.apk)], text=True)
    if ("name='dev.homeverb.homeverb_mobile'" not in badging
            or "sdkVersion:'28'" not in badging or 'application-debuggable' in badging):
        raise ValueError('Require the HomeVerb API-28-minimum release-mode application.')
    manifest = subprocess.check_output([str(build_tools / 'aapt'), 'dump', 'xmltree',
                                        str(args.apk), 'AndroidManifest.xml'], text=True)
    if not any('android:allowBackup' in line and line.rstrip().endswith('0x0')
               for line in manifest.splitlines()):
        raise ValueError('Application backup must be disabled.')
    native = {}
    matches = {}
    aar_path = ROOT / 'target/android/homeverb-core-universal.aar'
    with zipfile.ZipFile(args.apk) as archive, zipfile.ZipFile(aar_path) as aar, tempfile.TemporaryDirectory() as directory:
        inspect_contents(archive)
        for abi in ABIS:
            data = archive.read(f'lib/{abi}/libhomeverb_ffi.so')
            matches[abi] = match_core_library(data, aar.read(f'jni/{abi}/libhomeverb_ffi.so'), tools / 'llvm-strip')
            library = Path(directory) / abi
            library.write_bytes(data)
            inspect_library(readelf, library, abi)
            native[abi] = hashlib.sha256(data).hexdigest()
    report = {
        'artifact': args.apk.name,
        'sha256': hashlib.sha256(args.apk.read_bytes()).hexdigest(),
        'bytes': args.apk.stat().st_size,
        'source_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'source_dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True).strip()),
        'declared_adapter': args.adapter,
        'native_sha256': native,
        'aar_sha256': hashlib.sha256(aar_path.read_bytes()).hexdigest(),
        'native_matches_current_aar': matches,
        'signature': 'verified; local development signing, not distribution acceptance',
        'zip_and_core_elf_alignment': '16 KiB verified',
        'core_exports_per_abi': 17,
        'runtime_acceptance': 'record separately; archive inspection is not an execution test',
    }
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + '\n')
    print(f'PASS: {args.apk.name}; two complete ABIs, current AAR match, native exports, signature and alignment.')
    print(f'SHA-256: {report["sha256"]}')


if __name__ == '__main__':
    main()
