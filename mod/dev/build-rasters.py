"""把阿宇立繪轉成 Raster 用的半格字元格，給不支援 kitty 圖形協定的終端機顯示。

低解析度下用 CGA 四色（黑、白、洋紅、青）的硬邊風格：裁成頭部正方形、外圍描一圈青色，
背景留空露出終端機底色。

每個寬度輸出一個 assets/raster-<欄數>.json：{ "rows": 列數, "cells": { 狀態: base64 } }。
cells 的格式照 Claude Code 的 RasterProps：每格三個 little-endian u32
[字元, 前景色, 背景色]，顏色 0x00RRGGBB，0x01000000 代表終端機預設色。

用法：python build-rasters.py [立繪資料夾]
立繪資料夾預設是 ../assets，裡面要有 idle／thinking／working／asking／error／complete 六張 PNG。
換成自己的角色時，放進六張 PNG，再照構圖調整 HEAD_BOXES。
"""
import base64
import json
import pathlib
import struct
import sys

from PIL import Image, ImageEnhance, ImageFilter

ASSETS = pathlib.Path(__file__).resolve().parent.parent / "assets"
STATES = ("idle", "thinking", "working", "asking", "error", "complete")
WIDTHS = (24, 30, 36, 42, 48, 56)
CGA = [(0, 0, 0), (255, 255, 255), (255, 85, 255), (85, 255, 255)]
OUTLINE = (85, 255, 255)
CONTRAST = 1.6
# 各張立繪構圖不同，頭像的正方形框（left, top, right, bottom）逐張訂：
# 寧可切掉頭頂的頭髮，也要留住下巴。座標以寬 768 的立繪為準，其他寬度的圖會等比例換算。
BOX_REFERENCE_WIDTH = 768
HEAD_BOXES = {
    "idle": (130, 140, 730, 740),
    "thinking": (150, 180, 750, 780),
    "working": (104, 120, 664, 680),
    "asking": (0, 150, 680, 830),
    "error": (88, 150, 728, 790),
    "complete": (85, 200, 725, 840),
}
UPPER_HALF, LOWER_HALF, SPACE = 0x2580, 0x2584, 0x20
DEFAULT_COLOR = 0x01000000
OPAQUE = 128


def cga_palette():
    palette = Image.new("P", (1, 1))
    palette.putpalette([c for color in CGA for c in color] + [0] * (768 - 3 * len(CGA)))
    return palette


def cga_pixels(path, size, box):
    """回傳 size×size 的像素表，每格是 RGB tuple 或 None（露出終端機底色）。"""
    portrait = Image.open(path).convert("RGBA")
    scale = portrait.width / BOX_REFERENCE_WIDTH
    head = portrait.crop(tuple(round(v * scale) for v in box)).resize((size, size), Image.LANCZOS)
    shown = head.getchannel("A").point(lambda a: 255 if a >= OPAQUE else 0)
    outline = shown.filter(ImageFilter.MaxFilter(3))
    opaque = Image.alpha_composite(Image.new("RGBA", head.size, (0, 0, 0, 255)), head).convert("RGB")
    flat = ImageEnhance.Contrast(opaque).enhance(CONTRAST)
    flat = flat.quantize(palette=cga_palette(), dither=Image.Dither.NONE).convert("RGB")
    return [
        [flat.getpixel((x, y)) if shown.getpixel((x, y)) else OUTLINE if outline.getpixel((x, y)) else None
         for x in range(size)]
        for y in range(size)
    ]


def rgb(color):
    r, g, b = color
    return (r << 16) | (g << 8) | b


def cell(top, bottom):
    """上下兩個像素合成一格；沒有顏色的半格露出終端機背景。"""
    if top and bottom:
        return UPPER_HALF, rgb(top), rgb(bottom)
    if top:
        return UPPER_HALF, rgb(top), DEFAULT_COLOR
    if bottom:
        return LOWER_HALF, rgb(bottom), DEFAULT_COLOR
    return SPACE, DEFAULT_COLOR, DEFAULT_COLOR


def encode(path, columns, rows, box):
    pixels = cga_pixels(path, columns, box)
    packed = bytearray()
    for y in range(rows):
        for x in range(columns):
            packed += struct.pack("<3I", *cell(pixels[2 * y][x], pixels[2 * y + 1][x]))
    return base64.b64encode(bytes(packed)).decode("ascii")


def main():
    portraits = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else ASSETS
    for columns in WIDTHS:
        # 一格寬 1 高 2，用半格字元湊成方形像素，所以正方形頭像的列數是欄數的一半。
        rows = columns // 2
        cells = {
            state: encode(portraits / f"{state}.png", columns, rows, HEAD_BOXES[state]) for state in STATES
        }
        out = ASSETS / f"raster-{columns}.json"
        out.write_text(json.dumps({"rows": rows, "cells": cells}), encoding="utf-8")
        print(f"{out.name}: {columns}×{rows}, {out.stat().st_size // 1024} KB")


if __name__ == "__main__":
    main()
