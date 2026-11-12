CREATE TABLE IF NOT EXISTS users (
  id BIGSERIAL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL DEFAULT '',
  password_hash TEXT NOT NULL,
  avatar TEXT NOT NULL DEFAULT '',
  is_owner BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  unlimited BOOLEAN NOT NULL DEFAULT FALSE,
  expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS guestbook_messages (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
  nickname TEXT NOT NULL,
  avatar TEXT NOT NULL DEFAULT '',
  channel TEXT NOT NULL DEFAULT 'guestbook',
  content TEXT NOT NULL,
  contact_email TEXT NOT NULL DEFAULT '',
  ip_hash TEXT NOT NULL DEFAULT '',
  ip_region TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'visible',
  is_login_user BOOLEAN NOT NULL DEFAULT FALSE,
  is_admin_user BOOLEAN NOT NULL DEFAULT FALSE,
  parent_id BIGINT NOT NULL DEFAULT 0,
  owner_read_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_guestbook_channel_created
  ON guestbook_messages (channel, created_at DESC);

CREATE TABLE IF NOT EXISTS wallpapers (
  id TEXT PRIMARY KEY,
  url TEXT NOT NULL,
  preview_url TEXT NOT NULL DEFAULT '',
  source TEXT NOT NULL DEFAULT '',
  author TEXT NOT NULL DEFAULT '',
  source_url TEXT NOT NULL DEFAULT '',
  license_note TEXT NOT NULL DEFAULT '',
  kind TEXT NOT NULL DEFAULT 'pool',
  status TEXT NOT NULL DEFAULT 'active',
  added_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS bangumi_items (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  watched INTEGER NOT NULL DEFAULT 0,
  total INTEGER NOT NULL DEFAULT 0,
  latest_episode INTEGER NOT NULL DEFAULT 0,
  cover_path TEXT NOT NULL DEFAULT '',
  link_url TEXT NOT NULL DEFAULT '',
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS radar_feed (
  id TEXT PRIMARY KEY,
  up_uid TEXT NOT NULL,
  creator_name TEXT NOT NULL,
  latest_text TEXT NOT NULL,
  cover_path TEXT NOT NULL DEFAULT '',
  link_url TEXT NOT NULL DEFAULT '',
  is_new BOOLEAN NOT NULL DEFAULT FALSE,
  published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS bilibili_videos (
  bvid TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  cover TEXT NOT NULL DEFAULT '',
  published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  duration TEXT NOT NULL DEFAULT '',
  synced_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ai_chat_quota (
  identity_key TEXT NOT NULL,
  quota_date DATE NOT NULL,
  used INTEGER NOT NULL DEFAULT 0,
  last_request_at TIMESTAMPTZ,
  PRIMARY KEY (identity_key, quota_date)
);

CREATE TABLE IF NOT EXISTS ai_chat_daily (
  bucket DATE PRIMARY KEY,
  success INTEGER NOT NULL DEFAULT 0,
  upstream_error INTEGER NOT NULL DEFAULT 0,
  quota_denied INTEGER NOT NULL DEFAULT 0,
  rate_denied INTEGER NOT NULL DEFAULT 0,
  guest_calls INTEGER NOT NULL DEFAULT 0,
  user_calls INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS ai_images (
  id BIGSERIAL PRIMARY KEY,
  prompt TEXT NOT NULL,
  image_url TEXT NOT NULL,
  size TEXT NOT NULL DEFAULT '',
  author TEXT NOT NULL DEFAULT '',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ai_fixed_answers (
  id BIGSERIAL PRIMARY KEY,
  question TEXT NOT NULL,
  answer TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS owner_drafts (
  id BIGSERIAL PRIMARY KEY,
  kind TEXT NOT NULL,
  title TEXT NOT NULL DEFAULT '',
  body TEXT NOT NULL DEFAULT '',
  cover_url TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'draft',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS login_challenges (
  token TEXT PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS security_audit_logs (
  id BIGSERIAL PRIMARY KEY,
  action TEXT NOT NULL,
  detail JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE guestbook_messages ADD COLUMN IF NOT EXISTS parent_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE guestbook_messages ADD COLUMN IF NOT EXISTS owner_read_at TIMESTAMPTZ;
ALTER TABLE bilibili_videos ADD COLUMN IF NOT EXISTS cover_remote TEXT NOT NULL DEFAULT '';
ALTER TABLE ai_chat_daily ADD COLUMN IF NOT EXISTS not_configured INTEGER NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS bili_sync (
  uid TEXT PRIMARY KEY,
  state TEXT NOT NULL,
  reason TEXT NOT NULL DEFAULT '',
  attempted_at TIMESTAMPTZ,
  upstream_code INTEGER,
  synced_at TIMESTAMPTZ,
  owner_name TEXT NOT NULL DEFAULT ''
);

ALTER TABLE bili_sync ADD COLUMN IF NOT EXISTS owner_name TEXT NOT NULL DEFAULT '';

CREATE TABLE IF NOT EXISTS ai_chat_hourly (
  bucket TEXT PRIMARY KEY,
  success INTEGER NOT NULL DEFAULT 0,
  upstream_error INTEGER NOT NULL DEFAULT 0,
  quota_denied INTEGER NOT NULL DEFAULT 0,
  rate_denied INTEGER NOT NULL DEFAULT 0,
  not_configured INTEGER NOT NULL DEFAULT 0,
  guest_calls INTEGER NOT NULL DEFAULT 0,
  user_calls INTEGER NOT NULL DEFAULT 0
);

ALTER TABLE security_audit_logs ADD COLUMN IF NOT EXISTS event TEXT NOT NULL DEFAULT '';
ALTER TABLE security_audit_logs ADD COLUMN IF NOT EXISTS outcome TEXT NOT NULL DEFAULT '';
ALTER TABLE security_audit_logs ADD COLUMN IF NOT EXISTS actor_user_id BIGINT NOT NULL DEFAULT 0;
ALTER TABLE security_audit_logs ADD COLUMN IF NOT EXISTS ip_hash TEXT NOT NULL DEFAULT '';
ALTER TABLE security_audit_logs ADD COLUMN IF NOT EXISTS user_agent_hash TEXT NOT NULL DEFAULT '';
ALTER TABLE security_audit_logs ADD COLUMN IF NOT EXISTS target_type TEXT NOT NULL DEFAULT '';
ALTER TABLE security_audit_logs ADD COLUMN IF NOT EXISTS target_id TEXT NOT NULL DEFAULT '';
ALTER TABLE security_audit_logs ADD COLUMN IF NOT EXISTS detail_text TEXT NOT NULL DEFAULT '';
ALTER TABLE ai_images ADD COLUMN IF NOT EXISTS object_key TEXT NOT NULL DEFAULT '';

CREATE TABLE IF NOT EXISTS ai_image_quota (
  identity_key TEXT NOT NULL,
  quota_date DATE NOT NULL,
  used INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (identity_key, quota_date)
);

ALTER TABLE guestbook_messages ADD COLUMN IF NOT EXISTS ip_masked TEXT NOT NULL DEFAULT '';
ALTER TABLE guestbook_messages ADD COLUMN IF NOT EXISTS user_agent_hash TEXT NOT NULL DEFAULT '';
ALTER TABLE guestbook_messages ADD COLUMN IF NOT EXISTS content_hash TEXT NOT NULL DEFAULT '';
ALTER TABLE ai_chat_quota ADD COLUMN IF NOT EXISTS last_request_at TIMESTAMPTZ;
ALTER TABLE owner_drafts ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
ALTER TABLE ai_fixed_answers ADD COLUMN IF NOT EXISTS normalized_question TEXT NOT NULL DEFAULT '';

CREATE UNIQUE INDEX IF NOT EXISTS ai_fixed_answers_normalized_question_key
  ON ai_fixed_answers (normalized_question)
  WHERE normalized_question <> '';

CREATE TABLE IF NOT EXISTS guestbook_ip_cache (
  ip_hash TEXT PRIMARY KEY,
  ip_region TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
