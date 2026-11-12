from pathlib import Path
import unittest


STORE_PATH = Path(__file__).resolve().parents[1] / "acg-api" / "store.go"


class AcgApiMigrationTests(unittest.TestCase):
    def test_guestbook_parent_column_is_ensured_before_parent_index(self):
        text = STORE_PATH.read_text(encoding="utf-8")

        ensure_pos = text.index('ensureColumn(db, "guestbook_messages", "parent_id"')
        index_pos = text.index("CREATE INDEX IF NOT EXISTS idx_guestbook_messages_parent_created")

        self.assertLess(ensure_pos, index_pos)


if __name__ == "__main__":
    unittest.main()
