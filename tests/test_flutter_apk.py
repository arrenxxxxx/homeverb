"""Reject misleading ABI and test-plugin contents in local delivery APKs."""
import io
from pathlib import Path
import sys
import unittest
from unittest.mock import patch
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from verify_flutter_apk import inspect_contents, match_core_library


class FlutterApkTests(unittest.TestCase):
    def archive(self, *, extra=None, missing=None, test_plugin=False):
        output = io.BytesIO()
        with zipfile.ZipFile(output, 'w') as archive:
            for abi in ['arm64-v8a', 'x86_64']:
                for library in ['libapp.so', 'libflutter.so', 'libhomeverb_ffi.so']:
                    name = f'lib/{abi}/{library}'
                    if name != missing:
                        archive.writestr(name, b'fictional ELF')
            if extra:
                archive.writestr(extra, b'fictional JNI')
            archive.writestr('classes.dex', b'IntegrationTestPlugin' if test_plugin else b'fictional dex')
        return zipfile.ZipFile(output)

    def test_two_complete_runtimes_are_required(self):
        with self.archive() as archive:
            inspect_contents(archive)
        for abi in ['arm64-v8a', 'x86_64']:
            with self.archive(missing=f'lib/{abi}/libhomeverb_ffi.so') as archive:
                with self.assertRaises(ValueError):
                    inspect_contents(archive)

    def test_transitive_32_bit_jni_must_not_advertise_an_incomplete_runtime(self):
        for abi in ['armeabi-v7a', 'x86']:
            with self.archive(extra=f'lib/{abi}/libdartjni.so') as archive:
                with self.assertRaises(ValueError):
                    inspect_contents(archive)

    def test_integration_plugin_must_not_enter_delivery_artifact(self):
        with self.archive(test_plugin=True) as archive:
            with self.assertRaises(ValueError):
                inspect_contents(archive)

    def test_exact_aar_match_does_not_require_stripping(self):
        with patch('verify_flutter_apk.subprocess.run') as run:
            self.assertEqual(match_core_library(b'current Core', b'current Core', 'strip'), 'exact AAR bytes')
            run.assert_not_called()

    def test_only_normal_symbol_stripping_can_explain_packaged_difference(self):
        def strip(arguments, *, check):
            self.assertTrue(check)
            self.assertEqual(arguments[:2], ['reviewed-ndk-strip', '--strip-unneeded'])
            path = Path(arguments[2])
            self.assertEqual(path.read_bytes(), b'current Core plus symbols')
            path.write_bytes(b'current Core')

        with patch('verify_flutter_apk.subprocess.run', side_effect=strip):
            self.assertIn('strip-unneeded', match_core_library(b'current Core', b'current Core plus symbols', 'reviewed-ndk-strip'))
            with self.assertRaisesRegex(ValueError, 'does not match'):
                match_core_library(b'old Core', b'current Core plus symbols', 'reviewed-ndk-strip')
