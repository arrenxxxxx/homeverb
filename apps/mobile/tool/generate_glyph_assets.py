"""Export decorative Window Light glyph masks without rendering reference pages."""
import ast
import hashlib
import json
import math
from pathlib import Path
from PIL import Image, ImageDraw

APP = Path(__file__).resolve().parents[1]
SOURCE = APP.parents[1] / 'output/prototypes/homeverb-window-light-v1/render.py'
source = SOURCE.read_text()
# Load only drawing methods; importing the renderer would regenerate references.
canvas = next(node for node in ast.parse(source).body
              if isinstance(node, ast.ClassDef) and node.name == 'Canvas')
canvas.body = [node for node in canvas.body if isinstance(node, ast.FunctionDef)
               and node.name in {'__init__', 'line', 'rect', 'icon', 'step_symbol'}]
namespace = {'ImageDraw': ImageDraw, 'SCALE': 2, 'INK': '#ffffff', 'math': math}
exec(compile(ast.Module(body=[canvas], type_ignores=[]), str(SOURCE), 'exec'), namespace)
Canvas = namespace['Canvas']

sizes = {
    'back': [24], 'check': [14], 'fan': [52], 'home': [22, 28, 52],
    'phone': [52], 'platform': [22, 52], 'plus': [23, 24],
    'power': [22, 24], 'refresh': [16], 'search': [22, 24, 52],
    'settings': [20, 22, 24], 'washer': [52],
    'step-minus': [16, 24], 'step-plus': [16, 24],
}
atlas = Image.new('RGBA', (512, 512))
regions = {}
x = y = row_height = 0
for name, variants in sizes.items():
    for size in variants:
        edge = size * 2 + 8
        if x + edge > atlas.width:
            x, y, row_height = 0, y + row_height, 0
        tile = Image.new('RGBA', (edge, edge))
        c = Canvas(image=tile)
        if name.startswith('step-'):
            c.step_symbol('+' if name == 'step-plus' else '-',
                          2 + size / 2, 2 + size / 2, size, '#ffffff')
        else:
            c.icon(name, 2, 2, size, '#ffffff')
        atlas.paste(c.im, (x, y))
        regions[f'{name}:{size}'] = [x, y, edge, edge]
        x += edge
        row_height = max(row_height, edge)
assert y + row_height <= atlas.height
atlas.crop((0, 0, atlas.width, y + row_height)).save(APP / 'assets/glyphs.png')
(APP / 'assets/glyphs.json').write_text(json.dumps({
    'source_sha256': hashlib.sha256(source.encode()).hexdigest(),
    'scale': 2, 'padding': 2, 'regions': regions,
}, indent=2) + '\n')
print(f'Exported {len(regions)} decorative glyph masks; reference files unchanged.')
