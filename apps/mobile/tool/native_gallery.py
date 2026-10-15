"""Publish local viewport captures from the actual native Mock widget click test."""
from pathlib import Path
import json
import re
import shutil
from PIL import Image

APP = Path(__file__).resolve().parents[1]
ROOT = APP.parents[1]
SOURCE = APP / 'build/verification/native'
OUT = ROOT / 'output/flutter-verification'
manifest = json.loads((SOURCE / 'manifest.json').read_text())
captures = manifest['captures']
if not captures:
    raise ValueError('No native widget captures were recorded.')
(OUT / 'native').mkdir(parents=True, exist_ok=True)
(OUT / 'reference').mkdir(exist_ok=True)
seen = set()
for capture in captures:
    name = capture['file']
    if not re.fullmatch(r'\d+-[A-Z]\d{2}\.png', name) or name in seen:
        raise ValueError(f'Invalid or duplicate capture filename: {name}')
    if name.split('-')[1] != capture['screen'] + '.png':
        raise ValueError(f'Mismatched screen ID: {name}')
    seen.add(name)
    with Image.open(SOURCE / name) as image:
        if list(image.size) != manifest['size']:
            raise ValueError(f'Wrong viewport size: {name}')
    shutil.copy2(SOURCE / name, OUT / 'native' / name)
for screen in {x['screen'] for x in captures}:
    reference = ROOT / 'output/prototypes/homeverb-window-light-v1/screens' / f'{screen}.png'
    shutil.copy2(reference, OUT / 'reference' / reference.name)
(OUT / 'native/manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
html = '''<!doctype html><html lang="en"><meta charset="utf-8">
<meta name="viewport" content="width=device-width"><title>HomeVerb native Mock click captures</title>
<style>body{margin:0;background:#eeebe3;color:#292b2d;font:15px system-ui}header{padding:20px;background:#f6f2ea;position:sticky;top:0}h1{font-size:22px;margin:0 0 12px}select,button{padding:8px;margin-right:8px}main{display:grid;grid-template-columns:1fr 1fr;gap:20px;padding:20px;max-width:1000px;margin:auto}img{width:100%;height:auto}h2{font-size:16px}p{max-width:900px}a{color:#9e452b}</style>
<header><h1>Native Mock widget click captures</h1>
<p>The tested widgets call the real local Mock Core through FFI. All data is fictional. OS insets are simulated; status icons are absent. These are viewport captures, not physical-phone evidence or pixel-equality results. Test navigation is explicitly labelled separately from clicks.</p>
<a href="index.html">Strict 82-state visual comparison</a><p id="summary"></p>
<button id="previous">Previous</button><select id="capture"></select><button id="next">Next</button><p id="trigger"></p></header>
<main><section><h2>Native Mock widget viewport</h2><img id="actual"></section><section><h2>Layout reference: different sample data</h2><img id="reference"></section></main>
<script>const manifest=DATA;const entries=manifest.captures;const select=document.querySelector('#capture');entries.forEach((x,i)=>select.add(new Option(`${i+1} · ${x.screen}`,i)));document.querySelector('#summary').textContent=`${entries.length} captures · ${new Set(entries.map(x=>x.screen)).size} screen IDs · ${manifest.size.join(' × ')} pixels`;function render(){const x=entries[select.selectedIndex];document.querySelector('#trigger').textContent=x.trigger;document.querySelector('#actual').src='native/'+x.file;document.querySelector('#reference').src='reference/'+x.screen+'.png';}select.onchange=render;document.querySelector('#previous').onclick=()=>{select.selectedIndex=Math.max(0,select.selectedIndex-1);render()};document.querySelector('#next').onclick=()=>{select.selectedIndex=Math.min(entries.length-1,select.selectedIndex+1);render()};render();</script></html>'''
(OUT / 'native-gallery.html').write_text(
    html.replace('DATA', json.dumps(manifest).replace('<', '\\u003c'))
)
print(json.dumps({'captures': len(captures),
                  'screen_ids': len({x['screen'] for x in captures}),
                  'report': str(OUT / 'native-gallery.html')}, indent=2))
