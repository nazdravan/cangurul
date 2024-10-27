#!/usr/bin/env python3
"""Desenează sprite-urile cangurului și logo-ul (SVG → PNG/ICO).

    python3 assets/art/make.py

Scrie assets/art/*.svg, assets/sprites/*.png, assets/logo.png și
assets/cangurul.ico. Cere rsvg-convert (librsvg) și Pillow.
"""
import io
import pathlib
import subprocess

from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
ASSETS = HERE.parent
SPRITE_PX = 192

# Paleta: blană caldă, burtă crem, contur maro închis (se vede pe fundal deschis și închis).
FUR = "#E8863A"
FUR_DARK = "#C9672A"
BELLY = "#FBE3BF"
LINE = "#5A2C10"
EAR = "#F4A9A0"
EYE = "#2B1608"
NOSE = "#3A1C0A"

STYLE = f"""
<style>
  .o {{ stroke: {LINE}; stroke-width: 2.4; stroke-linejoin: round; stroke-linecap: round; }}
  .fur {{ fill: {FUR}; }}
  .dark {{ fill: {FUR_DARK}; }}
  .belly {{ fill: {BELLY}; }}
  .ear {{ fill: {EAR}; }}
  .eye {{ fill: {EYE}; }}
  .nose {{ fill: {NOSE}; }}
  .hi {{ fill: #fff; }}
</style>"""


def svg(body: str, extra_style: str = "") -> str:
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100">'
        f"{STYLE}{extra_style}{body}</svg>\n"
    )


# Profil, cu fața spre dreapta. Picioarele ating y≈89 (acolo e umbra în aplicație).
SIDE = """
  <path class="o dark" d="M40 70 C30 76 17 83 5 90 C18 90 33 87 47 80 Z"/>
  <path class="o fur" d="M38 74 C34 60 40 44 52 40 C62 37 68 46 68 58 C68 70 62 80 52 82 C45 83 40 80 38 74 Z"/>
  <path class="o belly" d="M55 46 C64 48 67 59 64 69 C62 76 57 79 52 79 C55 68 56 57 55 46 Z"/>
  <path class="o fur" d="M36 77 C36 68 44 64 52 66 C58 68 60 75 57 81 L74 83 C79 84 79 90 74 90 L45 90 C39 90 36 84 36 77 Z"/>
  <path class="o" style="fill:none;stroke-width:7.5" d="M58 50 C64 52 69 56 73 61"/>
  <path d="M58 50 C64 52 69 56 73 61" fill="none" stroke="{fur}" stroke-width="3.2" stroke-linecap="round"/>
  <ellipse class="o fur" cx="53" cy="18" rx="4.6" ry="11" transform="rotate(-18 53 18)"/>
  <ellipse class="o fur" cx="60" cy="16" rx="5" ry="12" transform="rotate(8 60 16)"/>
  <ellipse class="ear" cx="60.4" cy="17" rx="2.2" ry="7.5" transform="rotate(8 60 16)"/>
  <path class="o fur" d="M51 33 C51 25 57 21 64 22 C70 23 74 27 79 29 C85 31 87 35 85 38 C83 41 77 41 71 41 C66 42 60 43 56 41 C52 39 51 36 51 33 Z"/>
  <ellipse class="nose" cx="85" cy="33.5" rx="2.6" ry="2.1"/>
  <circle class="eye" cx="66" cy="29" r="3"/>
  <circle class="hi" cx="67" cy="28" r="1.05"/>
  <path d="M74 38.5 C77 39.5 80 39.2 82 38" fill="none" stroke="{line}" stroke-width="1.4" stroke-linecap="round"/>
""".format(fur=FUR, line=LINE)

