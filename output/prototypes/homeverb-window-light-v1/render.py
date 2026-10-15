"""Render the complete Window Light visual prototype locally, with reproducible assets."""
from pathlib import Path
from functools import lru_cache
import json
import math
import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageFilter

ROOT = Path(__file__).resolve().parent
SCALE = 2
INK, MUTED, ACCENT = '#292b2d', '#626566', '#b94e29'
FONT = Path('/home/arren/.local/share/fonts/apple-like')
data = json.loads((ROOT / 'design.json').read_text())
screens = {s['id']: s for b in data['boards'] for s in b['screens']}
layout = {}

@lru_cache(None)
def font(size, bold=False):
    path = FONT / ('Inter-SemiBold.otf' if bold else 'Inter-Regular.otf')
    if not path.exists():
        path = Path('/usr/share/fonts/truetype/dejavu') / ('DejaVuSans-Bold.ttf' if bold else 'DejaVuSans.ttf')
    return ImageFont.truetype(str(path), round(size * SCALE))

def rgb(value):
    return tuple(bytes.fromhex(value.lstrip('#')))

def background(w, h):
    yy, xx = np.mgrid[:h*SCALE, :w*SCALE]
    x, y = xx / (w*SCALE), yy / (h*SCALE)
    a = np.full((h*SCALE, w*SCALE, 3), (223, 223, 217), dtype=float)
    for cx, cy, sx, sy, color, strength in [
        (-.12,.06,.78,.43,(255,250,228),1), (1.05,.28,.72,.5,(158,178,192),.85),
        (0,.91,.9,.38,(235,204,169),.78), (.85,.95,.8,.25,(246,242,233),.8)]:
        alpha = np.exp(-((x-cx)/sx)**2-((y-cy)/sy)**2)[...,None]*strength
        a = a*(1-alpha)+np.array(color)*alpha
    shadow = np.exp(-((x+.85*y-.83)/.065)**2)*np.exp(-((y-.13)/.30)**2)
    shadow += .6*np.exp(-((x+.85*y-.29)/.042)**2)*np.exp(-((y-.12)/.35)**2)
    a -= shadow[...,None]*np.array((22,20,15))
    a += np.random.default_rng(4).normal(0,.48,(h*SCALE,w*SCALE,1))
    return Image.fromarray(np.clip(a,0,255).astype('uint8')).convert('RGBA')

