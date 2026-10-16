#!/usr/bin/env python3
"""Build, inspect and locally seal a development iOS XCFramework on macOS."""

import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
HEADER = ROOT / 'crates/ffi/include/homeverb.h'
MODULE = ROOT / 'packages/ios/module.modulemap'
INSTALL_NAME = '@rpath/HomeVerb.framework/HomeVerb'
DESTINATIONS = {
    'device': {
        'targets': {'arm64': 'aarch64-apple-ios'},
        'sdk': 'iphoneos', 'platform': 'IOS', 'bundle_platform': 'iPhoneOS', 'minimum': '13.0',
    },
    'simulator': {
        'targets': {'arm64': 'aarch64-apple-ios-sim', 'x86_64': 'x86_64-apple-ios'},
        'sdk': 'iphonesimulator', 'platform': 'IOSSIMULATOR',
        'bundle_platform': 'iPhoneSimulator', 'minimum': '14.0',
    },
}


def exports():
    names = re.findall(r'\b(homeverb_[a-z_]+)\s*\(', HEADER.read_text())
    if not names or len(names) != len(set(names)):
        raise ValueError('Require unique native declarations in the canonical header.')
    return names


def require_group(environment):
    group = environment.get('HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP', '')
    if (len(group) > 255 or '.' not in group or any(not part for part in group.split('.'))
            or not re.fullmatch(r'[A-Za-z0-9.-]+', group)):
        raise ValueError('Set HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP to the signed app private identifier.')
    return group


def output(*arguments):
    return subprocess.check_output(list(map(str, arguments)), text=True)


def run(*arguments, **kwargs):
    subprocess.run(list(map(str, arguments)), check=True, **kwargs)


def version(value):
    if not isinstance(value, str) or not re.fullmatch(r'\d+\.\d+(?:\.\d+)?', value):
        raise ValueError('Require an explicit numeric deployment version.')
    return tuple(map(int, value.split('.'))) + (0,) * (3 - len(value.split('.')))


def validate_reports(reports, destination, architecture):
    policy = DESTINATIONS[destination]
    if set(reports['architectures'].split()) != set(policy['targets']):
        raise ValueError('The framework has the wrong architecture set.')
    if 'DYLIB' not in reports['header'].split():
        raise ValueError('Require a dynamic Mach-O framework binary.')
    platforms = re.findall(r'^\s*platform\s+(\S+)\s*$', reports['build'], re.MULTILINE)
    minimums = re.findall(r'^\s*minos\s+(\S+)\s*$', reports['build'], re.MULTILINE)
    if platforms != [policy['platform']] or len(minimums) != 1 or version(minimums[0]) != version(policy['minimum']):
        raise ValueError('The Mach-O platform or deployment target does not match its slice.')
    identities = [line.strip() for line in reports['identity'].splitlines() if line.strip().startswith('@rpath/')]
    if identities != [INSTALL_NAME]:
        raise ValueError('Require the framework-relative install name.')
    dependencies = re.findall(r'^\s+(\S+) \(compatibility version ', reports['dependencies'], re.MULTILINE)
    if INSTALL_NAME not in dependencies or len(dependencies) < 2:
        raise ValueError('Missing framework identity or system dependencies.')
    for dependency in dependencies:
        if dependency != INSTALL_NAME and (not dependency.startswith(('/usr/lib/', '/System/Library/Frameworks/'))
                                           or '..' in Path(dependency).parts):
            raise ValueError('A non-system dependency must not enter the iOS framework.')
    defined = {parts[-1] for line in reports['symbols'].splitlines()
               if len(parts := line.split()) == 3 and parts[-2] == 'T'}
    for name in exports():
        if '_' + name not in defined:
            raise ValueError(f'Missing native export for {architecture}: {name}')
    if any('Java_' in symbol for symbol in defined):
        raise ValueError('Android JNI exports must not enter the iOS framework.')


def inspect_binary(binary, destination):
    architectures = output('xcrun', 'lipo', '-archs', binary)
    for architecture in DESTINATIONS[destination]['targets']:
        reports = {
            'architectures': architectures,
            'header': output('xcrun', 'otool', '-arch', architecture, '-hv', binary),
            'build': output('xcrun', 'vtool', '-arch', architecture, '-show-build', binary),
            'identity': output('xcrun', 'otool', '-arch', architecture, '-D', binary),
            'dependencies': output('xcrun', 'otool', '-arch', architecture, '-L', binary),
            'symbols': output('xcrun', 'nm', '-arch', architecture, '-gU', binary),
        }
        validate_reports(reports, destination, architecture)


