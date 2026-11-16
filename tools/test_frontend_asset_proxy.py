from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
R2_HOST = "fs.964643.xyz"
LEGACY_COS_ORIGIN = "https://tzyy-1330068502"
# 旧 COS 域名只允许出现在"把旧 URL 改写成 R2"的兼容层和它的测试输入夹具里。
LEGACY_ALLOWED = {
    "main/src/pages/aboutPreviewAssets.js",
    "main/src/pages/aboutPreviewAssets.test.js",
}


class FrontendAssetProxyTests(unittest.TestCase):
    def test_frontend_uses_r2_public_base(self):
        checked_paths = [ROOT / "main" / "index.html"]
        checked_paths.extend((ROOT / "main" / "src").rglob("*"))

        offenders = []
        saw_r2 = False
        for path in checked_paths:
            if not path.is_file() or path.suffix not in {".html", ".js", ".jsx", ".ts", ".tsx"}:
                continue
            rel = path.relative_to(ROOT).as_posix()
            text = path.read_text(encoding="utf-8")
            if R2_HOST in text:
                saw_r2 = True
            if LEGACY_COS_ORIGIN in text and rel not in LEGACY_ALLOWED:
                offenders.append(rel)

        self.assertEqual(
            [],
            offenders,
            "frontend must not link the retired COS bucket; use https://fs.964643.xyz",
        )
        self.assertTrue(saw_r2, "frontend should reference the R2 public base")


if __name__ == "__main__":
    unittest.main()
