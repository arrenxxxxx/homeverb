"""The pinned plugin comparator must reject changed observation semantics."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('dishwasher_mapping', ROOT / 'scripts/verify_dishwasher_read_mapping.py')
mapping = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mapping)


class DishwasherReadMappingTests(unittest.TestCase):
    def setUp(self):
        with zipfile.ZipFile(mapping.ARCHIVE) as archive:
            self.source = json.loads(archive.read(mapping.MEMBER))
        self.capability = json.loads((ROOT / 'examples/capability.meiju-dishwasher-details.json').read_text())

    def test_pinned_reference(self):
        mapping.verify(self.source, self.capability)

    def test_changed_program_in_either_firmware_branch_is_rejected(self):
        for index in range(2):
            source = copy.deepcopy(self.source)
            source['firmware_settings'][index]['settings']['modeList'][0]['value'] = 'other'
            with self.assertRaises(AssertionError):
                mapping.verify(source, self.capability)

    def test_changed_door_conversion_is_rejected(self):
        source = copy.deepcopy(self.source)
        method = next(x for x in source['converter_methods'] if x['name'] == 'convertThingToLua')
        method['source'] = method['source'].replace('1 == e2[t3] ? o2[i2] = 1', '1 == e2[t3] ? o2[i2] = 0')
        with self.assertRaises(AssertionError):
            mapping.verify(source, self.capability)

    def test_unreviewed_action_is_rejected(self):
        self.capability['actions']['start_cycle'] = {}
        with self.assertRaises(AssertionError):
            mapping.verify(self.source, self.capability)


if __name__ == '__main__':
    unittest.main()
