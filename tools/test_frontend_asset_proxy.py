from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
COS_HOST = "tzyy-1330068502.cos.ap-beijing.myqcloud.com"


class FrontendAssetProxyTests(unittest.TestCase):
    def test_frontend_uses_same_origin_cos_proxy(self):
        checked_paths = [ROOT / "main" / "index.html"]
        checked_paths.extend((ROOT / "main" / "src").rglob("*"))

        offenders = []
        for path in checked_paths:
            if not path.is_file() or path.suffix not in {".html", ".js", ".jsx", ".ts", ".tsx"}:
                continue
            text = path.read_text(encoding="utf-8")
            if COS_HOST in text:
                offenders.append(path.relative_to(ROOT).as_posix())

        self.assertEqual([], offenders, "frontend assets should use /cos/ instead of direct COS URLs")


if __name__ == "__main__":
    unittest.main()
