-- Guestbook messages (留言小纸条)
CREATE TABLE IF NOT EXISTS guestbook_messages (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id INTEGER,
  nickname TEXT NOT NULL,
  avatar TEXT NOT NULL DEFAULT '',
  content TEXT NOT NULL,
  contact_email TEXT NOT NULL DEFAULT '',
  content_hash TEXT NOT NULL DEFAULT '',
  ip_hash TEXT NOT NULL,
  ip_region TEXT NOT NULL DEFAULT '',
  ip_masked TEXT NOT NULL DEFAULT '',
  user_agent_hash TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'visible',
  is_login_user INTEGER NOT NULL DEFAULT 0,
  is_admin_user INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_guestbook_messages_status_created ON guestbook_messages(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_guestbook_messages_user_created ON guestbook_messages(user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS guestbook_ip_cache (
  ip_hash TEXT PRIMARY KEY,
  ip_region TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
