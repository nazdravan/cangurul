#!/usr/bin/env python3
"""Pixel-art text banner with a stepped 3D shadow (README headers, logos).

The text is drawn pixel by pixel on a small canvas (colour bands, shine,
outline, stepped shadow) and then upscaled with nearest-neighbour, so the
pixels stay square and crisp.

Usage:
  banner.py TEXT -o out.png [--color #f97316] [--bg #141211]
            [--width 2430] [--height 420] [--shadow 5]
            [--opaque [--no-dots]]

The background is transparent by default, so the banner fits both light and
dark pages; --opaque fills it with --bg (and a faint dot grid).

Letters A-Z, digits 0-9, space and - . ! ? : / & + are supported;
diacritics are dropped (Ă -> A) and the text is upper-cased.
"""
import argparse
import sys
import unicodedata

from PIL import Image

CELL = 7  # one "#" in GLYPHS = CELL x CELL drawn pixels
GAP = 8  # drawn pixels between letters

GLYPHS = {
    "A": [".####.", "##..##", "##..##", "######", "##..##", "##..##", "##..##"],
    "B": ["#####.", "##..##", "##..##", "#####.", "##..##", "##..##", "#####."],
    "C": [".#####", "##....", "##....", "##....", "##....", "##....", ".#####"],
    "D": ["#####.", "##..##", "##..##", "##..##", "##..##", "##..##", "#####."],
    "E": ["######", "##....", "##....", "#####.", "##....", "##....", "######"],
    "F": ["######", "##....", "##....", "#####.", "##....", "##....", "##...."],
    "G": [".#####", "##....", "##....", "##.###", "##..##", "##..##", ".####."],
    "H": ["##..##", "##..##", "##..##", "######", "##..##", "##..##", "##..##"],
    "I": ["######", "..##..", "..##..", "..##..", "..##..", "..##..", "######"],
    "J": ["....##", "....##", "....##", "....##", "##..##", "##..##", ".####."],
    "K": ["##..##", "##.##.", "####..", "###...", "####..", "##.##.", "##..##"],
    "L": ["##....", "##....", "##....", "##....", "##....", "##....", "######"],
    "M": ["##...##", "###.###", "#######", "##.#.##", "##...##", "##...##", "##...##"],
    "N": ["##..##", "###.##", "######", "##.###", "##..##", "##..##", "##..##"],
    "O": [".####.", "##..##", "##..##", "##..##", "##..##", "##..##", ".####."],
    "P": ["#####.", "##..##", "##..##", "#####.", "##....", "##....", "##...."],
    "Q": [".####.", "##..##", "##..##", "##..##", "##.###", "##.##.", ".##.##"],
    "R": ["#####.", "##..##", "##..##", "#####.", "##.##.", "##..##", "##..##"],
    "S": [".#####", "##....", "##....", ".####.", "....##", "....##", "#####."],
    "T": ["######", "..##..", "..##..", "..##..", "..##..", "..##..", "..##.."],
    "U": ["##..##", "##..##", "##..##", "##..##", "##..##", "##..##", ".####."],
    "V": ["##..##", "##..##", "##..##", "##..##", "##..##", ".####.", "..##.."],
    "W": ["##...##", "##...##", "##...##", "##.#.##", "#######", "###.###", "##...##"],
    "X": ["##..##", "##..##", ".####.", "..##..", ".####.", "##..##", "##..##"],
    "Y": ["##..##", "##..##", ".####.", "..##..", "..##..", "..##..", "..##.."],
    "Z": ["######", "....##", "...##.", "..##..", ".##...", "##....", "######"],
    "0": [".####.", "##..##", "##.###", "###.##", "##..##", "##..##", ".####."],
    "1": ["..##..", ".###..", "..##..", "..##..", "..##..", "..##..", "######"],
    "2": [".####.", "##..##", "....##", "...##.", "..##..", ".##...", "######"],
    "3": ["#####.", "....##", "....##", ".####.", "....##", "....##", "#####."],
    "4": ["##..##", "##..##", "##..##", "######", "....##", "....##", "....##"],
    "5": ["######", "##....", "##....", "#####.", "....##", "....##", "#####."],
    "6": [".####.", "##....", "##....", "#####.", "##..##", "##..##", ".####."],
    "7": ["######", "....##", "...##.", "..##..", "..##..", "..##..", "..##.."],
    "8": [".####.", "##..##", "##..##", ".####.", "##..##", "##..##", ".####."],
    "9": [".####.", "##..##", "##..##", ".#####", "....##", "....##", ".####."],
    " ": ["...", "...", "...", "...", "...", "...", "..."],
    "-": ["....", "....", "....", "####", "....", "....", "...."],
    "+": ["......", "..##..", "..##..", "######", "..##..", "..##..", "......"],
    ".": ["..", "..", "..", "..", "..", "##", "##"],
    ":": ["..", "##", "##", "..", "..", "##", "##"],
    "!": ["##", "##", "##", "##", "##", "..", "##"],
    "?": [".####.", "##..##", "....##", "...##.", "..##..", "......", "..##.."],
    "/": ["....##", "....##", "...##.", "..##..", ".##...", "##....", "##...."],
    "&": [".###..", "##.##.", ".###..", "####.#", "##.##.", "##.##.", ".##.##"],
}


