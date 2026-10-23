"""把 assets/ 的立繪 PNG 包成內嵌 WebP 的 SVG，給沒有 Image 元件的桌面版等遠端畫面用。

Svg 元件的原始碼上限是 131072 字元，所以從高畫質往下試，挑第一個塞得進去的品質。
執行：python dev/build-svgs.py（在 mod/ 底下）
"""
import base64
import io
from pathlib import Path

from PIL import Image

ASSETS = Path(__file__).resolve().parent.parent / 'assets'
STATES = ['idle', 'thinking', 'working', 'asking', 'complete', 'error']
MAX_CHARS = 120_000


def build_svg(image, quality):
    buffer = io.BytesIO()
    image.save(buffer, 'WEBP', quality=quality, method=6)
    data = base64.b64encode(buffer.getvalue()).decode()
    w, h = image.size
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">'
            f'<image width="{w}" height="{h}" href="data:image/webp;base64,{data}"/></svg>')


def main():
    (ASSETS / 'svg').mkdir(exist_ok=True)
    for state in STATES:
        image = Image.open(ASSETS / f'{state}.png')
        for quality in (90, 85, 80, 70, 60):
            svg = build_svg(image, quality)
            if len(svg) <= MAX_CHARS:
                break
        else:
            raise SystemExit(f'{state}.png 壓到品質 60 還是超過 {MAX_CHARS} 字元')
        (ASSETS / 'svg' / f'{state}.svg').write_text(svg, encoding='utf-8')
        print(f'{state}: quality {quality}, {len(svg)} chars')


if __name__ == '__main__':
    main()
