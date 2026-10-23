# Testele scriptului care construiește pagina de prezentare:
#   python3 -m unittest discover -s site/tests -v
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


if __name__ == "__main__":
    unittest.main()
