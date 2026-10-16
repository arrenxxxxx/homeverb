"""Rasterize the existing Window Light home glyph for platform launch icons."""
from pathlib import Path
from PIL import Image, ImageDraw

root = Path(__file__).resolve().parents[1]
targets = list((root / 'android/app/src/main/res').glob('mipmap-*/ic_launcher.png'))
targets += list((root / 'ios/Runner/Assets.xcassets/AppIcon.appiconset').glob('*.png'))
targets += list((root / 'web/icons').glob('*.png'))
targets += [root / 'web/favicon.png']
for path in targets:
    size = Image.open(path).size
    image = Image.new('RGB', (size[0] * 4, size[1] * 4), '#f6f2ea')
    draw = ImageDraw.Draw(image)
    unit = image.width / 36
    points = [(3, 10), (12, 2), (21, 10), (21, 21), (15, 21),
              (15, 14), (9, 14), (9, 21), (3, 21), (3, 10)]
    draw.line([((x + 6) * unit, (y + 6) * unit) for x, y in points],
              fill='#b94e29', width=max(1, round(1.7 * unit)), joint='curve')
    image.resize(size, Image.Resampling.LANCZOS).save(path)
print(f'Generated {len(targets)} HomeVerb icons.')
