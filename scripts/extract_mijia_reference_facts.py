#!/usr/bin/env python3
"""Project public exact-model facts and exhaustive coverage without networking."""
import argparse
import hashlib
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / 'docs/adapter-references/2026-10-22/xiaomi-home-11.7.709-73335d3c17df-readonly.zip'
DIGEST = 'cb0c7f5855be9b647ca9124999b35d612de58be7a1cd6657fbefba88d0fa00c4'
CATALOG = ROOT / 'crates/mijia/src/reference_profiles.json'
LEDGER = ROOT / 'docs/development/mijia-interface-coverage.json'
SCALARS = {'bool', 'uint8', 'uint16', 'uint32', 'int8', 'int16', 'int32', 'float'}


def exclusion(service, prop):
    if 'read' not in prop.get('access', []):
        return 'not_readable'
    if service['iid'] == 1 or service['type'].split(':')[3] in {
        'device-information', 'extra-device-information', 'remote-control-management', 'p2p-stream'
    }:
        return 'identity_pairing_or_media'
    name = prop['type'].split(':')[3]
    if any(term in name for term in ('password', 'passwd', 'token', 'secret', 'stream',
                                    'url', 'ssid', 'serial', 'mac', 'ip-address', 'user-id', 'key-id')):
        return 'private_identity_or_credentials'
    if prop.get('format') not in SCALARS:
        return 'opaque_value_requires_separate_contract'
    return None


def admitted_write(model, siid, piid):
    if model == 'jdjz.fan.2072fc':
        return siid == 2 and piid in {1, 3, 4, 5, 6, 7, 8, 9}
    if model == 'abhome.light.wy0a01':
        return siid == 2 and piid in {1, 2, 3, 7, 9, 12, 13}
    if model == 'zhimi.airpurifier.v6':
        return (siid == 2 and piid in {1, 2, 3}) or (siid in {5, 6, 7, 8} and piid == 1)
    return False


def project():
    if hashlib.sha256(ARCHIVE.read_bytes()).hexdigest() != DIGEST:
        raise ValueError('The reviewed Mijia archive changed.')
    with zipfile.ZipFile(ARCHIVE) as archive:
        models = json.loads(archive.read('models.json'))['models']
        contract = json.loads(archive.read('contract.json'))
    profiles, ledger = [], []
    for model in models:
        services, entries = [], []
        for service in model['services']:
            properties = []
            for prop in service.get('properties', []):
                reason = exclusion(service, prop)
                entries.append({'kind': 'property', 'siid': service['iid'], 'iid': prop['iid'],
                                'name': prop['type'].split(':')[3],
                                'read_status': reason or 'implemented_exact_scalar_mapping',
                                'write_status': ('implemented_exact_granted_legacy_property_set'
                                                 if admitted_write(model['model'], service['iid'], prop['iid'])
                                                 and 'write' in prop.get('access', [])
                                                 else 'not_admitted_by_reference'),
                                'reference_successful_instances': len(prop.get('analysis_verification', {}).get('successful_instances', []))})
                if reason:
                    continue
                p = {k: prop[k] for k in ('iid', 'type', 'format', 'access', 'unit', 'value-range') if k in prop}
                if 'value-list' in prop:
                    p['value-list'] = [{'value': v['value']} for v in prop['value-list']]
                properties.append(p)
            for kind, key in [('action', 'actions'), ('event', 'events')]:
                for item in service.get(key, []):
                    entries.append({'kind': kind, 'siid': service['iid'], 'iid': item['iid'],
                                    'name': item['type'].split(':')[3],
                                    'status': 'blocked_runtime_contract_and_separate_admission'})
            if properties:
                services.append({'iid': service['iid'], 'type': service['type'], 'properties': properties})
        if model['spec_urn']:
            profiles.append({'model': model['model'], 'spec_type': model['spec_urn'],
                             'source': {'type': model['spec_urn'], 'services': services}})
        ledger.append({'model': model['model'], 'spec_type': model['spec_urn'],
                       'status': ('exact_scalar_reads_and_granted_fan_settings'
                                  if model['model'] == 'jdjz.fan.2072fc'
                                  else 'exact_scalar_reads_and_granted_reference_settings'
                                  if model['model'] in {'abhome.light.wy0a01', 'zhimi.airpurifier.v6'}
                                  else 'exact_scalar_reads_only' if services
                                  else 'placeholder_no_reviewed_readable_spec'),
                       'reference_read_summary': model['independent_read_summary'], 'entries': entries})
    operations = [{'id': o['id'], 'route': o['route'],
                   'status': 'legacy_session_path_only' if o['id'] in ('directory', 'homes', 'properties') else 'not_admitted_by_reference',
                   'reason': 'Modern SID/session origin differs; no credential crossover or arbitrary RPC.'}
                  for o in contract['operations']]
    coverage = {'reference_sha256': DIGEST, 'status': 'partial_implementation',
                'physical_acceptance': 'pending_owner', 'operations': operations, 'models': ledger,
                'remaining': ['modern_native_login_and_renewal', 'cloud_cache_inventory_reconciliation',
                              'specialized_failed_property_read_paths', 'BLE_and_LAN', 'history_and_events',
                              'health_and_media', 'additional_write_runtime_contracts']}
    return {CATALOG: profiles, LEDGER: coverage}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    for path, value in project().items():
        expected = json.dumps(value, indent=2, ensure_ascii=True) + '\n'
        if args.check:
            if path.read_text() != expected:
                raise SystemExit(f'Outdated reference projection: {path.name}')
        else:
            path.write_text(expected)
    print('Exact Mijia reference facts and exhaustive coverage checked; no network or source execution.')


if __name__ == '__main__':
    main()
