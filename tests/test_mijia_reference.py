"""Security and completeness checks for the offline Mijia reference projection."""
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('mijia_reference', ROOT / 'scripts/extract_mijia_reference_facts.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ReferenceTests(unittest.TestCase):
    def test_private_or_opaque_values_do_not_become_scalar_observations(self):
        service = {'iid': 2, 'type': 'urn:miot-spec-v2:service:gateway:00007875:fiction:1'}
        for name, fmt in [('dhcp-server-mac-adress', 'string'), ('user-id', 'uint32'),
                          ('token', 'uint32'), ('family-guard-config', 'string')]:
            prop = {'type': f'urn:miot-spec-v2:property:{name}:00000001:fiction:1',
                    'format': fmt, 'access': ['read']}
            self.assertIsNotNone(module.exclusion(service, prop))
        prop = {'type': 'urn:miot-spec-v2:property:on:00000006:fiction:1', 'format': 'bool', 'access': ['write']}
        self.assertEqual(module.exclusion(service, prop), 'not_readable')

    def test_every_source_definition_has_one_audit_entry_and_no_private_values(self):
        projected = module.project()
        profiles, ledger = projected[module.CATALOG], projected[module.LEDGER]
        self.assertEqual(len(profiles), 17)
        self.assertEqual(len(ledger['models']), 19)
        entries = [entry for model in ledger['models'] for entry in model['entries']]
        self.assertEqual(sum(e['kind'] == 'property' for e in entries), 470)
        self.assertEqual(sum(e['kind'] == 'action' for e in entries), 40)
        self.assertEqual(sum(e['kind'] == 'event' for e in entries), 78)
        self.assertEqual(sum(e.get('numeric_sample_history_status') == 'implemented_core_native_app_reported_samples'
                             for e in entries), 161)
        self.assertEqual(ledger['numeric_property_history']['core_native_app'],
                         'implemented_api_1_38_opt_in_and_version_negotiated_flutter')
        for model in ledger['models']:
            identities = [(e['kind'], e['siid'], e['iid']) for e in model['entries']]
            self.assertEqual(len(identities), len(set(identities)))
        import json
        for value in projected.values():
            encoded = json.dumps(value)
            for field in ['runtime/private/', 'serviceToken', 'ssecurity', 'received_at', 'read_observations']:
                self.assertNotIn(field, encoded)


if __name__ == '__main__':
    unittest.main()