class Canvas:
    def __init__(self,w=390,h=844,image=None):
        self.w,self.h = w,h
        self.im = image.copy() if image else background(w,h)
        self.draw = ImageDraw.Draw(self.im)
    def line(self,points,fill=INK,width=1):
        self.draw.line([(round(x*SCALE),round(y*SCALE)) for x,y in points],fill=fill,width=round(width*SCALE),joint='curve')
    def rect(self,box,fill,radius=0,outline=None,width=1):
        self.draw.rounded_rectangle(tuple(round(n*SCALE) for n in box),round(radius*SCALE),fill=fill,outline=outline,width=round(width*SCALE))
    def text_width(self,text,size=14,bold=False):
        return self.draw.textlength(str(text),font=font(size,bold))/SCALE
    def text(self,text,x,y,size=14,color=INK,bold=False,width=None,center=False):
        words=str(text).split(' ');lines=['']
        for word in words:
            trial=(lines[-1]+' '+word).strip()
            if width and self.text_width(trial,size,bold)>width and lines[-1]: lines.append(word)
            else: lines[-1]=trial
        for i,line in enumerate(lines):
            dx=x-self.text_width(line,size,bold)/2 if center else x
            self.draw.text((round(dx*SCALE),round((y+i*size*1.4)*SCALE)),line,font=font(size,bold),fill=color,anchor='lt')
        return len(lines)*size*1.4
    def surface(self,x,y,w,h,dark=False,r=22):
        W,H=round(w*SCALE),round(h*SCALE)
        yy,xx=np.mgrid[:H,:W];t=(xx/W*.35+yy/H*.65)[...,None]
        start,end=((66,65,62),(43,46,47)) if dark else ((253,250,243),(238,239,235))
        a=np.array(start)*(1-t)+np.array(end)*t
        tile=Image.fromarray(a.astype('uint8')).convert('RGBA')
        mask=Image.new('L',(W,H));ImageDraw.Draw(mask).rounded_rectangle((0,0,W-1,H-1),round(r*SCALE),fill=255)
        tile.putalpha(mask)
        shadow=Image.new('RGBA',(W+48,H+48));sm=Image.new('L',shadow.size)
        sm.paste(mask.point(lambda v: round(v*.10)),(24,30));sm=sm.filter(ImageFilter.GaussianBlur(10))
        shadow.paste((43,44,46),mask=sm)
        self.im.alpha_composite(shadow,(round(x*SCALE)-24,round(y*SCALE)-24))
        self.im.alpha_composite(tile,(round(x*SCALE),round(y*SCALE)))
        self.draw=ImageDraw.Draw(self.im)
        self.rect((x,y,x+w,y+h),None,r, '#5e605c' if dark else '#ffffff',.6)
    def icon(self,key,x,y,size=24,color=INK):
        u=size/24
        def pts(p):return [(x+a*u,y+b*u) for a,b in p]
        def ln(p):self.line(pts(p),color,1.7*u)
        def rr(p,r=2):self.rect((x+p[0]*u,y+p[1]*u,x+p[2]*u,y+p[3]*u),None,r*u,color,1.6*u)
        if key=='power':
            self.draw.arc(tuple(round(v*SCALE) for v in (x+3*u,y+3*u,x+21*u,y+21*u)),310,230,fill=color,width=round(1.9*u*SCALE));ln([(12,1),(12,12)])
        elif key=='home':ln([(3,10),(12,2),(21,10),(21,21),(15,21),(15,14),(9,14),(9,21),(3,21),(3,10)])
        elif key=='platform':
            ln([(2,8),(12,2),(22,8),(12,14),(2,8)]);ln([(2,13),(12,19),(22,13)]);ln([(2,18),(12,24),(22,18)])
        elif key=='settings':
            rr((3,3,21,21),6);rr((9,9,15,15),3)
            for a,b in [((12,0),(12,4)),((12,20),(12,24)),((0,12),(4,12)),((20,12),(24,12))]:ln([a,b])
        elif key=='search':rr((2,2,17,17),8);ln([(16,16),(23,23)])
        elif key=='refresh':
            self.draw.arc(tuple(round(v*SCALE) for v in (x+3*u,y+3*u,x+21*u,y+21*u)),40,320,fill=color,width=max(1,round(1.7*u*SCALE)));ln([(19,1),(21,7),(15,7)])
        elif key=='plus':ln([(12,2),(12,22)]);ln([(2,12),(22,12)])
        elif key=='back':ln([(15,4),(7,12),(15,20)])
        elif key=='check':ln([(4,12),(10,18),(21,5)])
        elif key=='phone':rr((6,1,19,23),3);ln([(10,19),(15,19)])
        elif key=='washer':rr((3,1,21,23),3);rr((6,8,18,20),6);ln([(6,5),(10,5)])
        elif key=='fan':
            rr((3,1,21,19),9);ln([(12,19),(12,24),(5,24),(19,24)])
            for ang in [0,120,240]:
                a=math.radians(ang);ln([(12,10),(12+7*math.cos(a),10+7*math.sin(a))])
        elif key=='ac':rr((1,5,23,18),3);ln([(4,14),(20,14)])
        elif key=='tv':rr((1,3,23,19),1);ln([(6,19),(3,23)]);ln([(18,19),(21,23)])
        elif key=='sensor':rr((4,2,20,22),5);rr((11,10,13,14),1)
        elif key=='lamp':ln([(6,2),(18,2),(21,13),(3,13),(6,2)]);ln([(12,13),(12,23),(6,23),(18,23)])
        else:rr((3,3,21,21),6)
    def paste_fit(self,img,box):
        ratio=min(box[2]*SCALE/img.width,box[3]*SCALE/img.height)
        cp=img.resize((round(img.width*ratio),round(img.height*ratio)),Image.Resampling.LANCZOS)
        self.im.alpha_composite(cp,(round((box[0]+box[2]/2)*SCALE-cp.width/2),round((box[1]+box[3]/2)*SCALE-cp.height/2)))
        self.draw=ImageDraw.Draw(self.im)

