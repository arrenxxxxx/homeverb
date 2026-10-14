#!/usr/bin/env python3
"""Render the documentation diagram. SVG uses stdlib; PNG requires cairosvg."""
from pathlib import Path
from html import escape
R = Path(__file__).resolve().parents[1]
OUT = R / 'docs' / 'diagrams'
OUT.mkdir(parents=True, exist_ok=True)
parts = ['''<svg xmlns="http://www.w3.org/2000/svg" width="1320" height="1390" viewBox="0 0 1320 1390" role="img" aria-labelledby="title desc">
<title id="title">HomeVerb architecture without a persistent service</title>
<desc id="desc">CLI, App, and MCP each embed the same control library and directly access appliance LAN interfaces or vendor clouds. No HA, NAS gateway, project backend, or background automation.</desc>
<defs><marker id="arrow" markerWidth="10" markerHeight="10" refX="8" refY="5" orient="auto"><path d="M0,0 L10,5 L0,10 z" fill="#8a9bb1"/></marker></defs>
<rect width="1320" height="1390" fill="#f5f7fc"/>
<g font-family="DejaVu Sans, sans-serif">''']
def box(x,y,w,h,fill='#ffffff',stroke='#cbd7e8',r=16):
    parts.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{fill}" stroke="{stroke}" stroke-width="1.2"/>')
def text(x,y,t,size=20,color='#344f6d',weight=400):
    parts.append(f'<text x="{x}" y="{y}" font-size="{size}" fill="{color}" font-weight="{weight}">{escape(t)}</text>')
def line(x1,y1,x2,y2,arrow=True,dash=False):
    parts.append(f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="#8a9bb1" stroke-width="1.8"'+(' marker-end="url(#arrow)"' if arrow else '')+(' stroke-dasharray="6 5"' if dash else '')+'/>')
text(48,44,'HOMEVERB / ARCHITECTURE BASELINE',15,'#52709a',700)
text(48,91,'Cross-brand control. No persistent service.',32,'#152d4a',700)
text(48,126,'One embedded library. Three independent hosts. No central server.',21,'#607594')
entries=[(48,'CLI process','#e9efff','#315db5',['Command parsing / structured JSON','Embedded Core / exit on completion']),
         (464,'Mobile app process','#e6f3ef','#147760',['Flutter UI / in-process FFI','Embedded Core / foreground only']),
         (880,'MCP session process','#f0eafd','#7150a4',['AI host / stdio tool calls','Embedded Core / host-owned lifetime'])]
for x,title,fill,c,lines in entries:
    box(x,159,392,170,fill,fill)
    text(x+22,199,title,25,c,700)
    text(x+22,242,lines[0],17)
    text(x+22,282,lines[1],17,c,700)
    line(x+196,329,x+196,349,False,True)
text(212,369,'Shared code loaded by each host below - not a network service',19,'#607594')
box(48,397,920,330)
text(68,438,'Unified control Core',27,'#152d4a',700)
text(744,438,'In-process library',17,'#7184a0')
node_data=[('Accounts and auth','Login / pairing / refresh'),('Device registry','Stable IDs / parts / routes'),('Capability model','Properties / actions / schemas'),('Policy and routing','Permissions / LAN or cloud'),('Execute and verify','Deadlines / cancel / safe retries'),('State and results','Timestamps / source / unknown')]
for idx,(title,body) in enumerate(node_data):
    x=68+(idx%3)*298;y=465+(idx//3)*120
    box(x,y,280,98,'#f5f8fe','#dfe7f4',11)
    text(x+16,y+35,title,23,'#254d76',700)
    text(x+16,y+72,body,15,'#607594')
box(994,397,278,330,'#fffdf7','#e8ddbf')
text(1014,438,'Local persistence',24,'#82662c',700)
text(1014,484,'SQLite',23,'#505449',700)
text(1014,520,'Devices / config / snapshots',17,'#777767')
line(1014,543,1252,543,False)
text(1014,583,'System secret store',21,'#505449',700)
text(1014,619,'Tokens / passwords / keys',17,'#777767')
text(1014,681,'Per installation; no auto-sync',16,'#927b4c')
line(968,562,991,562)
line(508,730,508,774)
text(532,759,'Common Adapter contract',18,'#607594')
box(48,789,1224,188)
text(68,833,'Platform / protocol adapters',27,'#152d4a',700)
text(726,833,'Candidates only; no verified drivers',17,'#ac6b43')
providers=[('Midea protocols','Hualing: verify exact models'),('Samsung TV LAN','Pairing / protocol / wake: TBD'),('SmartThings cloud','Native authorization: TBD'),('Hisense platforms','Region / account / auth: TBD')]
for i,(title,body) in enumerate(providers):
    x=68+i*300
    box(x,861,284,91,'#f5f8fe','#dfe7f4',10)
    text(x+12,897,title,21,'#31577f',700)
    text(x+12,932,body,16,'#6c819e')
line(660,978,660,997,False)
line(346,997,973,997,False)
line(346,997,346,1022)
line(973,997,973,1022)
box(48,1035,596,159,'#e9f4f5','#cce3e6')
text(70,1077,'Appliance LAN interfaces',26,'#226471',700)
text(70,1120,'Direct access by the current client process',20,'#427484')
text(70,1158,'LAN-only targets must be reachable from that client.',17,'#61868d')
box(676,1035,596,159,'#ecf1fb','#d4dff0')
text(698,1077,'Vendor cloud APIs',26,'#3b5e91',700)
text(698,1120,'Direct authorized access; not a project backend',20,'#526f9d')
text(698,1158,'Backend-only confidential auth flows are excluded.',17,'#7188ad')
box(48,1230,1224,100,'#182f4a','#182f4a')
text(70,1268,'Explicitly excluded',21,'#ffffff',700)
text(70,1307,'HA / NAS gateways / daemons / project backend / HTTP MCP / background automation',21,'#dce6f5')
text(48,1365,'Baseline v0.1.1-docs | MCP may await calls during its host session; no device polling or background jobs.',16,'#7184a0')
parts.append('</g></svg>')
svg='\n'.join(parts) + '\n'
(OUT/'architecture-overview.svg').write_text(svg,encoding='utf-8')
try:
    import cairosvg
except ImportError:
    print('SVG generated. Optional PNG rendering requires: python -m pip install cairosvg')
else:
    cairosvg.svg2png(bytestring=svg.encode(),write_to=str(OUT/'architecture-overview.png'),scale=1.5)
    print('SVG and PNG generated.')
