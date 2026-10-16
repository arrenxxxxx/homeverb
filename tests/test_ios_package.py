"""iOS package refusal tests with fictional Mach-O reports and bundle files."""

from pathlib import Path
import plistlib
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import build_ios_native as ios


def reports(destination):
    policy = ios.DESTINATIONS[destination]
    return {
        'architectures': ' '.join(policy['targets']),
        'header': 'MH_MAGIC_64 ARM64 ALL 0 DYLIB 20 1000 NOUNDEFS DYLDLINK TWOLEVEL',
        'build': f'cmd LC_BUILD_VERSION\nplatform {policy["platform"]}\nminos {policy["minimum"]}\nsdk 26.0',
        'identity': f'fictional:\n{ios.INSTALL_NAME}\n',
        'dependencies': f'fictional:\n\t{ios.INSTALL_NAME} (compatibility version 0.0.0, current version 0.0.0)\n'
                        '\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1.0.0)\n',
        'symbols': '\n'.join(f'0000000000000100 T _{name}' for name in ios.exports()),
    }


def bundle_fixture(bundle):
    bundle.mkdir(parents=True)
    libraries = []
    for name, policy in ios.DESTINATIONS.items():
        framework = bundle / name / 'HomeVerb.framework'
        (framework / 'Headers').mkdir(parents=True)
        (framework / 'Modules').mkdir()
        (framework / '_CodeSignature').mkdir()
        (framework / 'HomeVerb').write_bytes(b'fictional Mach-O')
        (framework / 'Headers/homeverb.h').write_bytes(ios.HEADER.read_bytes())
        (framework / 'Modules/module.modulemap').write_bytes(ios.MODULE.read_bytes())
        (framework / '_CodeSignature/CodeResources').write_bytes(b'fictional local seal')
        (framework / 'Info.plist').write_bytes(plistlib.dumps({
            'CFBundleIdentifier': 'org.homeverb.native', 'CFBundleExecutable': 'HomeVerb',
            'CFBundlePackageType': 'FMWK', 'CFBundleShortVersionString': '0.2.0',
            'CFBundleVersion': '0.2.0', 'MinimumOSVersion': policy['minimum'],
            'CFBundleSupportedPlatforms': [policy['bundle_platform']],
        }))
        library = {'LibraryIdentifier': name, 'LibraryPath': 'HomeVerb.framework',
                   'SupportedPlatform': 'ios', 'SupportedArchitectures': list(policy['targets'])}
        if name == 'simulator':
            library['SupportedPlatformVariant'] = name
        libraries.append(library)
    (bundle / 'Info.plist').write_bytes(plistlib.dumps({
        'XCFrameworkFormatVersion': '1.0', 'AvailableLibraries': libraries,
    }))
    return bundle


