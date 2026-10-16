"""Build the offline atlas, coverage, screen inventory and asset manifest."""
from pathlib import Path
import hashlib
import json
import re
import shutil
from PIL import Image

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
data = json.loads((ROOT/'design.json').read_text())
screens = [s for b in data['boards'] for s in b['screens']]
assert len(screens) == 82
assert len(data['interactions']) == 201
assert data['skin']['available'] == [{'id':'window_light','name':'Window Light'}]

template = (ROOT/'atlas-template.html').read_text()
assert '__DESIGN_DATA__' in template
(ROOT/'index.html').write_text(template.replace('__DESIGN_DATA__',json.dumps(data,separators=(',',':')).replace('</','<\\/')))

coverage='# UI coverage map\n\nScope: the owner-approved mobile requirements and room/card/platform/sharing refinements. These locally rendered Window Light panels are visual design evidence, not implementation or hardware evidence.\n\n| Requirement | Rendered screens |\n|---|---|\n'
for item in json.loads((ROOT/'requirements.json').read_text()):
    coverage+='| '+item['requirement']+' | '+' · '.join(f'[{sid}](screens/{sid}.png)' for sid in item['screens'])+' |\n'
coverage+='\n'
coverage+='''## Window Light acceptance additions

| Requirement | Evidence / remaining acceptance |
|---|---|
| Apply one visual system to every state | All 82 panels use the shared locally rendered background, surface and accent tokens. |
| Exactly one skin option | S02 and S11 use stable ID `window_light`; no alternate/dark/system option. |
| Preserve future skin extensibility | [Skin contract](../../../docs/ui-window-light.md) and [Flutter app](../../../apps/mobile/README.md); local preference persistence is implemented, while additional skins remain out of scope. |
| Keep development coverage explicit | [Functional handoff](../../../docs/ui-functional-spec.md), [82-state inventory](../../../docs/ui-screen-inventory.md) and 15 acceptance groups. |
| Honest rendering evidence | Local Python composition; no Flutter build, browser UI test or physical-device verification is implied. |

## Complete screen inventory

| Board | Screens |
|---|---|
'''
for b in data['boards']:
    coverage+='| ['+b['title']+']('+b['image']+') | '+' · '.join('['+s['id']+': '+s['title']+']('+s['image']+')' for s in b['screens'])+' |\n'
(ROOT/'coverage.md').write_text(coverage)

def text_parts(value):
    if isinstance(value,str):return [value]
    if isinstance(value,list):return [v for child in value for v in text_parts(child)]
    return []