def build_framework(directory, destination, package_version):
    policy = DESTINATIONS[destination]
    framework = directory / destination / 'HomeVerb.framework'
    (framework / 'Headers').mkdir(parents=True)
    (framework / 'Modules').mkdir()
    libraries = []
    environment = {
        **os.environ,
        'IPHONEOS_DEPLOYMENT_TARGET': policy['minimum'],
        'RUSTFLAGS': os.environ.get('RUSTFLAGS', '') + ' -C link-arg=-Wl,-headerpad_max_install_names',
    }
    for target in policy['targets'].values():
        run('cargo', 'build', '-p', 'homeverb-ffi', '--target', target,
            '--target-dir', ROOT / 'target', '--release', '--offline', '--locked',
            cwd=ROOT, env=environment)
        libraries.append(ROOT / 'target' / target / 'release/libhomeverb_ffi.dylib')
    binary = framework / 'HomeVerb'
    if len(libraries) == 1:
        shutil.copy2(libraries[0], binary)
    else:
        # Combine simulator architectures only; device and simulator stay separate.
        run('xcrun', 'lipo', '-create', *libraries, '-output', binary)
    binary.chmod(0o755)
    run('xcrun', 'install_name_tool', '-id', INSTALL_NAME, binary)
    shutil.copy2(HEADER, framework / 'Headers/homeverb.h')
    shutil.copy2(MODULE, framework / 'Modules/module.modulemap')
    metadata = {
        'CFBundleIdentifier': 'org.homeverb.native',
        'CFBundleName': 'HomeVerb',
        'CFBundleExecutable': 'HomeVerb',
        'CFBundlePackageType': 'FMWK',
        'CFBundleInfoDictionaryVersion': '6.0',
        'CFBundleShortVersionString': package_version,
        'CFBundleVersion': package_version,
        'CFBundleSupportedPlatforms': [policy['bundle_platform']],
        'MinimumOSVersion': policy['minimum'],
    }
    (framework / 'Info.plist').write_bytes(plistlib.dumps(metadata))
    inspect_binary(binary, destination)
    # Local ad-hoc integrity only: no identity, account, provisioning or timestamp server.
    run('codesign', '--force', '--sign', '-', '--timestamp=none', framework)
    run('codesign', '--verify', '--strict', framework)
    return framework


def inspect_layout(bundle, package_version):
    if bundle.is_symlink() or any(path.is_symlink() for path in bundle.rglob('*')):
        raise ValueError('The development XCFramework must contain regular files, not symlinks.')
    metadata = plistlib.loads((bundle / 'Info.plist').read_bytes())
    if metadata.get('XCFrameworkFormatVersion') != '1.0':
        raise ValueError('Unsupported XCFramework metadata.')
    libraries = metadata.get('AvailableLibraries')
    if not isinstance(libraries, list) or len(libraries) != 2:
        raise ValueError('Require one device and one simulator framework.')
    found = {}
    expected_files = {'Info.plist'}
    for library in libraries:
        if not isinstance(library, dict):
            raise ValueError('Invalid XCFramework library record.')
        identifier = library.get('LibraryIdentifier', '')
        variant = library.get('SupportedPlatformVariant')
        destination = 'device' if variant is None else variant
        if (not isinstance(identifier, str) or not re.fullmatch(r'[A-Za-z0-9_-]+', identifier)
                or destination not in DESTINATIONS or destination in found
                or library.get('SupportedPlatform') != 'ios'
                or library.get('LibraryPath') != 'HomeVerb.framework'):
            raise ValueError('Invalid or duplicate XCFramework destination.')
        policy = DESTINATIONS[destination]
        if sorted(library.get('SupportedArchitectures', [])) != sorted(policy['targets']):
            raise ValueError('Wrong architectures in XCFramework metadata.')
        framework = bundle / identifier / 'HomeVerb.framework'
        info = plistlib.loads((framework / 'Info.plist').read_bytes())
        expected = {
            'CFBundleIdentifier': 'org.homeverb.native', 'CFBundleExecutable': 'HomeVerb',
            'CFBundlePackageType': 'FMWK', 'CFBundleShortVersionString': package_version,
            'CFBundleVersion': package_version, 'MinimumOSVersion': policy['minimum'],
            'CFBundleSupportedPlatforms': [policy['bundle_platform']],
        }
        if any(info.get(key) != value for key, value in expected.items()):
            raise ValueError('The framework metadata does not match its binary destination.')
        if ((framework / 'Headers/homeverb.h').read_bytes() != HEADER.read_bytes()
                or (framework / 'Modules/module.modulemap').read_bytes() != MODULE.read_bytes()):
            raise ValueError('The packaged C header or module map differs from the canonical contract.')
        if not (framework / 'HomeVerb').is_file():
            raise ValueError('Missing framework binary.')
        expected_files.update(f'{identifier}/HomeVerb.framework/{name}' for name in [
            'Info.plist', 'HomeVerb', 'Headers/homeverb.h', 'Modules/module.modulemap',
            '_CodeSignature/CodeResources',
        ])
        found[destination] = framework
    actual_files = {path.relative_to(bundle).as_posix() for path in bundle.rglob('*') if path.is_file()}
    if actual_files != expected_files:
        raise ValueError('Unexpected or missing files in the development XCFramework.')
    return found


