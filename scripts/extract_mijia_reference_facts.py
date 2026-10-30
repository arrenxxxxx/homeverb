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
ACTION_CATALOG = ROOT / 'crates/mijia/src/action_profiles.json'
EVENT_CATALOG = ROOT / 'crates/mijia/src/event_profiles.json'
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
        return (siid == 2 and piid in {1, 3, 4, 5, 6, 7, 8, 9}) or (siid == 3 and piid == 5)
    if model == 'xiaomi.hood.cyp10':
        return (siid == 6 and piid in {1, 2}) or (siid in {2, 3} and piid == 1) or (siid == 13 and piid in {1, 2})
    if model == 'abhome.light.wy0a01':
        return siid == 2 and piid in {1, 2, 3, 7, 9, 12, 13}
    if model == 'cuco.plug.v3':
        return (siid == 2 and piid in {1, 2}) or (siid in {7, 13} and piid == 1)
    if model == 'yutai.plug.jdls1':
        return (siid == 2 and piid in {1, 2}) or (siid, piid) in {(4, 1), (7, 7)}
    if model == 'opple.switch.0010d5':
        return (siid, piid) == (3, 1)
    if model == 'opple.switch.0010d6':
        return siid in {11, 12} and piid == 1
    if model == 'zhimi.airpurifier.v6':
        return (siid == 2 and piid in {1, 2, 3}) or (siid in {5, 6, 7, 8} and piid == 1)
    return False


def admitted_power_action(model, siid, aiid):
    return aiid == 1 and ((model in {'cuco.plug.v3', 'yutai.plug.jdls1'} and siid == 2)
                         or (model == 'opple.switch.0010d5' and siid == 3)
                         or (model == 'opple.switch.0010d6' and siid in {11, 12}))


def admitted_maintenance_action(model, siid, aiid):
    return ((model == 'xiaomi.camera.083ac1' and (siid, aiid) in {(2, 1), (2, 2), (4, 2)})
            or (model == 'xiaomi.hood.cyp10' and (siid, aiid) == (10, 1)))


def admitted_action(model, siid, aiid):
    return ((model == 'abhome.light.wy0a01' and siid == 2 and aiid in {1, 2, 3, 4})
            or (model == 'jdjz.fan.2072fc' and siid == 2 and aiid == 1)
            or (model == 'xiaomi.hood.cyp10' and siid == 6 and aiid == 1)
            or admitted_power_action(model, siid, aiid)
            or admitted_maintenance_action(model, siid, aiid))