class IosPackageTests(unittest.TestCase):
    def test_private_group_is_required_without_a_wildcard_or_fallback(self):
        for group in ['', 'group', '.bad', 'bad.', 'a..b', 'a.*', 'a. b', 'a.\0b', 'a.' + 'b' * 254]:
            with self.subTest(group=group), self.assertRaises(ValueError):
                ios.require_group({'HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP': group})
        self.assertEqual(ios.require_group({'HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP': 'SYNTHETIC.org.homeverb.test'}),
                         'SYNTHETIC.org.homeverb.test')

    def test_device_and_simulator_reports_pass_for_each_architecture(self):
        for name, policy in ios.DESTINATIONS.items():
            for architecture in policy['targets']:
                ios.validate_reports(reports(name), name, architecture)

    def test_wrong_platform_architecture_kind_or_deployment_is_refused(self):
        for name in ios.DESTINATIONS:
            for key, value in [
                ('architectures', 'arm64 x86_64 armv7'),
                ('header', 'MH_MAGIC_64 ARM64 ALL 0 EXECUTE'),
                ('build', 'cmd LC_BUILD_VERSION\nplatform MACOS\nminos 13.0'),
                ('build', 'cmd LC_BUILD_VERSION\nplatform IOS\nminos 15.0'),
                ('build', 'cmd LC_BUILD_VERSION\nplatform IOSSIMULATOR\nminos 15.0'),
                ('build', 'cmd LC_BUILD_VERSION\nplatform IOS'),
            ]:
                with self.subTest(name=name, key=key), self.assertRaises(ValueError):
                    ios.validate_reports({**reports(name), key: value}, name, 'arm64')

    def test_foreign_install_names_desktop_dependencies_and_undefined_exports_are_refused(self):
        valid = reports('device')
        for key, value in [
            ('identity', 'fictional:\n@rpath/libhomeverb_ffi.dylib\n'),
            ('dependencies', valid['dependencies'] + '\t/usr/local/lib/foreign.dylib (compatibility version 0.0.0)\n'),
            ('dependencies', valid['dependencies'] + '\t/usr/lib/../local/foreign.dylib (compatibility version 0.0.0)\n'),
            ('dependencies', valid['dependencies'] + '\t/System/Library/PrivateFrameworks/Private.framework/Private (compatibility version 0.0.0)\n'),
            ('dependencies', ''),
            ('symbols', valid['symbols'] + '\n00000100 T _Java_org_homeverb_NativeCredentials_nativeInstall'),
        ]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                ios.validate_reports({**valid, key: value}, 'device', 'arm64')
        for name in ios.exports():
            for replacement in ['', f'                 U _{name}']:
                symbols = valid['symbols'].replace(f'0000000000000100 T _{name}', replacement)
                with self.subTest(name=name), self.assertRaises(ValueError):
                    ios.validate_reports({**valid, 'symbols': symbols}, 'device', 'arm64')

    def test_exact_framework_layout_passes_and_preserves_the_canonical_contract(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            bundle = bundle_fixture(root / 'HomeVerb.xcframework')
            found = ios.inspect_layout(bundle, '0.2.0')
            self.assertEqual(set(found), {'device', 'simulator'})
            archive = root / 'core.zip'
            ios.archive(bundle, archive)
            with zipfile.ZipFile(archive) as contents:
                self.assertIsNone(contents.testzip())
                for name in found:
                    path = f'HomeVerb.xcframework/{name}/HomeVerb.framework/Headers/homeverb.h'
                    self.assertEqual(contents.read(path), ios.HEADER.read_bytes())

    def test_duplicate_wrong_or_traversing_destinations_are_refused(self):
        mutations = [
            lambda data: data.update(XCFrameworkFormatVersion='2.0'),
            lambda data: data['AvailableLibraries'].pop(),
            lambda data: data['AvailableLibraries'].append(data['AvailableLibraries'][0]),
            lambda data: data['AvailableLibraries'][1].pop('SupportedPlatformVariant'),
            lambda data: data['AvailableLibraries'][0].update(LibraryIdentifier='../outside'),
            lambda data: data['AvailableLibraries'][0].update(LibraryPath='../outside'),
            lambda data: data['AvailableLibraries'][0].update(SupportedPlatform='macos'),
            lambda data: data['AvailableLibraries'][0].update(SupportedArchitectures=['x86_64']),
        ]
        for mutate in mutations:
            with tempfile.TemporaryDirectory() as temporary:
                bundle = bundle_fixture(Path(temporary) / 'HomeVerb.xcframework')
                path = bundle / 'Info.plist'
                metadata = plistlib.loads(path.read_bytes())
                mutate(metadata)
                path.write_bytes(plistlib.dumps(metadata))
                with self.assertRaises(ValueError):
                    ios.inspect_layout(bundle, '0.2.0')

    def test_wrong_headers_metadata_extra_test_files_and_symlinks_are_refused(self):
        for change in ['header', 'module', 'metadata', 'extra', 'symlink', 'missing']:
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                bundle = bundle_fixture(Path(temporary) / 'HomeVerb.xcframework')
                framework = bundle / 'device/HomeVerb.framework'
                if change == 'header':
                    (framework / 'Headers/homeverb.h').write_text('fictional stale header\n')
                elif change == 'module':
                    (framework / 'Modules/module.modulemap').write_text('module Other {}\n')
                elif change == 'metadata':
                    path = framework / 'Info.plist'
                    info = plistlib.loads(path.read_bytes())
                    info['MinimumOSVersion'] = '99.0'
                    path.write_bytes(plistlib.dumps(info))
                elif change == 'extra':
                    (framework / 'TestCredentials.txt').write_text('fictional test fixture\n')
                elif change == 'symlink':
                    (framework / 'alias').symlink_to('HomeVerb')
                else:
                    (framework / 'HomeVerb').unlink()
                with self.assertRaises(ValueError):
                    ios.inspect_layout(bundle, '0.2.0')

    def test_failed_extracted_package_or_consumer_check_preserves_previous_archive(self):
        for fail in ['inspection', 'consumer', None]:
            with self.subTest(fail=fail), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                staging = root / 'staging'
                staging.mkdir()
                output = root / 'core.zip'
                output.write_bytes(b'previous deliverable')

                def run(*args, **kwargs):
                    if args[0] == 'xcodebuild':
                        bundle_fixture(Path(args[-1]))
                    elif args[0] == 'ditto':
                        with zipfile.ZipFile(args[-2]) as contents:
                            contents.extractall(args[-1])
                    else:
                        self.fail('Unexpected command in the synthetic package test.')

                def inspect(path, package_version):
                    if fail == 'inspection' and path.parent.name == 'extracted':
                        raise ValueError('Synthetic extracted-binary rejection.')
                    return ios.inspect_layout(path, package_version)

                with patch.object(ios, 'run', side_effect=run), patch.object(ios, 'inspect_package', side_effect=inspect), \
                        patch.object(ios, 'check_consumers', side_effect=ValueError('Synthetic link failure.') if fail == 'consumer' else None):
                    if fail:
                        with self.assertRaises(ValueError):
                            ios.package(staging, [root / 'device.framework', root / 'simulator.framework'], '0.2.0', output)
                        self.assertEqual(output.read_bytes(), b'previous deliverable')
                    else:
                        ios.package(staging, [root / 'device.framework', root / 'simulator.framework'], '0.2.0', output)
                        with zipfile.ZipFile(output) as contents:
                            self.assertIsNone(contents.testzip())


if __name__ == '__main__':
    unittest.main()
