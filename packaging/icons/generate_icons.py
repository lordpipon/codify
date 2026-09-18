#!/usr/bin/env python3
"""Generate Codify's app icon set (Phase 18).

Design — a deep black rounded tile, a crisp white X inside a thin box, and a
gold caret block in the lower-right corner (evoking a close-tab X and a text
cursor in one). No logos, no photos, no trademarks — a plain geometric mark.

Outputs (relative to repo root, under packaging/):

  icons/source.png                        1024 source
  linux/icons/hicolor/<S>/<S>/apps/org.codify.Codify.png   (16..1024)
  windows/codify.ico                      16..256 (multi-size)
  macos/codify.icns                       16..1024 (multi-size)

Run from repo root:  python3 packaging/icons/generate_icons.py
Requires: Pillow.
"""
from __future__ import annotations

import os
from PIL import Image, ImageDraw, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
PKG = os.path.join(ROOT, "packaging")
SS = 4  # supersampling factor for crisp margins

WHITE = (246, 248, 252)
BOX = (226, 230, 238)
GOLD = (244, 174, 62)
RIM = (64, 70, 92)


def tile() -> Image.Image:
    """1024px master: rounded dark tile with a vertical gradient + thin rim."""
    u = 1.0
    big = 1024
    img = Image.new("RGBA", (big, big), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)

    # rounded tile w/ gradient
    rad = int(210 * u)
    grad = Image.new("RGBA", (big, big))
    px = grad.load()
    top, bot = (9, 10, 14), (25, 28, 39)
    for y in range(big):
        t = y / (big - 1)
        c = tuple(int(top[i] + (bot[i] - top[i]) * t) for i in range(3))
        for x in range(big):
            px[x, y] = (c[0], c[1], c[2], 255)
    mask = Image.new("L", (big, big), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, big - 1, big - 1), radius=rad, fill=255)
    img.paste(grad, (0, 0), mask)

    # thin rim
    d.rounded_rectangle(
        (6, 6, big - 7, big - 7),
        radius=rad - 6,
        outline=RIM,
        width=6,
    )
    return img


def icon(image: Image.Image) -> Image.Image:
    """Add the mark: white X in a box + gold caret."""
    big = image.width
    u = big / 1024.0
    d = ImageDraw.Draw(image)
    W = 8 * 22  # line width

    # --- box ---
    b0 = int(178 * u)
    b1 = big - 1 - b0
    d.rounded_rectangle((b0, b0, b1, b1), radius=int(70 * u), outline=BOX, width=int(26 * u))

    # --- X ---
    x0 = int(320 * u)
    x1 = int(704 * u)
    xw = int(120 * u)
    cap = xw // 2
    for (ax, ay, bx, by) in ((x0, x0, x1, x1), (x1, x0, x0, x1)):
        d.line((ax, ay, bx, by), fill=WHITE, width=xw)
        for ex, ey in ((ax, ay), (bx, by)):
            d.ellipse((ex - cap, ey - cap, ex + cap, ey + cap), fill=WHITE)

    # --- gold caret (bottom-right inside the box) ---
    d.rounded_rectangle(
        (int(706 * u), int(760 * u), int(846 * u), int(838 * u)),
        radius=int(22 * u),
        fill=GOLD,
    )
    return image


def _hicolor(img: Image.Image) -> None:
    base = os.path.join(PKG, "linux", "icons", "hicolor")
    for s in (16, 24, 32, 48, 64, 128, 256, 512, 1024):
        d = os.path.join(base, f"{s}x{s}", "apps")
        os.makedirs(d, exist_ok=True)
        img.resize((s, s), Image.LANCZOS).save(
            os.path.join(d, "org.codify.Codify.png"), "PNG"
        )


def main() -> None:
    os.makedirs(os.path.join(PKG, "windows"), exist_ok=True)
    os.makedirs(os.path.join(PKG, "macos"), exist_ok=True)

    master = icon(tile())
    master.save(os.path.join(PKG, "icons", "source.png"), "PNG")
    _hicolor(master)

    ico_sizes = [16, 24, 32, 48, 64, 128, 256]
    master.save(
        os.path.join(PKG, "windows", "codify.ico"),
        "ICO",
        sizes=[(s, s) for s in ico_sizes],
    )
    # large master for the icns so small PNG chunks stay sharp
    big = icon(tile())
    big.save(os.path.join(PKG, "macos", "codify.icns"), "ICNS")
    print("wrote source.png, hicolor pngs, windows/codify.ico, macos/codify.icns")


if __name__ == "__main__":
    main()