def project():
    if hashlib.sha256(ARCHIVE.read_bytes()).hexdigest() != DIGEST:
        raise ValueError('The reviewed Mijia archive changed.')
    with zipfile.ZipFile(ARCHIVE) as archive:
        models = json.loads(archive.read('models.json'))['models']
        contract = json.loads(archive.read('contract.json'))
    profiles, ledger, action_profiles, event_profiles = [], [], [], []
    for model in models:
        services, entries = [], []
        for service in model['services']:
            properties = []
            for prop in service.get('properties', []):
                reason = exclusion(service, prop)
                entries.append({'kind': 'property', 'siid': service['iid'], 'iid': prop['iid'],
                                'name': prop['type'].split(':')[3],
                                'read_status': reason or 'implemented_exact_scalar_mapping',
                                'write_status': ('implemented_native_outlet_control_legacy_and_modern'
                                                 if model['model'] == 'cuco.plug.v3' and admitted_write(model['model'], service['iid'], prop['iid'])
                                                 else 'implemented_exact_granted_legacy_and_modern_property_set'
                                                 if admitted_write(model['model'], service['iid'], prop['iid'])
                                                 and 'write' in prop.get('access', [])
                                                 else 'not_admitted_by_reference'),
                                'numeric_sample_history_status': ('implemented_core_native_app_reported_samples'
                                                                  if not reason and prop.get('format') in SCALARS - {'bool'}
                                                                  else 'not_admitted'),
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
                                    'status': ('implemented_core_native_app_trusted_maintenance_action'
                                               if kind == 'action' and admitted_maintenance_action(model['model'], service['iid'], item['iid'])
                                               else 'implemented_core_native_app_trusted_power_toggle'
                                               if kind == 'action' and admitted_power_action(model['model'], service['iid'], item['iid'])
                                               else 'implemented_core_native_app_action'
                                               if kind == 'action' and admitted_action(model['model'], service['iid'], item['iid'])
                                               else 'typed_core_occurrence_history_native_app_pending'
                                               if kind == 'event'
                                               else 'blocked_runtime_contract_and_separate_admission')})
                    if kind == 'event':
                        event_profiles.append({'model': model['model'], 'spec_type': model['spec_urn'],
                                               'service_type': service['type'], 'siid': service['iid'],
                                               'eiid': item['iid'], 'event_type': item['type'],
                                               'arguments': item['arguments']})
                    if kind == 'action' and admitted_action(model['model'], service['iid'], item['iid']):
                        if item.get('in') != [] or item.get('out') != []:
                            raise ValueError('The reviewed action must have no inputs or outputs.')
                        action_profiles.append({'model': model['model'], 'spec_type': model['spec_urn'],
                                                'service_type': service['type'], 'siid': service['iid'],
                                                'aiid': item['iid'], 'action_type': item['type'],
                                                'in': [], 'out': []})
            if properties:
                services.append({'iid': service['iid'], 'type': service['type'], 'properties': properties})
        if model['spec_urn']:
            profiles.append({'model': model['model'], 'spec_type': model['spec_urn'],
                             'source': {'type': model['spec_urn'], 'services': services}})
        ledger.append({'model': model['model'], 'spec_type': model['spec_urn'],
                       'status': ('exact_scalar_reads_and_native_outlet_controls'
                                  if model['model'] == 'cuco.plug.v3'
                                  else 'exact_scalar_reads_and_granted_fan_and_quiet_settings'
                                  if model['model'] == 'jdjz.fan.2072fc'
                                  else 'exact_scalar_reads_and_granted_reference_settings'
                                  if model['model'] in {'abhome.light.wy0a01', 'zhimi.airpurifier.v6', 'xiaomi.hood.cyp10'}
                                  else 'exact_scalar_reads_only' if services
                                  else 'placeholder_no_reviewed_readable_spec'),
                       'reference_read_summary': model['independent_read_summary'], 'entries': entries})
    operations = [{'id': o['id'], 'route': o['route'],
                   'status': ('legacy_host_and_modern_typed_read' if o['id'] in ('directory', 'homes', 'properties')
                              else 'implemented_exact_granted_legacy_and_modern_property_set'
                              if o['id'] == 'property_write_candidate'
                              else 'implemented_fifteen_core_native_app_actions' if o['id'] == 'action_candidate'
                              else 'not_admitted_by_reference'),
                   'reason': ('Modern reference-origin typed reads and protected accounts are implemented; native and Flutter device publication and Core reads are implemented; provider acceptance remains pending.'
                              if o['id'] in ('directory', 'homes', 'properties')
                              else 'ADR-0104/0106/0114 admit 40 reviewed scalar mappings across eight exact models with grants and trusted elevated approval; all 40 have App integration; no arbitrary setter or physical acceptance.'
                              if o['id'] == 'property_write_candidate'
                              else 'ADR-0107/0108/0109 supply six exact actions through Core/native/App. ADR-0110/0111 add five elevated power toggles through Core/native/App with independent grants and trusted approval; ADR-0112/0113 add four Core/native/App maintenance/stop actions with independent trusted confirmation; all real acceptance remains pending.'
                              if o['id'] == 'action_candidate'
                              else 'No credential crossover or arbitrary RPC; separate exact control admission is required.')}
                  for o in contract['operations']]
    coverage = {'reference_sha256': DIGEST, 'status': 'partial_implementation',
                'physical_acceptance': 'pending_owner', 'operations': operations, 'models': ledger,
                'modern_authentication': {
                    'status': 'implemented_typed_rust_qr_and_same_subject_renewal',
                    'evidence': 'ADR-0093; fictional protocol and local TLS tests; anonymous URL shapes only',
                    'protected_account_persistence': 'implemented_payload_v2_with_independent_legacy_and_modern_slots',
                    'account_evidence': 'ADR-0095; fictional SQLite, secret-store and protocol fixtures',
                    'host_lifecycle': 'implemented_typed_session_qr_and_explicit_account_operations',
                    'native_c_bridge': 'implemented_api_1_37_opt_in_device_publication_with_1_36_logical_compatibility',
                    'app_authentication': 'implemented_foreground_qr_and_explicit_account_operations',
                    'app_evidence': 'ADR-0098; fictional Flutter and C-ABI tests; independent local QR decoding',
                    'provider_and_phone_acceptance': 'pending',
                    'legacy_session_conversion': 'unavailable',
                    'modern_api_origin_and_codec': 'implemented_reference_continuous_rc4_reads'},
                'modern_reads': {
                    'status': 'implemented_typed_reference_directory_homes_and_pinned_properties',
                    'origin': 'https://core.api.mijia.tech/app',
                    'codec': 'continuous_rc4_drop1024_with_bounded_encrypted_gzip',
                    'logical_parent_reconciliation': 'implemented_groups_protected_bindings_and_native_flutter_publication',
                    'group_evidence': 'ADR-0099; fictional bounded parent-graph tests',
                    'protected_account': 'implemented_typed_rust',
                    'typed_host_directory_and_recovery': 'implemented',
                    'native_c_logical_directory_summary': 'implemented_api_1_36',
                    'protected_device_bindings': 'implemented_payload_v3_explicit_read_protocol_and_logical_evidence',
                    'typed_host_device_publication': 'implemented_explicit_qr_continuation_refresh_and_recovery',
                    'native_c_device_publication': 'implemented_api_1_37_explicit_opt_in',
                    'ordinary_account_refresh': 'implemented_protected_protocol_selection_without_fallback',
                    'physical_core_and_app': 'implemented_native_flutter_device_publication_and_core_reads',
                    'app_evidence': 'ADR-0102; fictional ABI negotiation, QR-to-card clicks and Core-client state requests',
                    'android_package': 'archive_checked_api_1_37_no_runtime_acceptance',
                    'provider_acceptance': 'pending'},
                'modern_controls': {
                    'status': 'implemented_exact_granted_native_property_set',
                    'evidence': 'ADR-0104/0106/0114; fictional protocol, TLS, Core, native Host and Flutter tests',
                    'origin': 'https://core.api.mijia.tech/app',
                    'route': '/miotspec/prop/set',
                    'codec': 'continuous_rc4_drop1024_with_bounded_encrypted_gzip',
                    'models': {'jdjz.fan.2072fc': 9, 'abhome.light.wy0a01': 7,
                               'zhimi.airpurifier.v6': 7, 'cuco.plug.v3': 4,
                               'xiaomi.hood.cyp10': 6, 'yutai.plug.jdls1': 4,
                               'opple.switch.0010d5': 1, 'opple.switch.0010d6': 2},
                    'authority': 'exact_startup_grants_and_independent_protected_family_admission',
                    'elevated_outlet': 'immutable_trusted_approval_required',
                    'native_and_app': 'api_1_43_exact_switch_outlet_extension_with_trusted_flutter',
                    'uncertainty': 'preserve_unknown_without_retry_or_protocol_fallback',
                    'android_package': 'archive_checked_85fefd8_api_1_43_no_runtime_acceptance',
                    'provider_acceptance': 'pending'},
                'numeric_property_history': {
                    'status': 'implemented_core_native_app_reported_samples',
                    'evidence': 'ADR-0105; pinned MIT property-log keys and Apache value channel',
                    'route': '/user/get_user_device_data',
                    'legacy_origin': 'https://api.io.mi.com/app',
                    'modern_origin': 'https://api.mijia.tech/app',
                    'codec': 'per_field_rc4_drop1024',
                    'request_type': 'prop',
                    'numeric_fields': 161,
                    'values': 'original_signed_numeric_domain_without_daily_scaling',
                    'completeness': 'unproven',
                    'core_native_app': 'implemented_api_1_38_opt_in_and_version_negotiated_flutter',
                    'android_package': 'archive_checked_2e776a9_no_runtime_acceptance',
                    'provider_acceptance': 'pending'},
                'modern_history': {
                    'status': 'implemented_exact_daily_energy_through_core_and_native_history',
                    'evidence': 'ADR-0103; pinned MIT statistics transport and Apache exact model semantics',
                    'origin': 'https://api.mijia.tech/app',
                    'route': '/v2/user/statistics',
                    'codec': 'per_field_rc4_drop1024',
                    'model': 'cuco.plug.v3',
                    'spec_type': 'urn:miot-spec-v2:device:outlet:0000A002:cuco-v3:2',
                    'unit': 'kWh', 'completeness': 'unproven',
                    'provider_acceptance': 'pending'},
                'remaining': ['modern_provider_and_phone_acceptance',
                              'modern_api_origin_and_codec_acceptance', 'cloud_cache_inventory_reconciliation',
                              'specialized_failed_property_read_paths', 'BLE_and_LAN', 'history_and_events',
                              'health_and_media', 'additional_write_runtime_contracts']}
    coverage['reported_event_history'] = {
        'status': 'implemented_typed_core_occurrence_times_only',
        'evidence': 'ADR-0115; pinned public SDK MIoT event keys and community history envelope',
        'event_definitions': len(event_profiles), 'route': '/user/get_user_device_data',
        'legacy_origin': 'https://api.io.mi.com/app',
        'modern_origin': 'https://api.mijia.tech/app',
        'codec': 'per_field_rc4_drop1024', 'request_type': 'event',
        'key': 'exact_siid_dot_eiid', 'maximum_records': 31, 'maximum_window_days': 32,
        'returned_data': 'reported_timestamps_only_without_payload_or_actor',
        'completeness': 'unproven', 'core_native_app': 'typed_core_implemented_native_app_pending',
        'protected_admission': 'exact_account_model_root_membership_and_selected_protocol',
        'descriptor_publication': 'explicit_typed_opt_in_and_refresh_with_event_only_components',
        'payload_semantics_pagination_and_subscriptions': 'pending',
        'provider_acceptance': 'pending',
    }
    coverage['no_argument_actions'] = {
        'status': 'implemented_core_native_app_and_protocol', 'evidence': 'ADR-0107/0108/0109',
        'exact_actions': 6, 'route': '/miotspec/action',
        'legacy_profile': 'SID_xiaomiio_api_io_mi_com_per_field',
        'modern_profile': 'SID_mijia_core_api_mijia_tech_continuous',
        'core_native_app': 'implemented_api_1_40_opt_in_and_version_negotiated_flutter',
        'android_package': 'archive_checked_ab34f21_no_runtime_acceptance',
        'provider_acceptance': 'pending',
    }
    coverage['power_toggle_actions'] = {
        'status': 'implemented_protocol_core_native_app_with_trusted_approval',
        'evidence': 'ADR-0110/0111; pinned exact empty-input switch actions',
        'exact_actions': 5, 'models': 4, 'route': '/miotspec/action',
        'risk': 'elevated', 'idempotency': 'non_idempotent',
        'native_app': 'implemented_api_1_41_opt_in_and_trusted_flutter_confirmation',
        'android_package': 'archive_checked_d505ccb_no_runtime_acceptance',
        'provider_acceptance': 'pending',
    }
    writable = [(model['model'], service['iid'], prop['iid'])
                for model in models for service in model['services']
                for prop in service.get('properties', []) if 'write' in prop.get('access', [])]
    pending_writes = [item for item in writable if not admitted_write(*item)]
    coverage['property_control_coverage'] = {
        'declared_writable_definitions': len(writable),
        'implemented_exact_definitions': len(writable) - len(pending_writes),
        'pending_separate_admission': len(pending_writes),
        'meaning': 'Source write access alone does not establish command semantics, risk, authorization, runtime support or physical acceptance.',
    }
    coverage['maintenance_actions'] = {
        'status': 'implemented_protocol_core_native_app_with_trusted_approval',
        'evidence': 'ADR-0112/0113; pinned exact no-input/no-output maintenance actions',
        'exact_actions': 4, 'models': 2, 'route': '/miotspec/action',
        'risk': 'elevated', 'idempotency': 'non_idempotent',
        'native_app': 'implemented_api_1_42_and_flutter',
        'android_package': 'archive_checked_87f3410_no_runtime_acceptance',
        'provider_acceptance': 'pending',
    }
    return {CATALOG: profiles, LEDGER: coverage, ACTION_CATALOG: action_profiles, EVENT_CATALOG: event_profiles}


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