def inspect_package(bundle, package_version):
    frameworks = inspect_layout(bundle, package_version)
    for destination, framework in frameworks.items():
        inspect_binary(framework / 'HomeVerb', destination)
        run('codesign', '--verify', '--strict', framework)
    return frameworks


def check_consumers(directory, frameworks):
    source = directory / 'consumer.m'
    pointers = ',\n'.join(f'    (const void *)&{name}' for name in exports())
    source.write_text('@import HomeVerb;\n'
                      'static const void *const symbols[] __attribute__((used)) = {\n'
                      + pointers + '\n};\nint main(void) { return symbols[0] == 0; }\n')
    for destination, framework in frameworks.items():
        policy = DESTINATIONS[destination]
        sdk = output('xcrun', '--sdk', policy['sdk'], '--show-sdk-path').strip()
        for architecture in policy['targets']:
            target = f'{architecture}-apple-ios{policy["minimum"]}'
            if destination == 'simulator':
                target += '-simulator'
            binary = directory / f'consumer-{destination}-{architecture}'
            run('xcrun', '--sdk', policy['sdk'], 'clang', '-target', target, '-isysroot', sdk,
                '-Wall', '-Wextra', '-Werror', '-fmodules', '-F', framework.parent,
                source, '-framework', 'HomeVerb', '-o', binary)
            imports = output('xcrun', 'nm', '-u', binary)
            if not all('_' + name in imports.split() for name in exports()):
                raise ValueError('The consumer must link every declared C function.')


def archive(bundle, pending):
    with zipfile.ZipFile(pending, 'w', zipfile.ZIP_DEFLATED) as contents:
        for path in sorted(bundle.rglob('*')):
            if path.is_symlink():
                raise ValueError('Refuse symlinks in the development archive.')
            if path.is_file():
                contents.write(path, Path(bundle.name) / path.relative_to(bundle))
    with zipfile.ZipFile(pending) as contents:
        if contents.testzip() is not None:
            raise ValueError('The XCFramework archive failed its integrity check.')


def package(directory, frameworks, package_version, output_path):
    bundle = directory / 'HomeVerb.xcframework'
    command = ['xcodebuild', '-create-xcframework']
    for framework in frameworks:
        command.extend(['-framework', str(framework)])
    run(*command, '-output', bundle)
    inspect_package(bundle, package_version)
    pending = directory / output_path.name
    archive(bundle, pending)
    extracted = directory / 'extracted'
    run('ditto', '-x', '-k', pending, extracted)
    validated = inspect_package(extracted / bundle.name, package_version)
    check_consumers(directory, validated)
    # Preserve the previous deliverable until the actual extracted package passes.
    pending.replace(output_path)


def main():
    try:
        require_group(os.environ)
        if sys.platform != 'darwin':
            raise ValueError('The iOS framework builder requires macOS and Xcode.')
        for tool in ['cargo', 'xcrun', 'xcodebuild', 'codesign', 'ditto']:
            if shutil.which(tool) is None:
                raise ValueError(f'Missing build dependency: {tool}')
        package_version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
        # Confirm the public catalog is readable without changing native API versioning.
        native_version = json.loads((ROOT / 'packages/native-contract/api.json').read_text())['api_version']
        destination = ROOT / 'target/ios'
        destination.mkdir(parents=True, exist_ok=True)
        output_path = destination / 'homeverb-core.xcframework.zip'
        with tempfile.TemporaryDirectory(prefix='.homeverb-xcframework-', dir=destination) as temporary:
            directory = Path(temporary)
            frameworks = [build_framework(directory, name, package_version) for name in DESTINATIONS]
            package(directory, frameworks, package_version, output_path)
        print(f'PASS: iOS XCFramework, native API {native_version}, {len(exports())} C exports per architecture: {output_path}')
        print('Local ad-hoc integrity and consumer linkage only; no signed app, Keychain, Flutter or physical-device acceptance.')
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f'FAIL: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
