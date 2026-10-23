#!/usr/bin/env python3
# Construiește pagina de prezentare (GitHub Pages) din site/, din descrierea
# ultimului release și din CHANGELOG*.md. Doar biblioteca standard.
import html
import re

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
