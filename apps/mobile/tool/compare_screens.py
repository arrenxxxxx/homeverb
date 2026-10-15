"""Compare actual Flutter captures with the approved raster references; never bless differences."""
from pathlib import Path
import argparse
import json
import shutil
from PIL import Image, ImageChops, ImageStat

APP = Path(__file__).resolve().parents[1]
ROOT = APP.parents[1]
REFERENCE = ROOT / 'output/prototypes/homeverb-window-light-v1/screens'
CAPTURES = APP / 'build/verification/screens'
OUT = ROOT / 'output/flutter-verification'
OUT.mkdir(parents=True, exist_ok=True)
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--require-exact', action='store_true', help='Fail if any reference pixel differs.')
args = parser.parse_args()
expected = {p.name for p in REFERENCE.glob('*.png')}
actual_names = {p.name for p in CAPTURES.glob('*.png')}
if actual_names != expected or len(expected) != 82:
    raise ValueError(f'Incomplete capture set: missing={expected-actual_names}, extra={actual_names-expected}')
for name in ['reference', 'actual', 'diff']:
    (OUT / name).mkdir(exist_ok=True)
results = []
for actual in sorted(CAPTURES.glob('*.png')):
    reference = REFERENCE / actual.name
    before, after = Image.open(reference).convert('RGB'), Image.open(actual).convert('RGB')
    if before.size != after.size:
        raise ValueError(f'{actual.stem}: size mismatch {before.size} != {after.size}')
    diff = ImageChops.difference(before, after)
    changed = ImageChops.lighter(ImageChops.lighter(*diff.split()[:2]), diff.split()[2])
    histogram = changed.histogram()
    total = before.width * before.height
    results.append({'id': actual.stem, 'size': list(before.size),
        'identical': diff.getbbox() is None,
        'changed_pixels_percent': round((total - histogram[0]) * 100 / total, 3),
        'mean_absolute_channel_error': round(sum(ImageStat.Stat(diff).mean) / 3, 4),
        'amended_contract': actual.stem == 'P08'})
    shutil.copy2(reference, OUT / 'reference' / reference.name)
    shutil.copy2(actual, OUT / 'actual' / actual.name)
    diff.point(lambda v: min(255, v * 4)).save(OUT / 'diff' / actual.name)
(OUT / 'metrics.json').write_text(json.dumps(results, indent=2) + '\n')
html = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>HomeVerb Flutter visual verification</title><style>
body{margin:0;background:#eeebe3;color:#292b2d;font:15px system-ui}header{position:sticky;top:0;background:#f6f2ea;padding:18px 24px;border-bottom:1px solid #ccc;z-index:1}h1{font-size:22px;margin:0 0 8px}select{padding:8px;min-width:120px}main{padding:20px;display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:18px}img{width:100%;height:auto;display:block}h2{font-size:16px}.note{color:#855428}#metrics{margin:8px 0 0}button{padding:8px;margin-right:8px}
</style><header><h1>HomeVerb · Flutter / approved prototype</h1>
<p class="note">Visual gate NOT PASSED: captures are native Flutter widgets, but differences remain. P08 intentionally follows the amended contract. System chrome is deterministic test artwork; physical phones are unverified.</p>
<button id="prev">Previous</button><select id="screen"></select><button id="next">Next</button><p id="metrics"></p></header>
<main><section><h2>Approved prototype</h2><img id="reference"></section><section><h2>Flutter capture</h2><img id="actual"></section><section><h2>Absolute difference ×4</h2><img id="diff"></section></main>
<script>const data=DATA;const select=document.querySelector('#screen');data.forEach(x=>select.add(new Option(x.id,x.id)));function render(){const x=data[select.selectedIndex];for(const kind of ['reference','actual','diff'])document.getElementById(kind).src=kind+'/'+x.id+'.png';document.getElementById('metrics').textContent=`${x.size.join(' × ')} · Changed pixels ${x.changed_pixels_percent}% · Mean channel error ${x.mean_absolute_channel_error}/255 · Exact match: ${x.identical}`;}select.onchange=render;document.querySelector('#prev').onclick=()=>{select.selectedIndex=Math.max(0,select.selectedIndex-1);render();};document.querySelector('#next').onclick=()=>{select.selectedIndex=Math.min(data.length-1,select.selectedIndex+1);render();};render();</script></html>'''
(OUT / 'index.html').write_text(html.replace('DATA', json.dumps(results)))
print(json.dumps({'screens': len(results), 'exact_matches': sum(x['identical'] for x in results),
    'mean_channel_error': round(sum(x['mean_absolute_channel_error'] for x in results) / len(results), 4),
    'report': str(OUT / 'index.html')}, indent=2))
if args.require_exact and not all(x['identical'] for x in results):
    raise SystemExit(1)
