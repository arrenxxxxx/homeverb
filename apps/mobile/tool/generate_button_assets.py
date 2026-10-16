"""Generate decorative button faces without text or interaction."""
from pathlib import Path
from PIL import Image, ImageDraw

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
