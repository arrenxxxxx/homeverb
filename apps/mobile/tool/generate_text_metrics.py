"""Record independent font ink bounds for native text alignment."""
import hashlib
import json
from pathlib import Path
from PIL import ImageFont

ASSETS = Path(__file__).resolve().parents[1] / 'assets'
characters = set(chr(i) for i in range(32, 127)) | {'…'}


def collect(value):
    if isinstance(value, str):
        characters.update(value)
    elif isinstance(value, list):
        for item in value:
            collect(item)
    elif isinstance(value, dict):
        for item in value.values():
            collect(item)


collect(json.loads((ASSETS / 'prototype.json').read_text()))
variants = []
for weight, sizes, filename in [
    (400, [10, 11, 12, 13, 14, 16, 68], 'Inter-Regular.otf'),
    (600, [10, 11, 12, 13, 14, 15, 16, 17, 19, 22, 23, 24, 25, 26, 27, 28, 30, 32], 'Inter-SemiBold.otf'),
]:
    path = ASSETS / 'fonts' / filename
    for size in sizes:
        font = ImageFont.truetype(str(path), size * 2)
        missing = font.getmask(chr(0x10ffff))
        bounds = {}
        for character in sorted(characters):
            mask = font.getmask(character)
            # Keep missing glyphs on Flutter's ordinary font-fallback path.
            if mask.size == missing.size and bytes(mask) == bytes(missing):
                continue
            box = font.getbbox(character, anchor='ls')
            bounds[character] = [box[1], box[3]]
        variants.append({
            'weight': weight, 'size': size,
            'font_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'bounds': bounds,
        })
(ASSETS / 'text-ink-metrics.json').write_text(json.dumps({
    'pixel_ratio': 2, 'variants': variants,
}, indent=2) + '\n')
