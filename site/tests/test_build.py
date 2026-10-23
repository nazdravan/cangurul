# Testele scriptului care construiește pagina de prezentare:
#   python3 -m unittest discover -s site/tests -v
import json
import re
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(ROOT / "site"))
import build  # noqa: E402


class ChangelogTest(unittest.TestCase):
    def test_sections_from_real_changelog(self):
        md = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
        s = build.changelog_sections(md, 2)
        self.assertEqual([x["version"] for x in s], ["3.0.3", "3.0.2"])
        self.assertEqual(s[0]["date"], "2026-10-23")
        self.assertNotIn("[3.0.2]: https://", s[1]["body"])

    def test_fewer_sections_than_requested(self):
        md = "## [1.0.0] — 2026-01-01\n\n- a\n"
        self.assertEqual(len(build.changelog_sections(md, 3)), 1)

    def test_md_list_with_continuation(self):
        html = build.md_to_html("### Adăugat\n\n- Butonul **Formatează**: unu\n  doi\n- trei\n")
        self.assertEqual(
            html,
            "<h4>Adăugat</h4>\n<ul><li>Butonul <strong>Formatează</strong>: unu doi</li><li>trei</li></ul>",
        )

    def test_md_escapes_html_in_code(self):
        self.assertIn("<code>CÎT &lt; 3 &amp;</code>", build.md_to_html("- `CÎT < 3 &`\n"))

    def test_md_link(self):
        self.assertIn(
            '<a href="https://semver.org/">SemVer</a>',
            build.md_to_html("[SemVer](https://semver.org/)"),
        )


RO_MONTHS = ["ianuarie", "februarie", "martie", "aprilie", "mai", "iunie", "iulie",
             "august", "septembrie", "octombrie", "noiembrie", "decembrie"]


def release():
    return json.loads((HERE / "release.json").read_text(encoding="utf-8"))


class ReleaseTest(unittest.TestCase):
    def test_release_assets_maps_all_kinds(self):
        a = build.release_assets(release())
        self.assertEqual(set(a), set(build.ASSET_KINDS))
        self.assertEqual(a["win-x64-msi"]["name"], "cangurul-3.0.3-windows-x64.msi")
        self.assertEqual(a["sha256"]["name"], "SHA256SUMS.txt")

    def test_missing_asset_names_the_file(self):
        r = release()
        r["assets"] = [x for x in r["assets"] if not x["name"].endswith(".dmg")]
        with self.assertRaises(SystemExit) as e:
            build.release_assets(r)
        self.assertIn("macos-universal.dmg", str(e.exception))

    def test_format_size(self):
        self.assertEqual(build.format_size(9213164, "ro"), "8,8 MB")
        self.assertEqual(build.format_size(9213164, "en"), "8.8 MB")
        self.assertEqual(build.format_size(9213164, "ru"), "8,8 МБ")

    def test_format_date(self):
        self.assertEqual(build.format_date("2026-10-23T12:20:00Z", RO_MONTHS, "{d} {month} {y}"), "23 octombrie 2026")
        self.assertEqual(build.format_date("2026-10-03", RO_MONTHS, "{month} {d}, {y}"), "octombrie 3, 2026")


class BuildTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.out = Path(tempfile.mkdtemp())
        build.build(release(), cls.out)

    def page(self, p):
        return (self.out / p).read_text(encoding="utf-8")

    def test_three_pages_with_lang(self):
        for path, lang in [("index.html", "ro"), ("en/index.html", "en"), ("ru/index.html", "ru")]:
            html = self.page(path)
            self.assertIn(f'<html lang="{lang}"', html)
            self.assertNotIn("{{", html)
            for h in ("ro", "en", "ru"):
                self.assertIn(f'hreflang="{h}"', html)

    def test_same_keys_in_all_languages(self):
        keys = [set(json.loads((ROOT / "site/i18n" / f"{lang}.json").read_text(encoding="utf-8")))
                for lang in ("ro", "en", "ru")]
        self.assertEqual(keys[0], keys[1])
        self.assertEqual(keys[0], keys[2])

    def test_main_button_works_without_js(self):
        self.assertRegex(self.page("index.html"), r'id="download-main"[^>]*href="#descarca"')

    def test_all_assets_linked(self):
        html = self.page("index.html")
        for name in ("cangurul-3.0.3-windows-x64.msi", "cangurul-3.0.3-windows7-x86-portable.zip",
                     "cangurul-3.0.3-macos-universal.dmg", "SHA256SUMS.txt"):
            self.assertIn(f"/download/v3.0.3/{name}", html)
        self.assertIn("4,2 MB", html)  # .msi x64 din release.json

    def test_news_from_changelog_of_each_language(self):
        self.assertIn("Instalatorul <code>.msi</code> pentru Windows", self.page("index.html"))
        self.assertIn("Windows <code>.msi</code> installer", self.page("en/index.html"))
        self.assertIn("23 октября 2026", self.page("ru/index.html"))

    def test_anchors_and_author(self):
        for p in ("index.html", "en/index.html", "ru/index.html"):
            html = self.page(p)
            for a in ("ce-face", "comparatie", "noutati", "descarca"):
                self.assertIn(f'id="{a}"', html)
            self.assertIn("Nicolae Postica", html)
            self.assertIn('href="https://github.com/nazdravan"', html)

    def test_home_links_stay_in_language(self):
        self.assertIn('class="brand" href="./"', self.page("en/index.html"))
        self.assertIn('href="../" hreflang="ro"', self.page("en/index.html"))
        self.assertIn('href="./" hreflang="ro"', self.page("index.html"))

    def test_relative_assets(self):
        self.assertIn('href="assets/style.css"', self.page("index.html"))
        self.assertIn('href="../assets/style.css"', self.page("en/index.html"))
        for f in ("style.css", "main.js", "logo.png", "screenshot.png", "analizor.png", "og.png"):
            self.assertTrue((self.out / "assets" / f).exists(), f)
        self.assertTrue((self.out / ".nojekyll").exists())

    def test_news_skip_versions_not_released_yet(self):
        md = "## [3.0.4] — 2026-11-01\n\n- viitor\n\n## [3.0.3] — 2026-10-23\n\n- acum\n"
        t = {"months": RO_MONTHS, "date_format": "{d} {month} {y}"}
        html = build.news_html(md, t, "3.0.3")
        self.assertNotIn("viitor", html)
        self.assertIn("acum", html)

    def test_russian_says_besides_windows(self):
        self.assertIn("помимо Windows", self.page("ru/index.html"))

    def test_win7_download_offered_to_button(self):
        self.assertIn("windows7-x86-portable.zip", re.search(r'data-kinds="([^"]*)"', self.page("index.html"))[1])
        self.assertIn('data-label-win7="', self.page("index.html"))

    def test_render_missing_key(self):
        with self.assertRaises(KeyError):
            build.render("{{ nu_exista }}", {})

    def test_render_escapes_double_braces_only(self):
        self.assertEqual(build.render("{{ a }}|{{{ a }}}", {"a": "<b>"}), "&lt;b&gt;|<b>")


if __name__ == "__main__":
    unittest.main()