def extract_products():
    src=Image.open(ROOT/'assets/source-device-home.png').convert('RGBA')
    specs={
        'ac':((38,347,254,424),[(7,8),(206,8),(212,14),(212,57),(207,68),(16,68),(5,57),(5,13)]),
        'lamp':((376,328,463,465),[(16,7),(72,7),(81,70),(48,70),(48,119),(67,123),(67,133),(20,133),(20,123),(38,119),(38,70),(7,70)]),
        'tv':((60,701,232,812),[(4,4),(168,4),(168,98),(149,98),(160,108),(155,110),(143,100),(29,100),(16,110),(10,108),(23,99),(4,99)]),
        'sensor':((376,712,460,810),[(15,7),(63,6),(76,16),(77,80),(66,92),(14,91),(5,80),(5,20)])}
    for key,(box,poly) in specs.items():
        im=src.crop(box);mask=Image.new('L',im.size);ImageDraw.Draw(mask).polygon(poly,fill=255)
        im.putalpha(mask.filter(ImageFilter.GaussianBlur(.65)));im.save(ROOT/'assets'/f'{key}.png')

extract_products()
products={k:Image.open(ROOT/'assets'/f'{k}.png').convert('RGBA') for k in ['ac','lamp','tv','sensor']}
background(390,844).save(ROOT/'assets/window-light-background.png')

def statusbar(c):
    c.text('9:41',24,16,12,bold=True)
    for i in range(4):c.rect((c.w-69+i*4,25-i*2,c.w-67+i*4,28),INK,1)
    c.rect((c.w-44,18,c.w-24,28),None,3,INK,1)
    c.rect((c.w-42,20,c.w-27,26),INK,1)
    c.rect((c.w-23,21,c.w-21,25),INK,1)

def nav(c,selected):
    y=c.h-78
    c.rect((0,y,c.w,c.h),'#f6f2ea')
    for i,(label,key) in enumerate([('Devices','home'),('Platforms','platform'),('Settings','settings')]):
        x=c.w*(i+.5)/3;col=ACCENT if selected==label else MUTED
        c.icon(key,x-11,y+12,22,col);c.text(label,x,y+40,11,col,selected==label,center=True)
    c.rect((c.w/2-46,c.h-11,c.w/2+46,c.h-7),INK,2)

NAMES={'ac':('Bedroom AC','Primary Bedroom','24°C','Cooling'), 'lamp':('Floor lamp','Living Room','65%','Brightness'), 'tv':('Living room TV','Living Room','Off','Power'), 'sensor':('Air sensor','Living Room','22°C','46% humidity')}
def device_card(c,key,x,y,w=169,h=233,cached=False,large=False):
    dark=key=='ac';c.surface(x,y,w,h,dark)
    col='#fffaf1' if dark else INK;mut='#d3d1ca' if dark else MUTED
    if large:
        c.paste_fit(products[key],(x+18,y+22,100,95));tx=x+138
        c.text(NAMES[key][0],tx,y+23,19,col,True,width=w-154)
        c.text(NAMES[key][1],tx,y+77,14,mut,width=w-154)
        c.text(NAMES[key][2],tx,y+116,26,col,True)
        c.text(NAMES[key][3],tx,y+154,14,mut)
        c.rect((x+w-62,y+h-63,x+w-14,y+h-15),ACCENT,24)
        c.icon('power',x+w-50,y+h-51,24,'#ffffff')
        return
    c.paste_fit(products[key],(x+15,y+12,w-30,98))
    c.text(NAMES[key][0],x+14,y+117,15,col,True,width=w-24)
    c.text(NAMES[key][1],x+14,y+143,11,mut)
    c.text(NAMES[key][2],x+14,y+h-59,24,col,True)
    c.text('Cached · 2h ago' if cached else NAMES[key][3],x+14,y+h-26,10,mut)
    if key!='sensor':
        fill=ACCENT if key in ['ac','lamp'] and not cached else '#e0ddd5'
        c.rect((x+w-53,y+h-64,x+w-11,y+h-22),fill,21)
        c.icon('power',x+w-43,y+h-54,22,'#ffffff' if fill==ACCENT else INK)

