#!/usr/bin/env python3
"""Project reviewed facts from the owner-supplied October 22 Meiju archive.

No bundled program, JavaScript or Lua is executed. The archive and member digest
are checked before reading. Only public product selectors and reviewed scalar
protocol facts enter the compiled catalog; sessions and response values do not.
"""

import argparse
import copy
import hashlib
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / 'docs/adapter-references/2026-10-22/meiju-11.7.10.6-analysis-20261022.zip'
SHA256 = 'fa4d5c978b7a7b0197f64c7cbd8badff49f57ecf83034e1f6ae93c038831e453'
MEMBER = 'meiju-11.7.10.6-analysis/analysis/six-device-contract.json'
OUTPUT = ROOT / 'crates/midea/src/ac_reference_features.json'
COVERAGE = ROOT / 'docs/development/meiju-interface-coverage.json'


def project():
    if hashlib.sha256(ARCHIVE.read_bytes()).hexdigest() != SHA256:
        raise ValueError('The reviewed Meiju archive has changed.')
    with zipfile.ZipFile(ARCHIVE) as archive:
        contract = json.loads(archive.read(MEMBER))
        generic = json.loads(archive.read(MEMBER.replace('six-device-contract.json', 'integration_contract.json')))
    wall = next(p for p in json.loads((ROOT / 'crates/midea/src/ac_features.json').read_text())
                if p['selector'] == 'wall')
    builders = {item['operation']: item['payload_constructor_source']
                for item in contract['air_conditioner']['feature_bound_control_builders']}
    for operation, fragments in {
        'switchCoolProvinceSw': ['i2 = e4 ? 1 : 0', 'a2.cool_power_saving = i2'],
        'switchStrongCool': ['o2 = e4 ? 1 : 0', 'i2.strong_cool = o2'],
        'switchLightSensitiveSw': ['n2 = e4 ? 3 : 0', 'light_sensitive: n2'],
    }.items():
        if any(fragment not in builders[operation] for fragment in fragments):
            raise ValueError('The reviewed scalar constructor evidence has changed.')
    # The other optional functions require additional state/encoding review.
    compatible = {'vertical_swing', 'horizontal_swing', 'vertical_direction',
                  'horizontal_direction', 'breezeless', 'display',
                  'prevent_overcooling', 'dry_after_shutdown'}
    profiles = []
    for device in contract['devices']:
        if device['type'] != '0xAC':
            continue
        matches = device['static_config_matches']
        if len(matches) != 1:
            raise ValueError('Ambiguous AC product configuration.')
        keys = {key for function in matches[0]['functions']
                for key in function['luaKey'].split(',') if key}
        features = [copy.deepcopy(f) for f in wall['features']
                    if f['name'] in compatible and f['wire'] in keys]
        # A support flag is not permission to reuse another profile's paired
        # ECO payload or auxiliary-heat force/default behavior.
        if 'prevent_straight_wind' in keys and 'prevent_straight_wind_lr' not in keys:
            features.append(copy.deepcopy(next(f for f in wall['features']
                                               if f['name'] == 'avoid_direct_airflow')))
        for name, wire, enabled in [
            ('cool_energy_saving', 'cool_power_saving', 1),
            ('strong_cooling', 'strong_cool', 1),
            ('light_sensing', 'light_sensitive', 3),
        ]:
            if wire in keys:
                features.append({'name': name, 'wire': wire, 'read_field': wire,
                                 'parameter': {'type': 'boolean'},
                                 'options': [{'value': value, 'control': {wire: enabled if value else 0}}
                                             for value in [False, True]]})
        # Exact profiles need only reads for retained optional fields. The
        # adapter independently requires a valid report before every setter.
        used = {f['wire'] for f in features} | {f['read_field'] for f in features}
        profiles.append({'selector': 'official-' + device['sn8'],
                         'codes': [device['sn8']], 'models': [device['productModel']],
                         'queries': [q for q in wall['queries'] if q in used],
                         'preserve': ['buzzer'], 'features': features})
    if len(profiles) != 4:
        raise ValueError('Expected four reviewed AC profiles.')
    coverage = {
        'reference_sha256': SHA256,
        'status': 'partial_implementation',
        'network_used': False,
        'operations': [],
    }
    for operation in generic['operations']:
        identifier = operation['id']
        status = 'blocked_modern_authentication_and_contract'
        if identifier in {'login_id', 'home_devices'}:
            status = 'existing_community_path_only'
        elif identifier in {'tsl_custom_action', 'plugin_cloud_bridge', 'transparent', 'transparent_oem'}:
            status = 'generic_execution_not_admitted'
        elif identifier in {'sse_access', 'sse_confirm'}:
            status = 'foreground_subscription_contract_missing'
        coverage['operations'].append({
            'id': identifier, 'method': operation['http_method'],
            'route': operation['route'], 'semantics': operation['semantics'],
            'implementation': status,
        })
    return {OUTPUT: profiles, COVERAGE: coverage}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    for path, value in project().items():
        expected = json.dumps(value, indent=2, ensure_ascii=True) + '\n'
        if args.check:
            if path.read_text() != expected:
                raise SystemExit('The Meiju reference projection is out of date.')
        else:
            path.write_text(expected)
    print('Four exact AC profiles and the full interface coverage ledger checked; no network or source execution.')


if __name__ == '__main__':
    main()
