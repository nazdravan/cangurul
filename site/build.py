#!/usr/bin/env python3
# Construiește pagina de prezentare (GitHub Pages) din site/, din descrierea
# ultimului release și din CHANGELOG*.md. Doar biblioteca standard.
import argparse
import html
import json
import re
import shutil
from pathlib import Path

SECTION = re.compile(r"^## \[(?P<version>[^\]]+)\] — (?P<date>\d{4}-\d{2}-\d{2})$", re.M)
# Liniile de referință de la sfârșitul jurnalului: [3.0.3]: https://…
REFERENCE = re.compile(r"^\[[^\]]+\]: \S+$", re.M)


def changelog_sections(md, count):
    """Primele `count` versiuni din jurnal: version, date, body (Markdown)."""
    matches = list(SECTION.finditer(md))[:count]
    sections = []
    for i, m in enumerate(matches):
        end = matches[i + 1].start() if i + 1 < len(matches) else len(md)
        body = md[m.end():end]
        # Ultima secțiune se termină înaintea următorului titlu sau a referințelor.
        cut = re.search(r"^## |^\[[^\]]+\]: \S+$", body, re.M)
        if cut:
            body = body[:cut.start()]
        sections.append({"version": m["version"], "date": m["date"], "body": body.strip()})
    return sections


# Fișierele unui release: cheia folosită în pagină -> sfârșitul numelui,
# după `cangurul-<versiune>-` (SHA256SUMS.txt are numele întreg).
ASSET_KINDS = {
    "win-x64-msi": "windows-x64.msi",
    "win-x86-msi": "windows-x86.msi",
    "win-x64-zip": "windows-x64-portable.zip",
    "win-x86-zip": "windows-x86-portable.zip",
    "win7-zip": "windows7-x86-portable.zip",
    "linux-appimage": "linux-x86_64.AppImage",
    "linux-deb": "linux-x86_64.deb",
    "linux-tar": "linux-x86_64.tar.gz",
    "mac-dmg": "macos-universal.dmg",
    "sha256": "SHA256SUMS.txt",
}


def release_assets(release):
    """Cheie -> {name, url, size}. Un fișier lipsă oprește build-ul: pagina
    veche, cu linkuri bune, rămâne publicată."""
    version = release["tagName"].removeprefix("v")
    by_name = {a["name"]: a for a in release["assets"]}
    assets = {}
    for kind, suffix in ASSET_KINDS.items():
        name = suffix if kind == "sha256" else f"cangurul-{version}-{suffix}"
        if name not in by_name:
            raise SystemExit(f"Release-ul {release['tagName']} nu are fișierul {name}")
        a = by_name[name]
        assets[kind] = {"name": name, "url": a["url"], "size": a["size"]}
    return assets


def format_size(n, lang):
    mb = f"{n / 1024 / 1024:.1f}"
    if lang == "en":
        return f"{mb} MB"
    return mb.replace(".", ",") + (" МБ" if lang == "ru" else " MB")


def format_date(iso, months, pattern):
    y, m, d = (int(x) for x in iso[:10].split("-"))
    return pattern.format(d=d, month=months[m - 1], y=y)


def inline(text):
    """`cod`, **bold** și [text](url) pe un text deja escapat."""
    parts = re.split(r"(`[^`]*`)", text)
    out = []
    for part in parts:
        if part.startswith("`") and part.endswith("`") and len(part) > 1:
            out.append(f"<code>{html.escape(part[1:-1], quote=False)}</code>")
            continue
        part = html.escape(part, quote=False)
        part = re.sub(r"\*\*(.+?)\*\*", r"<strong>\1</strong>", part)
        part = re.sub(
            r"\[([^\]]+)\]\(([^)\s]+)\)",
            lambda m: f'<a href="{html.escape(m[2])}">{m[1]}</a>',
            part,
        )
        out.append(part)
    return "".join(out)


def md_to_html(md):
    """Subsetul de Markdown din jurnal: ###, liste, paragrafe, cod, bold, linkuri."""
    blocks = []
    items = None
    para = []

    def flush_para():
        if para:
            blocks.append(f"<p>{inline(' '.join(para))}</p>")
            para.clear()

    def flush_list():
        nonlocal items
        if items is not None:
            blocks.append("<ul>" + "".join(f"<li>{inline(i)}</li>" for i in items) + "</ul>")
            items = None

    for line in md.splitlines():
        if line.startswith("### "):
            flush_para()
            flush_list()
            blocks.append(f"<h4>{inline(line[4:].strip())}</h4>")
        elif line.startswith("- "):
            flush_para()
            if items is None:
                items = []
            items.append(line[2:].strip())
        elif line.startswith("  ") and items:
            items[-1] += " " + line.strip()
        elif not line.strip():
            flush_para()
            flush_list()
        else:
            flush_list()
            para.append(line.strip())
    flush_para()
    flush_list()
    return "\n".join(blocks)


