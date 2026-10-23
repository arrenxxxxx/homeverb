#!/usr/bin/env python3
"""Project reviewed, non-private washer metadata from the pinned reference ZIP."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / 'docs/adapter-references/2026-10-22/hisense-aijia-7.0.8.12-washers-readonly-20261022.zip'
ARCHIVE_SHA256 = '6a5e43de1dc844cbbc15d0066e267d4c5f9cdcca9ea5f3ad4e469cac24bd4bde'
OUTPUT = ROOT / 'crates/hisense/src/profiles.json'
PREFIX = 'hisense-aijia-7.0.8.12-washers-readonly/'
FUNCTION_PREFIX = 'connector.device.function.'
CAPABILITY = 'vendor.hisense.washer'
# Only these reviewed state meanings are projected. Actions/defaults are excluded.
FIELDS = {
    'Online': ('online', 'boolean'),
    'PowerControl': ('power.state', 'boolean'),
    'ChildLock': ('child_lock', 'boolean'),
    'RemoteControlFlag': ('remote_permission', 'string'),
    'RunStatus': ('run_status', 'string'),
    'RemainRunTime': ('remaining_minutes', 'integer'),
    'Modes': ('program', 'string'),
    'DoorStatus': ('door_open', 'boolean'),
    'DoorLockStatus': ('door_locked', 'boolean'),
    'StartPauseStatus': ('start_pause_status', 'string'),
}


def render() -> str:
    if hashlib.sha256(ARCHIVE.read_bytes()).hexdigest() != ARCHIVE_SHA256:
        raise ValueError('The Hisense reference archive does not match the reviewed checksum.')
    with zipfile.ZipFile(ARCHIVE) as archive:
        manifest = json.loads(archive.read(PREFIX + 'delivery_manifest.json'))
        model_bytes = archive.read(PREFIX + 'models.json')
        models = json.loads(model_bytes)
    # The original package remains the source of truth; never execute its scripts.
    entries = manifest['files']
    entry = next(item for item in entries if item['path'] == 'models.json')
    assert hashlib.sha256(model_bytes).hexdigest() == entry['sha256']
    profiles = []
    for model in models['models']:
        identity = model['identity']
        dual = identity['productCode'] == 'WV20W-1'
        assert identity['productCode'] in ('WV20W-1', 'WH120E5S-1')
        fields = []
        for function in model['functions']:
            name = function['code'].removeprefix(FUNCTION_PREFIX)
            base, _, suffix = name.partition('.')
            if base not in FIELDS or suffix not in ('', 'left', 'right', 'multiple'):
                continue
            prop, kind = FIELDS[base]
            component = suffix if suffix in ('left', 'right') else 'main'
            if suffix == 'multiple':
                prop = 'combined_run_status'
            assert len(function['states']) == 1
            state = function['states'][0]
            attr = next(a for a in function['attributes'] if a['code'] == state['attribute'])
            field = dict(wire=state['code'], component=component,
                         property=prop if prop == 'power.state' else f'{CAPABILITY}.{prop}',
                         kind=kind)
            if kind == 'integer':
                assert attr['suffix'] == 'min' and attr['value'] == '[0,1920]'
                field.update(minimum=0, maximum=1920, unit='min')
            else:
                field['values'] = {v['value']: v['code'] for v in attr['value']}
                if kind == 'boolean':
                    assert set(field['values']) == {'0', '1'}
            fields.append(field)
        programs = []
        for program in model['programs']:
            component = program['code'].rsplit('.', 1)[-1] if dual else 'main'
            assert component in (('left', 'right') if dual else ('main',))
            programs.append(dict(component=component, code=program['code'], value=program['value'],
                                 function_codes=program['functionCodeList']))
        profiles.append(dict(product_code=identity['productCode'], category_code=identity['categoryCode'],
                             partner_id=identity['partnerId'], components=['main', 'left', 'right'] if dual else ['main'],
                             fields=fields, programs=programs,
                             missing_program_function_codes=model['mode_parameter_detail']['program_function_codes_missing_from_general_map']))
    # One record per line keeps generated metadata reviewable without duplicating reports.
    lines = ['{', '  "source_models_sha256": ' + json.dumps(hashlib.sha256(model_bytes).hexdigest()) + ',', '  "profiles": [']
    for index, profile in enumerate(profiles):
        lines.append('    {')
        for key, value in profile.items():
            comma = ',' if key != 'missing_program_function_codes' else ''
            if key in ('fields', 'programs'):
                lines.append(f'      "{key}": [')
                lines.extend('        ' + json.dumps(item, ensure_ascii=True) + (',' if i + 1 < len(value) else '') for i, item in enumerate(value))
                lines.append('      ],')
            else:
                lines.append(f'      "{key}": ' + json.dumps(value, ensure_ascii=True) + comma)
        lines.append('    }' + (',' if index + 1 < len(profiles) else ''))
    return '\n'.join(lines + ['  ]', '}', ''])


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    text = render()
    if args.check:
        if OUTPUT.read_text() != text:
            raise SystemExit('Hisense profiles differ from the pinned reference projection.')
        print('PASS: Hisense profiles match the pinned reference projection.')
    else:
        OUTPUT.write_text(text)


if __name__ == '__main__':
    main()
