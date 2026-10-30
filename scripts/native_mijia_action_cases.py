"""Fictional API 1.41 startup counterexamples shared by schema and C checks."""


def invalid_action_startups(startup):
    entries = startup['mijia_action_grants']
    for index, entry in enumerate(entries):
        changes = [
            ('device_id', '*'),
            ('device_id', 'ABCDEFAB-0000-4000-8000-000000000001'),
            ('device_id', '40000000-0000-5000-8000-000000000001'),
            ('device_id', '40000000-0000-4000-7000-000000000001'),
            ('component_id', 'light-3'), ('capability', 'power'),
            ('action', 'set'), ('action', 'reset'),
            ('arguments', {}), ('value', True), ('siid', 2), ('aiid', 1),
            ('confirmed', True), ('model', 'abhome.light.wy0a01'),
        ]
        for field, value in changes:
            yield f'action {index} invalid {field}/{value}', {
                **startup, 'mijia_action_grants': [{**entry, field: value}]}
        for field in entry:
            yield f'action {index} missing {field}', {
                **startup, 'mijia_action_grants': [
                    {key: value for key, value in entry.items() if key != field}]}
        for other in entries:
            candidate = {**entry, 'component_id': other['component_id'],
                         'capability': other['capability']}
            if not any(all(candidate[key] == valid[key] for key in
                           ('component_id', 'capability', 'action')) for valid in entries):
                yield f'action {index} foreign tuple', {
                    **startup, 'mijia_action_grants': [candidate]}
    for adapter in ['homeverb.mock', 'midea.meiju']:
        yield f'actions on {adapter}', {**startup, 'adapter': adapter}
    omitted = dict(startup)
    del omitted['adapter']
    yield 'actions on default adapter', omitted
    for grants in [None, {}, True, [None], [entries[0], entries[0]],
                   [{**entries[0], 'device_id': f'40000000-0000-4000-8000-{i:012x}'}
                    for i in range(65)]]:
        yield 'invalid action grant list', {**startup, 'mijia_action_grants': grants}
    for component in ['switch-1', 'switch-4', 'switch-10', 'switch-13']:
        yield f'unreviewed elevated action {component}', {
            **startup, 'mijia_action_grants': [{**entries[0],
                'component_id': component, 'capability': 'vendor.xiaomi.mijia.switch',
                'action': 'toggle'}]}