ROOT = Path(__file__).resolve().parent.parent
SITE = ROOT / "site"
LANGS = {"ro": "", "en": "en/", "ru": "ru/"}
NEWS_COUNT = 3
PLACEHOLDER = re.compile(r"\{\{\{\s*(\w+)\s*\}\}\}|\{\{\s*(\w+)\s*\}\}")
IMAGES = {
    "logo.png": ROOT / "assets/logo.png",
    "screenshot.png": ROOT / "docs/assets/screenshot.png",
    "analizor.png": ROOT / "docs/assets/analizor.png",
    "og.png": ROOT / "docs/assets/cangurul-banner.png",
    "kangaroo.svg": ROOT / "assets/art/east.svg",
    "kangaroo-tired.svg": ROOT / "assets/art/tired.svg",
}


def render(template, values):
    """{{{ cheie }}} = HTML brut, {{ cheie }} = text escapat; o cheie lipsă e o eroare."""
    def sub(m):
        raw, escaped = m[1], m[2]
        value = values[raw or escaped]
        return value if raw else html.escape(value)
    return PLACEHOLDER.sub(sub, template)


def news_html(md, t):
    out = []
    for s in changelog_sections(md, NEWS_COUNT):
        date = format_date(s["date"], t["months"], t["date_format"])
        out.append(
            f'<article class="release" data-reveal><div class="release-head"><h3>{html.escape(s["version"])}</h3>'
            f'<time datetime="{s["date"]}">{date}</time></div>\n'
            f'<div class="release-body">{md_to_html(s["body"])}</div></article>'
        )
    return "\n".join(out)


def build(release, out):
    assets = release_assets(release)
    version = release["tagName"].removeprefix("v")
    template = (SITE / "template.html").read_text(encoding="utf-8")
    kinds = json.dumps({
        "windows": assets["win-x64-msi"]["url"],
        "mac": assets["mac-dmg"]["url"],
        "linux": assets["linux-appimage"]["url"],
    })
    for lang, path in LANGS.items():
        t = json.loads((SITE / "i18n" / f"{lang}.json").read_text(encoding="utf-8"))
        values = {k: v for k, v in t.items() if isinstance(v, str)}
        values.update({
            "root": "../" if path else "",
            "path": path,
            "home_ro": "../" if path else "./",
            "version": version,
            "date": format_date(release["publishedAt"], t["months"], t["date_format"]),
            "kinds_json": kinds,
            # Mărimea titlului se calculează din rândul lui cel mai lung (style.css).
            "h1_chars": str(max(len(t["hero_title_1"]), len(t["hero_title_2"]) + 2)),
            "news_html": news_html((ROOT / t["changelog_file"]).read_text(encoding="utf-8"), t),
        })
        for code in LANGS:
            values[f"current_{code}"] = 'aria-current="page"' if code == lang else ""
        for kind, a in assets.items():
            key = kind.replace("-", "_")
            values[f"url_{key}"] = a["url"]
            values[f"name_{key}"] = a["name"]
            values[f"size_{key}"] = format_size(a["size"], lang)
        page = out / path / "index.html"
        page.parent.mkdir(parents=True, exist_ok=True)
        page.write_text(render(template, values), encoding="utf-8")

    (out / "assets").mkdir(parents=True, exist_ok=True)
    for f in (SITE / "assets").iterdir():
        shutil.copy2(f, out / "assets" / f.name)
    for name, src in IMAGES.items():
        shutil.copy2(src, out / "assets" / name)
    (out / ".nojekyll").write_text("")


def main():
    p = argparse.ArgumentParser(description="Construiește pagina de prezentare.")
    p.add_argument("--release", required=True, help="JSON de la gh release view --json tagName,publishedAt,url,assets")
    p.add_argument("--out", required=True, help="dosarul în care se scrie pagina")
    args = p.parse_args()
    release = json.loads(Path(args.release).read_text(encoding="utf-8"))
    out = Path(args.out)
    if out.exists():
        shutil.rmtree(out)
    build(release, out)
    print(f"Gata: {out}")


if __name__ == "__main__":
    main()
