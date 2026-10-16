"""Reject misleading ABI and test-plugin contents in local delivery APKs."""
import io
from pathlib import Path
import sys
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from verify_flutter_apk import inspect_contents


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
