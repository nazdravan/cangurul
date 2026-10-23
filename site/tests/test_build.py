# Testele scriptului care construiește pagina de prezentare:
#   python3 -m unittest discover -s site/tests -v
import json
import sys
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


if __name__ == "__main__":
    unittest.main()