def hex_rgb(s):
    s = s.lstrip("#")
    if len(s) != 6:
        raise argparse.ArgumentTypeError(f"not a #rrggbb colour: {s}")
    return tuple(int(s[i:i + 2], 16) for i in (0, 2, 4))


def mix(a, b, t):
    return tuple(round(a[i] + (b[i] - a[i]) * t) for i in range(3))


def palette(c):
    white, black = (255, 255, 255), (0, 0, 0)
    return {
        # Letter bands, top to bottom: from a pale tint to the colour and darker.
        "bands": [mix(white, c, t) for t in (0.15, 0.35, 0.55, 0.8)] + [c, mix(c, black, 0.08), mix(c, black, 0.22)],
        "shine": mix(white, c, 0.05),
        "shade": [mix(c, black, t) for t in (0.45, 0.57, 0.67, 0.75, 0.82, 0.86, 0.89, 0.92)],
        "outline": mix(c, black, 0.85),
    }


def normalize(text):
    out = []
    for ch in unicodedata.normalize("NFD", text.upper()):
        if unicodedata.combining(ch):
            continue
        if ch not in GLYPHS:
            sys.exit(f"unsupported character: {ch!r}")
        out.append(ch)
    return "".join(out)


def text_pixels(text):
    """Text pixels, with each outer corner cut by one pixel."""
    px = set()
    x0 = 0
    for c in text:
        g = GLYPHS[c]
        for r, line in enumerate(g):
            for k, ch in enumerate(line):
                if ch == "#":
                    px |= {(x0 + k * CELL + i, r * CELL + j) for i in range(CELL) for j in range(CELL)}
        x0 += len(g[0]) * CELL + GAP
    corners = {(x, y) for x, y in px
               if any((x + dx, y) not in px and (x, y + dy) not in px
                      for dx in (-1, 1) for dy in (-1, 1))}
    return px - corners, x0 - GAP, 7 * CELL


def render(text, color, bg, width, height, depth, dots, transparent):
    pal = palette(color)
    letters, tw, th = text_pixels(normalize(text))
    # Largest integer scale that leaves a margin of about one letter cell.
    scale = max(1, min(width // (tw + depth + 2 * CELL), height // (th + depth + 2 * CELL)))
    w, h = width // scale, height // scale
    ox, oy = (w - tw - depth) // 2, (h - th - depth) // 2
    letters = {(x + ox, y + oy) for x, y in letters}
    around = lambda s: {(x + dx, y + dy) for x, y in s for dx in (-1, 0, 1) for dy in (-1, 0, 1)} - s

    img = Image.new("RGBA", (w, h), (0, 0, 0, 0) if transparent else bg + (255,))
    put = lambda p, c: 0 <= p[0] < w and 0 <= p[1] < h and img.putpixel(p, c + (255,))
    if dots and not transparent:
        for x in range(0, w, 3):
            for y in range(1, h, 3):
                put((x, y), mix(bg, (255, 255, 255), 0.05))

    # Stepped shadow: the text pushed diagonally, darker with every step.
    shade = pal["shade"]
    for k in range(depth, 0, -1):
        for x, y in letters:
            put((x + k, y + k), shade[min(k - 1, len(shade) - 1)])
    shadowed = {(x + k, y + k) for x, y in letters for k in range(depth + 1)}
    for p in around(shadowed):
        put(p, pal["outline"])

    # The letters: horizontal bands, shine on the top and left edges of the upper half.
    bands = pal["bands"]
    for x, y in letters:
        put((x, y), bands[(y - oy) * len(bands) // th])
    for x, y in letters:
        if ((x, y - 1) not in letters or (x - 1, y) not in letters) and (y - oy) < th // 2:
            put((x, y), pal["shine"])
    for p in around(letters) - shadowed:
        put(p, pal["outline"])

    big = img.resize((w * scale, h * scale), Image.NEAREST)
    final = Image.new("RGBA", (width, height), (0, 0, 0, 0) if transparent else bg + (255,))
    final.paste(big, ((width - big.width) // 2, (height - big.height) // 2))
    return final if transparent else final.convert("RGB")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("text")
    ap.add_argument("-o", "--out", required=True)
    ap.add_argument("--color", type=hex_rgb, default="#f97316", help="main letter colour")
    ap.add_argument("--bg", type=hex_rgb, default="#141211", help="background colour (only with --opaque)")
    ap.add_argument("--width", type=int, default=2430)
    ap.add_argument("--height", type=int, default=420)
    ap.add_argument("--shadow", type=int, default=5, help="shadow depth in drawn pixels (0 = none)")
    ap.add_argument("--opaque", action="store_true", help="fill the background (default: transparent)")
    ap.add_argument("--no-dots", action="store_true", help="with --opaque: plain background without the dot grid")
    a = ap.parse_args()
    img = render(a.text, a.color, a.bg, a.width, a.height, a.shadow, not a.no_dots, not a.opaque)
    img.save(a.out, optimize=True)
    print(f"{a.out} ({img.width}x{img.height})")


if __name__ == "__main__":
    main()
