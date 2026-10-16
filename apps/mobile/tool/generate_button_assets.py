"""Generate decorative button faces and font-derived label metrics."""
import hashlib
import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

OUT = Path(__file__).resolve().parents[1] / 'assets'
# The independent prototype uses inclusive rounded-rectangle bounds at 2x.
# A one-pixel center stretch preserves the 30-pixel corners and two-pixel border.
for name, fill, border in [
    ('primary', '#b94e29', None),
    ('secondary', '#eeebe3', '#d7d6cf'),
    ('danger', '#fae7df', None),
    ('disabled', '#e4e2dc', None),
]:
    face = Image.new('RGBA', (61, 61))
    ImageDraw.Draw(face).rounded_rectangle(
        (0, 0, 60, 60), radius=30, fill=fill, outline=border, width=2,
    )
    face.save(OUT / f'button-{name}.png')

# Baseline-relative ink bounds come from the bundled font, not page captures.
# Include ASCII for dynamic counts and all characters in the current button labels.
font_path = OUT / 'fonts/Inter-SemiBold.otf'
font = ImageFont.truetype(str(font_path), 28)
catalog = json.loads((OUT / 'prototype.json').read_text())
characters = set(chr(i) for i in range(32, 127))
for board in catalog['boards']:
    for screen in board['screens']:
        for block in screen['ui']['blocks']:
            if block[0] == 'button':
                characters.update(block[1])
bounds = {}
for character in sorted(characters):
    box = font.getbbox(character, anchor='ls')
    bounds[character] = [box[1], box[3]]
(OUT / 'button-label-metrics.json').write_text(json.dumps({
    'font_sha256': hashlib.sha256(font_path.read_bytes()).hexdigest(),
    'font_size_pixels': 28,
    'bounds': bounds,
}, indent=2) + '\n')
