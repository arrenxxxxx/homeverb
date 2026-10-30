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
        actions = projected[module.ACTION_CATALOG]
        self.assertEqual(len(actions), 11)
        self.assertEqual({(a['model'], a['siid'], a['aiid']) for a in actions}, {
            ('abhome.light.wy0a01', 2, 1), ('abhome.light.wy0a01', 2, 2),
            ('abhome.light.wy0a01', 2, 3), ('abhome.light.wy0a01', 2, 4),
            ('jdjz.fan.2072fc', 2, 1), ('xiaomi.hood.cyp10', 6, 1),
            ('cuco.plug.v3', 2, 1), ('yutai.plug.jdls1', 2, 1),
            ('opple.switch.0010d5', 3, 1), ('opple.switch.0010d6', 11, 1),
            ('opple.switch.0010d6', 12, 1),
        })
        for action in actions:
            self.assertEqual(action['in'], [])
            self.assertEqual(action['out'], [])
            for key, kind in [('spec_type', 'device'), ('service_type', 'service'), ('action_type', 'action')]:
                self.assertTrue(action[key].startswith(f'urn:miot-spec-v2:{kind}:'))
        self.assertEqual(ledger['no_argument_actions']['exact_actions'], 6)
        self.assertEqual(ledger['power_toggle_actions']['exact_actions'], 5)
        self.assertEqual(ledger['power_toggle_actions']['native_app'], 'native_api_1_41_implemented_flutter_pending')
        self.assertEqual(ledger['no_argument_actions']['core_native_app'],
                         'implemented_api_1_40_opt_in_and_version_negotiated_flutter')
        self.assertEqual(len(profiles), 17)
        self.assertEqual(len(ledger['models']), 19)
        self.assertEqual(sum(ledger['modern_controls']['models'].values()), 33)
        self.assertEqual(ledger['modern_controls']['models']['xiaomi.hood.cyp10'], 6)
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