inventory='''# Window Light screen inventory

Status: visual specification, October 15, 2026. All 82 states below are rendered in the [current atlas](../output/prototypes/homeverb-window-light-v1/index.html). Actions navigate review images; the atlas does not run appliance commands or authenticate accounts. See [functional requirements and acceptance](ui-functional-spec.md) and [Flutter/skin implementation](ui-window-light.md).

Each entry preserves its stable screen ID, visible content and explicit branch targets. Static explanatory copy is included to make safety and recovery requirements reviewable. System dialogs, keyboard variants and generic loading/save-error variants reuse these components; they remain required acceptance cases even when no separate image ID exists. Unlisted Back/Cancel behavior returns to the actual caller, as defined by the functional handoff.

The legacy S11 dark-appearance study is superseded by the one-option Window Light preview. There is no alternate skin in this release.

'''
# Retain contract amendments that do not have separate rendered states.
runtime_notes = {
    "P08": """Required runtime variants: P04 retains one Connecting and retrieving devices state until the combined call returns; no staged success or percentage is available. Cancel invalidates the pending UI result and requires local account summaries to be reloaded before retrying; it never automatically retries. P08 distinguishes complete-empty, partial and failed retrieval; partial does not supply a usable partial device list, so reload saved cards. AUTH_REQUIRED offers explicit reauthorization; pending credentials offer recovery or same-account sign-in. Do not send an identity conflict to a fictional merge control: native API 1.14.1 exposes no identity-linking method. Separate authoritative screenshots for these variants remain required; the base image alone does not establish their visual acceptance. Retrieve Devices is an explicit bounded refresh using the saved session. Reauthorization stays scoped to the selected account, removal requires exact local-scope confirmation, and View Devices uses locally reloaded cards.""",
    "P10": """API 1.7 amendment: distinguish expired confirmed authorization from interrupted
credential persistence. A pending account offers Recover Saved Authorization using
`recover_account`, with finite progress/cancel and explicit failure/retry states.
Inspect both the returned credential state and retrieval result; stored credentials
with failed publication are not completed card loading. Recovery preserves known
organization and sends no controls or new login. API 1.8 additionally offers explicit Sign In to the Original Account through the
protected connection function when saved identity/bindings are intact, including
expired pending sessions. Missing/corrupt material stays pending; explain the
limitation without an automatic reconnect/removal loop.
This is a functional amendment, not newly rendered or visually verified atlas states.""",
    "P11": """API 1.5 amendment: obtain `get_account`, show the linked-card count and actual scope,
then pass that exact ID set to `remove_account` after confirmation. Only
`data.status: removed` leads to P01. `pending` keeps the account visible with cleanup
errors and explicit retry; changed scope requires another review, and shared-route
removal is unavailable. Rooms and operation receipts remain. These are functional
requirements; the rendered dialog does not establish their visual acceptance.
Provider revocation remains unavailable.""",
    "S04": """API 1.6 amendment: Clear Cached State and Clear Recent Operations call separate
native cleanup methods after scope confirmation. Report removed snapshot/receipt
counts. Active journals block history clearing with `RESOURCE_BUSY`; retain the
view and offer an explicit retry. Clearing history includes detailed unknown
outcomes but keeps request-ID replay protection. Cached state may reappear from an
already-running read. Reload local views after interrupted calls. Full Remove Local
Data remains unavailable. These confirmation/busy/error/result variants are a
functional amendment, not newly rendered or visually verified atlas states.""",
}
for b in data['boards']:
    inventory+='## '+b['title']+'\n\n'
    for s in b['screens']:
        ui=s['ui'];sid=s['id']
        inventory+=f"### {sid} — {s['title']}\n\n"
        inventory+=f"[Rendered state](../output/prototypes/homeverb-window-light-v1/screens/{sid}.png) · "
        inventory+=('Root: '+ui['root'] if ui['root'] else 'Nested screen')
        if ui['base']:inventory+=f" · {ui['overlay'].title()} over {ui['base']}"
        inventory+='\n\n'+s['description']+'\n\n'
        pieces=[]
        for block in ui['blocks']:
            if block[0]=='home':pieces.append('Room filters; device cards; observed state/freshness; explicit refresh; separate quick actions; three-root navigation.')
            elif block[0]=='climate':pieces.append('Observed target 24°C; Cooling; room 26°C; explicit target stepper; timestamp. Example values only.')
            elif block[0]=='skin':pieces.append('One Window Light tile, marked Current; stable skin ID window_light.')
            elif block[0]=='skin_preview':pieces.append('Window Light Devices preview using the shared tokens; no state mutation.')
            elif block[0].startswith('tablet'):pieces.append('Three-root rail; responsive device grid or selected-device master/detail view.')
            elif block[0]=='devices':pieces.append('Matching fictional device cards: '+', '.join(block[1])+'.')
            else:
                part=' / '.join(x for x in text_parts(block[1:]) if x not in ['secondary','danger','disabled','warning','success','neutral','error'])
                pieces.append(part)
        inventory+='Visible content: '+'; '.join(pieces)+'\n\n'
        actions=[e for e in data['interactions'] if e['source']==sid]
        inventory+='| Action or outcome | Review destination |\n|---|---|\n'
        for e in actions:inventory+=f"| {e['action'].replace('|','/')} | [{e['target']}](#{e['target'].lower()}--{next(z['title'] for z in screens if z['id']==e['target']).lower().replace(' ','-')}) |\n"
        if not actions:inventory+='| Back / Close | Return to the recorded caller |\n'
        inventory+='\n'
        if sid in runtime_notes:
            inventory += runtime_notes[sid] + '\n\n'
# Stable local image links avoid markdown renderer-specific heading slug rules.
inventory=re.sub(r'\[([A-Z][0-9]{2})\]\(#[^)]+\)',lambda m:f'[{m[1]}](../output/prototypes/homeverb-window-light-v1/screens/{m[1]}.png)',inventory)
(REPO/'docs/ui-screen-inventory.md').write_text(inventory.rstrip()+'\n')

for name in ['window-light-background','ac','lamp','tv','sensor']:
    shutil.copy2(ROOT/'assets'/f'{name}.png',ROOT/'flutter-reference/assets'/f'{name}.png')
manifest=[]
for path in sorted(ROOT.rglob('*')):
    if path.suffix.lower() not in ['.png','.jpg']:continue
    with Image.open(path) as im:dimensions=list(im.size)
    manifest.append({'file':str(path.relative_to(ROOT)),'dimensions':dimensions,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
(ROOT/'assets.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(f'Built offline atlas, coverage, 82-state documentation and manifest for {len(manifest)} image files')
