"""Check atlas completeness, references, rendered bounds and asset integrity."""
from pathlib import Path
from html.parser import HTMLParser
import hashlib
import json
import re
from urllib.parse import unquote, urlparse
from PIL import Image

ROOT=Path(__file__).resolve().parent
REPO=ROOT.parents[2]
data=json.loads((ROOT/'design.json').read_text())
old=json.loads((ROOT/'source-inventory.json').read_text())
screens=[s for b in data['boards'] for s in b['screens']]
ids={s['id'] for s in screens}
assert len(data['boards'])==21 and len(screens)==len(ids)==82
assert ids=={s['id'] for b in old['boards'] for s in b['screens']}
assert len(data['flows'])==14
assert {sid for f in data['flows'] for sid in f['screens']}==ids
assert len(data['interactions'])==198
assert len({(e['source'],e['action'],e['target']) for e in data['interactions']})==198
assert all(e['source'] in ids and e['target'] in ids for e in data['interactions'])
assert all(s['ui']['blocks'] for s in screens)
assert {s['ui']['root'] for s in screens if s['ui']['root']}=={'Devices','Platforms','Settings'}
assert data['skin']=={'default':'window_light','available':[{'id':'window_light','name':'Window Light'}]}
by_id={s['id']:s for s in screens}
assert by_id['S02']['ui']['blocks'].count(['skin'])==1
assert by_id['S11']['title']=='Window Light skin preview'
assert by_id['C04']['ui']['blocks'][0]==['climate','Read 2 min ago',False]
assert all('dark appearance' not in b['subtitle'].lower() for b in data['boards'])

layout=json.loads((ROOT/'layout.json').read_text())
assert set(layout)==ids
for sid,entry in layout.items():
    w,h=entry['logical_size']
    limit=h-78 if by_id[sid]['ui']['root'] and not sid.startswith('W') else h-18
    assert entry['content_bottom']<=limit,(sid,entry,limit)
    assert by_id[sid]['dimensions']==[w*2,h*2]

manifest=json.loads((ROOT/'assets.json').read_text())
manifest_paths={e['file'] for e in manifest}
assert len(manifest_paths)==len(manifest)
for entry in manifest:
    path=ROOT/entry['file']
    assert hashlib.sha256(path.read_bytes()).hexdigest()==entry['sha256'],path
    with Image.open(path) as im:
        assert list(im.size)==entry['dimensions'],path
        im.verify()
for s in screens:assert s['image'] in manifest_paths
for b in data['boards']:
    assert b['image'] in manifest_paths and b['thumbnail'] in manifest_paths
for name in ['window-light-background','ac','lamp','tv','sensor']:
    assert (ROOT/'assets'/f'{name}.png').read_bytes()==(ROOT/'flutter-reference/assets'/f'{name}.png').read_bytes()

html=(ROOT/'index.html').read_text()
embedded=re.search(r'<script id="design-data" type="application/json">(.*?)</script>',html,re.S)
assert json.loads(embedded[1])==data
assert '__DESIGN_DATA__' not in html and '__SCREEN_COUNT__' not in html
assert 'gpt-image-2' not in html and '1672 × 941' not in html
assert 'without redrawing' not in html
assert all(token in html for token in ['data-mode="boards"','data-mode="screens"','data-mode="journeys"','showModal','ArrowRight','Preview Window Light'])
scripts=re.findall(r'<script>(.*?)</script>',html,re.S)
assert len(scripts)==1
Path('/tmp/homeverb-window-light-atlas.js').write_text(scripts[0])

links=[]
class LinkReader(HTMLParser):
    def handle_starttag(self,tag,attrs):
        for key,value in attrs:
            if key in ['href','src'] and value:links.append((ROOT/'index.html',value))
LinkReader().feed(html)
files=[*ROOT.glob('*.md'),ROOT/'flutter-reference/README.md',
       REPO/'docs/ui-functional-spec.md',REPO/'docs/ui-window-light.md',REPO/'docs/ui-screen-inventory.md']
for path in files:
    links.extend((path,url) for url in re.findall(r'\[[^\]]*\]\(([^\s)]+)\)',path.read_text()))
checked=0
for source,url in links:
    parsed=urlparse(url)
    if parsed.scheme or not parsed.path:continue
    path=source.parent/unquote(parsed.path)
    assert path.exists(),(source,url)
    checked+=1

inventory=(REPO/'docs/ui-screen-inventory.md').read_text()
assert set(re.findall(r'^### ([A-Z][0-9]{2}) ',inventory,re.M))==ids
coverage=(ROOT/'coverage.md').read_text().split('## Window Light acceptance additions')[0]
assert len(re.findall(r'^\| [^|]+ \|',coverage,re.M))-1==41
print(json.dumps({'screens':82,'boards':21,'journeys':14,'branches':198,
    'requirement_groups':41,'skin_options':1,'images_verified':len(manifest),
    'local_links_verified':checked,'layout_overflow':0,'flutter_build':'not_run_sdk_unavailable',
    'browser_interaction_test':'not_run'},indent=2))