# Din față (spre sud).
FRONT_BODY = """
  <path class="o dark" d="M66 74 C76 78 84 84 90 90 C80 90 70 87 62 82 Z"/>
  <path class="o fur" d="M32 66 C32 50 40 42 50 42 C60 42 68 50 68 66 C68 78 61 85 50 85 C39 85 32 78 32 66 Z"/>
  <path class="o belly" d="M41 60 C41 52 45 49 50 49 C55 49 59 52 59 60 C59 72 56 80 50 80 C44 80 41 72 41 60 Z"/>
  <path class="o fur" d="M34 89 C30 89 30 84 34 82 C38 80 44 81 46 84 C47 88 44 90 40 90 Z"/>
  <path class="o fur" d="M66 89 C70 89 70 84 66 82 C62 80 56 81 54 84 C53 88 56 90 60 90 Z"/>
  <path class="o" style="fill:none;stroke-width:7" d="M40 54 C40 60 42 63 45 64"/>
  <path class="o" style="fill:none;stroke-width:7" d="M60 54 C60 60 58 63 55 64"/>
  <path d="M40 54 C40 60 42 63 45 64 M60 54 C60 60 58 63 55 64" fill="none" stroke="{fur}" stroke-width="2.8" stroke-linecap="round"/>
""".format(fur=FUR)

FRONT_EARS = """
  <ellipse class="o fur" cx="39" cy="17" rx="5" ry="12" transform="rotate(-16 39 17)"/>
  <ellipse class="ear" cx="39.3" cy="18" rx="2.3" ry="7.6" transform="rotate(-16 39 17)"/>
  <ellipse class="o fur" cx="61" cy="17" rx="5" ry="12" transform="rotate(16 61 17)"/>
  <ellipse class="ear" cx="60.7" cy="18" rx="2.3" ry="7.6" transform="rotate(16 61 17)"/>
"""

FRONT_HEAD = """
  <path class="o fur" d="M36 33 C36 24 42 20 50 20 C58 20 64 24 64 33 C64 41 58 46 50 46 C42 46 36 41 36 33 Z"/>
  <ellipse class="belly" cx="50" cy="38.5" rx="7.5" ry="5.6"/>
  <ellipse class="nose" cx="50" cy="35.6" rx="2.8" ry="2.1"/>
"""

FRONT = FRONT_BODY + FRONT_EARS + FRONT_HEAD + """
  <circle class="eye" cx="44" cy="30" r="2.9"/>
  <circle class="hi" cx="45" cy="29" r="1"/>
  <circle class="eye" cx="56" cy="30" r="2.9"/>
  <circle class="hi" cx="57" cy="29" r="1"/>
  <path d="M47 40.2 C48.6 41.6 51.4 41.6 53 40.2" fill="none" stroke="{line}" stroke-width="1.4" stroke-linecap="round"/>
""".format(line=LINE)

# Din spate (spre nord): fără față, coada coboară pe mijloc.
BACK = """
  <path class="o fur" d="M32 66 C32 50 40 42 50 42 C60 42 68 50 68 66 C68 78 61 85 50 85 C39 85 32 78 32 66 Z"/>
  <path class="o fur" d="M34 89 C30 89 30 84 34 82 C38 80 44 81 46 84 C47 88 44 90 40 90 Z"/>
  <path class="o fur" d="M66 89 C70 89 70 84 66 82 C62 80 56 81 54 84 C53 88 56 90 60 90 Z"/>
  <path class="o dark" d="M44 72 C42 80 46 88 52 95 C57 89 58 80 56 72 C53 68 47 68 44 72 Z"/>
  <ellipse class="o dark" cx="39" cy="17" rx="5" ry="12" transform="rotate(-16 39 17)"/>
  <ellipse class="o dark" cx="61" cy="17" rx="5" ry="12" transform="rotate(16 61 17)"/>
  <path class="o fur" d="M36 33 C36 24 42 20 50 20 C58 20 64 24 64 33 C64 41 58 46 50 46 C42 46 36 41 36 33 Z"/>
"""

