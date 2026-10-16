"""Export the Window Light surface corner masks and inclusive outline artwork."""
from pathlib import Path
from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[1] / 'assets'
for radius in [12, 14, 16, 18, 20, 22, 25, 28]:
    pixels = radius * 2
    edge = pixels * 2 + 1
    mask = Image.new('RGBA', (edge, edge))
    ImageDraw.Draw(mask).rounded_rectangle(
        (0, 0, edge - 1, edge - 1), pixels, fill='white',
    )
    mask.save(OUT / f'surface-{radius}-mask.png')
    for width in [1, 2]:
        border = Image.new('RGBA', (edge, edge))
        ImageDraw.Draw(border).rounded_rectangle(
            (0, 0, edge - 1, edge - 1), pixels, outline='white', width=width,
        )
        border.save(OUT / f'surface-{radius}-border-{width}.png')
print('Exported 24 decorative surface masks; no reference files changed.')
