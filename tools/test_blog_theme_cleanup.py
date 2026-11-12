import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BLOG = ROOT / "blog"


class BlogThemeCleanupTest(unittest.TestCase):
    def read(self, relative_path):
        return (BLOG / relative_path).read_text(encoding="utf-8")

    def test_author_sidebar_and_nav_cleanup_are_configured(self):
        hexo_config = self.read("_config.yml")
        theme_config = self.read("_config.butterfly.yml")
        card_author = self.read("themes/butterfly/layout/includes/widget/card_author.pug")
        mobile_sidebar = self.read("themes/butterfly/layout/includes/sidebar.pug")
        nav = self.read("themes/butterfly/layout/includes/header/nav.pug")

        self.assertRegex(hexo_config, r"(?m)^author:\s*Taozhiyo\s*$")
        self.assertNotRegex(hexo_config, r"(?m)^author:\s*bistutzyy\s*$")

        self.assertRegex(theme_config, r"(?m)^social:\s*false\s*$")
        self.assertNotIn("mailto:173236231@qq.com", theme_config)
        self.assertNotIn("fab fa-github: https://github.com/bistutzyy", theme_config)

        self.assertRegex(
            theme_config,
            r"card_author:\n(?:.*\n)*?    button:\n      enable:\s*false",
        )
        self.assertNotIn("Follow Me", theme_config)
        self.assertRegex(theme_config, r"card_webinfo:\n    enable:\s*false")
        self.assertRegex(theme_config, r"(?m)^search:\n(?:.*\n)*?  use:\s*$")

        self.assertNotIn(".site-data", card_author)
        self.assertNotIn("card-info-social-icons", card_author)
        self.assertNotIn("theme.aside.card_author.button", card_author)
        self.assertNotIn(".site-data", mobile_sidebar)

        self.assertIn("nav#nav", nav)
        self.assertIn("#menus", nav)
        self.assertIn("if theme.search.use", nav)


if __name__ == "__main__":
    unittest.main()
