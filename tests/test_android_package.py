"""Android package admission tests with fictional ELF reports and classes."""

from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from build_android_native import ABIS, SYMBOLS, inspect_library, package_aar
from build_android_acceptance import validate_acceptance_permissions


class AndroidPackageTests(unittest.TestCase):
    def test_acceptance_permissions_remain_metadata_only(self):
        allowed = "uses-permission: name='android.permission.ACCESS_NETWORK_STATE'\n"
        validate_acceptance_permissions("package: fictional\n" + allowed)
        for denied in ['', allowed * 2, allowed + "uses-permission: name='android.permission.INTERNET'\n",
                       allowed.replace('ACCESS_NETWORK_STATE', 'ACCESS_FINE_LOCATION'),
                       allowed.replace('uses-permission:', 'uses-permission-sdk-23:')]:
            with self.subTest(denied=denied), self.assertRaises(ValueError):
                validate_acceptance_permissions(denied)

    def test_missing_platform_helper_preserves_previous_archive(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / 'core.aar'
            output.write_bytes(b'previous archive')
            jar = root / 'classes.jar'
            with zipfile.ZipFile(jar, 'w') as classes:
                classes.writestr('org/homeverb/NativeCredentials.class', b'fictional class')
            with self.assertRaises(ValueError):
                package_aar(output, {}, jar)
            self.assertEqual(output.read_bytes(), b'previous archive')

    def inspect(self, abi, changes=None):
        reports = {
            '--file-header': f'Machine: {ABIS[abi][1]}\n',
            '--dyn-symbols': '\n'.join(f'1: 0 8 FUNC GLOBAL DEFAULT 12 {name}' for name in SYMBOLS),
            '--program-headers': 'LOAD 0x0 0x0 0x0 0x100 0x100 R E 0x4000\n',
            '--dynamic': 'NEEDED Shared library: [libc.so]\n',
        }
        reports.update(changes or {})
        with patch('build_android_native.subprocess.check_output',
                   side_effect=lambda args, **kwargs: reports[args[1]]):
            inspect_library(Path('llvm-readelf'), Path('fictional.so'), abi)

    def test_both_android_architectures_pass(self):
        for abi in ABIS:
            with self.subTest(abi=abi):
                self.inspect(abi)

    def test_wrong_architecture_or_desktop_runtime_is_refused(self):
        for abi in ABIS:
            other = next(key for key in ABIS if key != abi)
            for changes in [
                {'--file-header': f'Machine: {ABIS[other][1]}\n'},
                {'--dynamic': 'NEEDED Shared library: [libc.so.6]\n'},
                {'--dynamic': 'NEEDED Shared library: [libjvm.so]\n'},
            ]:
                with self.subTest(abi=abi, changes=changes), self.assertRaises(ValueError):
                    self.inspect(abi, changes)

    def test_missing_or_undefined_export_is_refused(self):
        for missing in SYMBOLS:
            for section in ['', f'1: 0 0 FUNC GLOBAL DEFAULT UND {missing}']:
                symbols = '\n'.join(f'1: 0 8 FUNC GLOBAL DEFAULT 12 {name}'
                                    for name in SYMBOLS if name != missing)
                with self.subTest(missing=missing, section=section), self.assertRaises(ValueError):
                    self.inspect('x86_64', {'--dyn-symbols': symbols + '\n' + section})

    def test_empty_or_mixed_load_alignment_is_refused(self):
        for segments in ['', 'LOAD 0x0 0x0 0x0 0x100 0x100 R E 0x4000\n'
                             'LOAD 0x4000 0x4000 0x4000 0x100 0x100 RW 0x1000\n']:
            with self.subTest(segments=segments), self.assertRaises(ValueError):
                self.inspect('x86_64', {'--program-headers': segments})

    def test_archive_keeps_each_selected_library_in_its_abi_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            jar = root / 'classes.jar'
            with zipfile.ZipFile(jar, 'w') as classes:
                classes.writestr('org/homeverb/NativeCredentials.class', b'fictional class')
                classes.writestr('org/homeverb/NativeLanNetworks.class', b'fictional class')
            libraries = {}
            for abi in ABIS:
                library = root / f'{abi}.so'
                library.write_bytes(f'fictional {abi} ELF'.encode())
                libraries[abi] = library
            for selected in [{'arm64-v8a': libraries['arm64-v8a']},
                             {'x86_64': libraries['x86_64']}, libraries]:
                output = root / 'output/core.aar'
                package_aar(output, selected, jar)
                with zipfile.ZipFile(output) as aar:
                    expected = {'AndroidManifest.xml', 'proguard.txt', 'classes.jar', 'R.txt'}
                    expected.update(f'jni/{abi}/libhomeverb_ffi.so' for abi in selected)
                    self.assertEqual(set(aar.namelist()), expected)
                    for abi, library in selected.items():
                        self.assertEqual(aar.read(f'jni/{abi}/libhomeverb_ffi.so'), library.read_bytes())

    def test_test_classes_and_failed_packaging_preserve_previous_archive(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / 'core.aar'
            output.write_bytes(b'previous archive')
            jar = root / 'classes.jar'
            for extra in [True, False]:
                with zipfile.ZipFile(jar, 'w') as classes:
                    classes.writestr('org/homeverb/NativeCredentials.class', b'fictional class')
                    classes.writestr('org/homeverb/NativeLanNetworks.class', b'fictional class')
                    if extra:
                        classes.writestr('org/homeverb/TestCredentials.class', b'fictional test')
                with self.assertRaises(ValueError if extra else FileNotFoundError):
                    package_aar(output, {'x86_64': root / 'missing.so'}, jar)
                self.assertEqual(output.read_bytes(), b'previous archive')
                self.assertFalse(list(root.glob('.homeverb-aar-*')))


if __name__ == '__main__':
    unittest.main()