# Obosit (după o ciocnire): urechi lăsate, ochi în X, steluțe.
STAR = "M0 -4 L1.2 -1.2 L4 0 L1.2 1.2 L0 4 L-1.2 1.2 L-4 0 L-1.2 -1.2 Z"
TIRED = FRONT_BODY + """
  <ellipse class="o fur" cx="30" cy="28" rx="4.8" ry="11.5" transform="rotate(-62 30 28)"/>
  <ellipse class="ear" cx="30.5" cy="28.3" rx="2.2" ry="7.3" transform="rotate(-62 30 28)"/>
  <ellipse class="o fur" cx="70" cy="28" rx="4.8" ry="11.5" transform="rotate(62 70 28)"/>
  <ellipse class="ear" cx="69.5" cy="28.3" rx="2.2" ry="7.3" transform="rotate(62 70 28)"/>
""" + FRONT_HEAD + """
  <path d="M41.5 27.5 L46.5 32.5 M46.5 27.5 L41.5 32.5 M53.5 27.5 L58.5 32.5 M58.5 27.5 L53.5 32.5"
        fill="none" stroke="{eye}" stroke-width="2" stroke-linecap="round"/>
  <path d="M46.5 41 C48.5 40 51.5 40 53.5 41" fill="none" stroke="{line}" stroke-width="1.4" stroke-linecap="round"/>
  <path d="M50.5 41.2 C50.5 44 53.5 44 53.5 41.6" fill="{ear}" stroke="{line}" stroke-width="1.1"/>
  <g fill="#FFD23F" stroke="{line}" stroke-width="0.9" stroke-linejoin="round">
    <path transform="translate(28 11) scale(1.25)" d="{star}"/>
    <path transform="translate(50 6) scale(1.05)" d="{star}"/>
    <path transform="translate(72 11) scale(1.25)" d="{star}"/>
  </g>
""".format(eye=EYE, line=LINE, ear=EAR, star=STAR)

SPRITES = {
    "east": SIDE,
    "west": f'<g transform="translate(100 0) scale(-1 1)">{SIDE}</g>',
    "south": FRONT,
    "north": BACK,
    "tired": TIRED,
}

# Logo: pătrat rotunjit portocaliu, grilă și un traseu desenat, cangurul alb în salt.
LOGO = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#F7A14B"/>
      <stop offset="1" stop-color="#D9531A"/>
    </linearGradient>
    <clipPath id="sq"><rect x="8" y="8" width="240" height="240" rx="56"/></clipPath>
  </defs>
  <rect x="8" y="8" width="240" height="240" rx="56" fill="url(#bg)"/>
  <g clip-path="url(#sq)" stroke="#fff" stroke-opacity="0.16" stroke-width="3">
    <path d="M48 8V248 M88 8V248 M128 8V248 M168 8V248 M208 8V248 M8 48H248 M8 88H248 M8 128H248 M8 168H248 M8 208H248"/>
  </g>
  <path d="M40 112 V204 H216" fill="none" stroke="#FFE7C2" stroke-width="14" stroke-linecap="round" stroke-linejoin="round"/>
  <g transform="translate(50 57) scale(1.6)">
    {kangaroo}
  </g>
</svg>
"""

# În logo cangurul e o siluetă albă; ochiul rămâne un punct portocaliu.
LOGO_STYLE = """
<style>
  .s path, .s ellipse {{ fill: #fff; stroke: #fff; stroke-width: 2.4; stroke-linejoin: round; }}
  .s .ear, .s .hi {{ display: none; }}
  .s .eye {{ fill: #DE5E1F; stroke: none; }}
  .s path[fill="none"] {{ fill: none; }}
</style>""".format()


def render(svg_path: pathlib.Path, px: int) -> Image.Image:
    png = subprocess.run(
        ["rsvg-convert", "-w", str(px), "-h", str(px), str(svg_path)],
        check=True, capture_output=True,
    ).stdout
    return Image.open(io.BytesIO(png)).convert("RGBA")


def main() -> None:
    (ASSETS / "sprites").mkdir(exist_ok=True)
    for name, body in SPRITES.items():
        path = HERE / f"{name}.svg"
        path.write_text(svg(body))
        render(path, SPRITE_PX).save(ASSETS / "sprites" / f"{name}.png", optimize=True)

    side = SIDE.replace(f'stroke="{FUR}"', 'stroke="#fff"')
    logo = LOGO.format(kangaroo=f'{LOGO_STYLE}<g class="s">{side}</g>')
    (HERE / "logo.svg").write_text(logo)
    render(HERE / "logo.svg", 256).save(ASSETS / "logo.png", optimize=True)
    sizes = [16, 24, 32, 48, 64, 128, 256]
    big = render(HERE / "logo.svg", 256)
    big.save(ASSETS / "cangurul.ico", sizes=[(s, s) for s in sizes])


if __name__ == "__main__":
    main()