def home(c,mode='all',x=20,y=109,w=350):
    c.text('My home',x,y,16,bold=True);c.text('⌄',x+81,y,16);c.icon('settings',x+w-22,y-1,20,MUTED)
    y+=46
    for label,xx in [('All',x),('Living Room',x+45),('Bedroom',x+149),('Bathroom',x+239)]:
        selected=(label=='Living Room' if mode=='living' else label=='All')
        c.text(label,xx,y,13,INK if selected else MUTED,selected)
        if selected:c.rect((xx,y+25,xx+min(58,c.text_width(label,13)),y+28),ACCENT,2)
    y+=49
    c.text('2 devices' if mode=='living' else '4 devices',x,y,11,MUTED)
    freshness='Cached · 2h ago' if mode=='cached' else 'Updated just now'
    c.text(freshness,x+w-24-c.text_width(freshness,11),y,11,MUTED);c.icon('refresh',x+w-17,y-1,16,MUTED)
    y+=26
    if mode=='large':
        for i,key in enumerate(['ac','lamp']):device_card(c,key,x,y+i*213,w,197,large=True)
        c.text('2 more devices below',x,y+430,14,MUTED)
        return y+455
    keys=['lamp','tv'] if mode=='living' else ['ac','lamp','tv','sensor']
    cw=(w-12)/2
    for i,key in enumerate(keys):device_card(c,key,x+(i%2)*(cw+12),y+(i//2)*247,cw,233,mode=='cached')
    return y+math.ceil(len(keys)/2)*247

def climate(c,x,y,w,freshness='Updated just now',enabled=True):
    c.surface(x,y,w,218,True)
    c.text('OBSERVED TARGET',x+w/2,y+22,10,'#d3d1ca',True,center=True)
    c.text('24°',x+w/2,y+52,68,'#fffaf2',False,center=True)
    c.text('Cooling · Room 26°C',x+w/2,y+134,14,'#e0ddd5',center=True)
    c.rect((x+31,y+165,x+78,y+205),'#535452',20);c.text('−',x+54,y+171,24,'#fff',center=True)
    c.rect((x+w-78,y+165,x+w-31,y+205),ACCENT if enabled else '#535452',20);c.text('+',x+w-54,y+171,24,'#fff',center=True)
    c.text(freshness,x+w/2,y+179,10,'#d3d1ca',center=True)
    return 230

def rows(c,items,x,y,w,title=None):
    heights=[]
    for label,value in items:
        heights.append(62 if c.text_width(label,13)+c.text_width(value,12)>w-50 else 44)
    total=sum(heights)+(34 if title else 0)
    c.surface(x,y,w,total,r=18)
    cy=y
    if title:c.text(title,x+16,cy+13,14,bold=True);cy+=34
    for i,((label,value),height) in enumerate(zip(items,heights)):
        c.text(label,x+16,cy+14,13,bold=False)
        if label in ['Power','Oscillation','Muted'] and value in ['On','Off']:
            on=value=='On';c.text(value,x+w-89,cy+15,12,MUTED)
            c.rect((x+w-58,cy+9,x+w-14,cy+35),ACCENT if on else '#c2c4bc',13)
            cx=x+w-27 if on else x+w-45;c.rect((cx-10,cy+12,cx+10,cy+32),'#fffcf5',10)
        elif label in ['Target','Speed'] and '+' in value:
            value=value.strip().removeprefix('−').strip().removesuffix('+').strip()
            for cx,sign in [(x+w-125,'−'),(x+w-30,'+')]:
                c.rect((cx-15,cy+7,cx+15,cy+37),'#e6e2d8',15);c.text(sign,cx,cy+13,16,INK,center=True)
            c.text(value,x+w-78,cy+16,12,INK,True,center=True)
        elif '≡' in value:
            value=value.replace('≡','').strip()
            for dy in [0,4,8]:c.line([(x+w-27,cy+18+dy),(x+w-16,cy+18+dy)],MUTED,1)
            c.text(value,x+w-40-c.text_width(value,12),cy+15,12,MUTED)
        elif height>44:c.text(value,x+16,cy+37,12,MUTED,width=w-32)
        else:c.text(value,x+w-16-c.text_width(value,12),cy+15,12,MUTED)
        cy+=height
        if i<len(items)-1:c.line([(x+16,cy),(x+w-16,cy)],'#deded7',.6)
    return total+12

def block(c,b,x,y,w):
    k=b[0]
    if k=='note':return c.text(b[1],x+2,y+2,13,MUTED,width=w-4)+12
    if k=='section':c.text(b[1],x+3,y+4,10,MUTED,True);return 25
    if k=='link':return c.text(b[1],x+2,y+9,13,ACCENT,True,width=w-4)+22
    if k=='rows':return rows(c,b[1],x,y,w)
    if k=='card':return rows(c,b[2],x,y,w,b[1])
    if k=='button':
        style=b[2] if len(b)>2 else '';fill=ACCENT if style=='' else '#eeebe3';col='#fffaf5' if style=='' else INK
        if style=='danger':fill='#fae7df';col='#9e3421'
        if style=='disabled':fill='#e4e2dc';col='#888781'
        c.rect((x,y,x+w,y+48),fill,15, '#d7d6cf' if style=='secondary' else None)
        c.text(b[1],x+w/2,y+16,14,col,True,center=True);return 60
    if k=='field':
        c.text(b[1],x+2,y,12,MUTED);c.surface(x,y+22,w,50,r=14)
        c.text(b[2] or 'Search by name',x+15,y+38,15,INK if b[2] else MUTED)
        if len(b)>3:
            c.rect((x,y+22,x+w,y+72),None,14,'#ae452d',1)
            return 83+c.text(b[3],x+2,y+81,12,'#9e3421',width=w-4)
        return 88
    if k=='choices':
        cy=y;selected=b[2] if len(b)>2 else 0
        selected=selected if isinstance(selected,list) else [selected]
        for i,item in enumerate(b[1]):
            n=max(1,math.ceil(c.text_width(item,13)/(w-65)));h=34+n*18
            c.surface(x,cy,w,h,r=16)
            if i in selected:c.rect((x,cy,x+w,cy+h),None,16,ACCENT,1.2)
            c.text(item,x+15,cy+18,13,INK,i in selected,width=w-60)
            c.rect((x+w-34,cy+h/2-9,x+w-16,cy+h/2+9),ACCENT if i in selected else None,9,None if i in selected else '#b3b3ac')
            if i in selected:c.icon('check',x+w-32,cy+h/2-7,14,'#ffffff')
            cy+=h+9
        return cy-y+4
    if k in ['chips','tabs']:
        cx=x;cy=y
        for i,label in enumerate(b[1]):
            tw=c.text_width(label,12)+24
            if cx+tw>x+w:cx=x;cy+=42
            selected=(k=='tabs' and i==b[2]) or (k=='chips' and label=='3')
            c.rect((cx,cy,cx+tw,cy+32),INK if selected else '#f8f5ee',16)
            c.text(label,cx+12,cy+10,12,'#fff' if selected else MUTED)
            cx+=tw+7
        return cy-y+47
    if k=='hero':
        c.surface(x,y,w,200)
        if b[1] in products:c.paste_fit(products[b[1]],(x+75,y+13,w-150,87))
        else:c.icon(b[1],x+w/2-26,y+27,52,ACCENT)
        c.text(b[2],x+w/2,y+119,23,INK,True,center=True)
        # Center wrapped support copy within the card.
        h=c.text(b[3],x+20,y+158,12,MUTED,width=w-40)
        return 214
    if k=='climate':return climate(c,x,y,w,b[1] if len(b)>1 else 'Updated just now',b[2] if len(b)>2 else True)
    if k=='status':
        palette={'warning':('#f8ead9','#855428'),'success':('#e8eee2','#476344'),'error':('#f7e4de','#9e3421'),'neutral':('#e7ebea','#4c6067')}
        fill,col=palette[b[3]];lines=math.ceil(c.text_width(b[2],12)/(w-32));h=52+lines*17
        c.rect((x,y,x+w,y+h),fill,17)
        c.text(b[1],x+16,y+15,14,col,True,width=w-32)
        c.text(b[2],x+16,y+39,12,col,width=w-32);return h+13
    if k=='progress':
        c.rect((x,y+7,x+w,y+13),'#deded6',3);c.rect((x,y+7,x+w*b[1],y+13),ACCENT,3)
        return 26+c.text(b[2],x,y+25,12,MUTED,width=w)+12
    if k=='slider':
        c.surface(x,y,w,94,r=18);c.text(b[1],x+16,y+16,13);c.text(b[2],x+w-16-c.text_width(b[2],13,True),y+16,13,INK,True)
        c.rect((x+20,y+61,x+w-20,y+66),'#d5d6cf',3);cx=x+20+(w-40)*b[3]
        c.rect((x+20,y+61,cx,y+66),ACCENT,3);c.rect((cx-10,y+53,cx+10,y+73),'#fffaf3',10,ACCENT,2);return 107
    if k=='stepper':return rows(c,[[b[1],'−       '+b[2]+'       +']],x,y,w)
    if k=='code':
        c.surface(x,y,w,94,r=20);c.text(b[1],x+w/2,y+31,32,INK,True,center=True);return 109
    if k=='devices':
        for i,key in enumerate(b[1]):device_card(c,key,x+(i%2)*(w/2+6),y+(i//2)*247,(w-12)/2)
        return math.ceil(len(b[1])/2)*247
    if k=='skin':
        preview=background(166,100);pc=Canvas(166,100,preview)
        pc.surface(10,12,65,76,True,12);pc.surface(84,12,71,76,r=12)
        pc.paste_fit(products['ac'],(17,23,52,30));pc.paste_fit(products['lamp'],(104,20,31,47))
        pc.rect((22,65,62,72),ACCENT,3)
        c.surface(x,y,w,205);c.paste_fit(pc.im,(x+12,y+12,w-24,135))
        c.text('Window Light',x+18,y+159,17,INK,True);c.text('Current',x+w-76,y+162,12,ACCENT,True)
        c.rect((x,y,x+w,y+205),None,22,ACCENT,1.2);return 221
    if k=='skin_preview':
        preview=render('D01');c.paste_fit(preview,(x,y,w,393));return 408
    raise ValueError('Unknown block: '+k)

def rail(c):
    c.rect((0,0,88,c.h),'#f4f0e8');c.icon('home',30,26,28,ACCENT)
    for i,(key,name) in enumerate([('home','Devices'),('platform','Platforms'),('settings','Settings')]):
        if i==0:c.rect((12,118,76,190),'#e7d3c4',20)
        c.icon(key,33,134+i*92,22,ACCENT if i==0 else MUTED)
        c.text(name,44,166+i*92,10,ACCENT if i==0 else MUTED,center=True)

@lru_cache(None)
def render(sid):
    s=screens[sid];ui=s['ui'];wide=sid.startswith('W');w,h=(1024,768) if wide else (390,844)
    c=Canvas(w,h)
    if wide:
        rail(c);c.text('HomeVerb',118,37,28,INK,True);c.text('My home',118,83,16)
        if sid=='W01':
            c.icon('search',925,40,24);c.icon('plus',972,39,24);c.icon('settings',974,84,20,MUTED)
            block(c,['tabs',['All','Living Room','Primary Bedroom','Bathroom'],0],118,127,824)
            c.text('4 devices · Updated just now',118,183,13,MUTED)
            for i,key in enumerate(['ac','lamp','tv','sensor']):device_card(c,key,118+i%3*282,215+i//3*253,264,235)
        else:
            rows(c,[['Bedroom AC','Selected'],['Floor lamp','Living Room'],['Living room TV','Living Room'],['Air sensor','Living Room']],118,138,276)
            c.text('Bedroom AC',430,137,27,INK,True);c.text('Primary Bedroom',430,178,14,MUTED)
            climate(c,430,216,550);rows(c,[['Power','On'],['Mode','Cool  ›'],['Fan','Auto  ›']],430,459,550)
            block(c,['button','Refresh State','secondary'],430,620,550)
        layout[sid]={'logical_size':[w,h],'content_bottom':720,'scrollable':False}
        return c.im
    statusbar(c)
    if ui['base']:
        c=Canvas(w,h,render(ui['base']));c.im.alpha_composite(Image.new('RGBA',c.im.size,(20,24,27,95)));c.draw=ImageDraw.Draw(c.im)
        # Measure the same blocks on a scratch surface before positioning the sheet.
        probe=Canvas();dialog=ui['overlay']=='dialog';cw=298 if dialog else 350
        title_h=probe.text(ui['header'],20,0,22 if dialog else 23,INK,True,width=cw)
        need=sum(block(probe,b,20,0,cw) for b in ui['blocks'])+title_h+(56 if dialog else 80)
        if ui['overlay']=='dialog':
            sw=338;sx=26;top=(h-need)/2;c.surface(sx,top,sw,need,r=25);x=sx+20;cw=sw-40
            cy=top+25+c.text(ui['header'],x,top+25,22,INK,True,width=cw)+17
        else:
            top=max(155,h-need);c.surface(0,top,w,h-top+24,r=28);c.rect((w/2-18,top+11,w/2+18,top+15),'#bbbcb5',2)
            x=20;cw=350;cy=top+39+c.text(ui['header'],x,top+39,23,INK,True,width=cw)+21
        for b in ui['blocks']:cy+=block(c,b,x,cy,cw)
        layout[sid]={'logical_size':[w,h],'content_bottom':round(cy,1),'scrollable':False}
    else:
        root=ui['root'];x=20;cw=350
        if root:
            if sid=='P01':cy=57+c.text(ui['header'],x,57,28,INK,True,width=340)+23
            else:c.text(ui['header'],x,60,30,INK,True);cy=118
            if root=='Devices':c.icon('search',300,65,22);c.icon('plus',345,64,23)
        else:
            c.icon('back',16,58,24);cy=100+c.text(ui['header'],x,100,25,INK,True,width=cw)+22
            c.text('HomeVerb',w-88,62,11,MUTED)
        if ui['subtitle']:cy+=c.text(ui['subtitle'],x,cy-9,13,MUTED)+9
        if ui['blocks'][0][0]=='home':
            cy=home(c,ui['blocks'][0][1],20,110,350)
        else:
            for b in ui['blocks']:cy+=block(c,b,x,cy,cw)
        if root:nav(c,root)
        else:c.rect((w/2-46,h-11,w/2+46,h-7),INK,2)
        layout[sid]={'logical_size':[w,h],'content_bottom':round(cy,1),'scrollable':sid=='D12'}
    return c.im

def build():
    for sid,s in screens.items():
        im=render(sid).convert('RGB');im.save(ROOT/'screens'/f'{sid}.png')
        s['dimensions']=list(im.size)
    # Four-page boards remain readable as separate full-resolution screens.
    for board in data['boards']:
        wide=board['slug'].startswith('20-');bw,bh=(2160,1050) if wide else (1820,1160)
        sheet=Canvas(bw//2,bh//2,Image.new('RGBA',(bw,bh),'#ece9e1'))
        sheet.text('HOMEVERB   /   WINDOW LIGHT',26,20,10,ACCENT,True)
        sheet.text(board['title'],26,43,22,INK,True)
        sheet.text(board['subtitle'],26,76,12,MUTED)
        tw=480 if wide else 206
        for i,s in enumerate(board['screens']):
            xx=26+i*(tw+12)
            sheet.paste_fit(render(s['id']),(xx,110,tw,360 if wide else 446))
            sheet.text(s['id']+' / '+s['title'],xx,480 if wide else 560,10,INK,True,width=tw)
        path=ROOT/'boards'/f"{board['slug']}.png";sheet.im.convert('RGB').save(path)
        board.update(image=f"boards/{board['slug']}.png",thumbnail=f"boards/{board['slug']}-thumb.jpg",dimensions=list(sheet.im.size))
        thumb=sheet.im.convert('RGB');thumb.thumbnail((1365,870),Image.Resampling.LANCZOS);thumb.save(ROOT/board['thumbnail'],quality=91)
    for name,ids,cols in [('key-pages.jpg',['D01','C01','P01','S01','S02','H04'],3),('overview.jpg',list(screens),8)]:
        cw,ch=(300,720) if name=='key-pages.jpg' else (146,345)
        hh=96+math.ceil(len(ids)/cols)*ch
        sheet=Image.new('RGB',(cols*cw+40,hh),'#ece9e1');d=ImageDraw.Draw(sheet)
        d.text((24,18),'HomeVerb / Window Light',font=font(16,True),fill=INK)
        d.text((24,61),'One included skin · Complete screen and recovery coverage',font=font(7),fill=MUTED)
        for i,sid in enumerate(ids):
            im=render(sid).convert('RGB');im.thumbnail((cw-20,ch-39),Image.Resampling.LANCZOS)
            xx=20+i%cols*cw;yy=94+i//cols*ch;sheet.paste(im,(xx+(cw-20-im.width)//2,yy))
            d.text((xx,yy+ch-31),sid+' / '+screens[sid]['title'],font=font(5 if cols==8 else 7,True),fill=INK)
        sheet.save(ROOT/name,quality=94)
    (ROOT/'design.json').write_text(json.dumps(data,indent=2)+'\n')
    (ROOT/'layout.json').write_text(json.dumps(layout,indent=2)+'\n')
    over=[(sid,v) for sid,v in layout.items() if not v['scrollable'] and v['content_bottom']>(766 if screens[sid]['ui']['root'] and not sid.startswith('W') else v['logical_size'][1]-18)]
    print('Rendered',len(screens),'screens and',len(data['boards']),'boards. Layout overflow:',over)

if __name__=='__main__':build()
