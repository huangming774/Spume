from pathlib import Path
import unittest


SCRIPT_PATH = Path(__file__).resolve().parents[1] / "deploy" / "remote-install-acg-api.sh"
SYNC_ENV_SCRIPT_PATH = Path(__file__).resolve().parents[1] / "deploy" / "sync-auth-env.sh"


class RemoteInstallAcgApiTests(unittest.TestCase):
    def test_waits_for_acg_api_to_become_active(self):
        text = SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("wait_for_service_active()", text)
        self.assertIn("wait_for_http_ready()", text)
        self.assertIn('state="$(run_sudo systemctl is-active "$service" || true)"', text)
        self.assertIn('"$service is $state; waiting..."', text)
        self.assertIn("wait_for_service_active acg-api 30 1", text)
        self.assertIn("wait_for_http_ready http://127.0.0.1:8787/api/v1/health 30 1", text)
        self.assertNotIn("run_sudo systemctl is-active acg-api\n", text)

    def test_logs_acg_api_status_and_journal_when_wait_fails(self):
        text = SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn('systemctl --no-pager --full status "$service"', text)
        self.assertIn('journalctl -u "$service" -n 80 --no-pager', text)
        self.assertIn("grep -E ':(8787)[[:space:]]' || true", text)

    def test_nginx_api_snippet_allows_owner_asset_upload_size(self):
        text = SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn('client_max_body_size 10m;', text)

    def test_nginx_api_snippet_adds_security_headers_and_rate_limits(self):
        text = SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("limit_req_zone $binary_remote_addr zone=api_guestbook:10m rate=5r/m;", text)
        self.assertIn("limit_req_zone $binary_remote_addr zone=api_auth:10m rate=10r/m;", text)
        self.assertIn("limit_req_zone $binary_remote_addr zone=api_chat:10m rate=30r/m;", text)
        self.assertIn("run_sudo mkdir -p /etc/nginx/conf.d", text)
        self.assertIn('add_header X-Frame-Options "SAMEORIGIN" always;', text)
        self.assertIn('add_header X-Content-Type-Options "nosniff" always;', text)
        self.assertIn('add_header Referrer-Policy "strict-origin-when-cross-origin" always;', text)
        self.assertIn('add_header Permissions-Policy "geolocation=(), microphone=(), camera=()" always;', text)
        self.assertIn('add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;', text)
        self.assertIn("limit_req zone=api_guestbook burst=10 nodelay;", text)
        self.assertIn("limit_req zone=api_auth burst=20 nodelay;", text)
        self.assertIn("limit_req zone=api_chat burst=30 nodelay;", text)

    def test_nginx_snippet_proxies_cos_assets_with_security_headers(self):
        text = SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("location /cos/", text)
        self.assertIn("tzyy-1330068502.cos.ap-beijing.myqcloud.com", text)
        self.assertIn("proxy_ssl_server_name on;", text)
        self.assertIn('proxy_set_header Host "tzyy-1330068502.cos.ap-beijing.myqcloud.com";', text)
        self.assertIn('add_header X-Content-Type-Options "nosniff" always;', text)
        self.assertIn('add_header Cache-Control "public, max-age=31536000, immutable" always;', text)

    def test_static_main_location_gets_security_headers(self):
        text = SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("ensure_static_security_headers()", text)
        self.assertIn("find_taozhiyy_nginx_conf()", text)
        self.assertIn('stripped.startswith("add_header Cache-Control")', text)
        self.assertIn('add_header Cache-Control "no-cache" always;', text)
        self.assertNotIn('add_header Cache-Control "public, max-age=3600"', text)
        self.assertIn("/var/backups/nginx", text)
        self.assertNotIn('$conf.before-static-security-', text)
        self.assertNotIn('run_sudo python3 - "$conf" >"$tmp"', text)
        self.assertIn('add_header X-Frame-Options "SAMEORIGIN" always;', text)
        self.assertIn('add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;', text)

    def test_server_baseline_installs_fail2ban_and_database_backup(self):
        text = SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("ensure_fail2ban()", text)
        self.assertIn("install_sqlite_backup_cron()", text)
        self.assertIn("systemctl enable --now fail2ban", text)
        self.assertIn("/var/backups/acg-api", text)
        self.assertIn("sqlite3 /var/lib/acg-api/acg.db", text)

    def test_sudo_password_env_takes_precedence_and_two_arg_install_is_supported(self):
        text = SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn('SUDO_PASSWORD="${SUDO_PASSWORD:-${2:-}}"', text)
        self.assertIn('SERVICE_SRC="${2:?usage: remote-install-acg-api.sh <binary> [sudo_password] <unit-file>}"', text)

    def test_nginx_api_snippet_is_rewritten_on_redeploy(self):
        text = SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn('SNIP_TMP=$(mktemp)', text)
        self.assertIn('cat >"$SNIP_TMP" <<\'NGX\'', text)
        self.assertIn('run_sudo install -m 0644 "$SNIP_TMP" "$SNIP"', text)
        self.assertNotIn('run_sudo tee "$SNIP" >/dev/null <<\'NGX\'', text)
        self.assertNotIn('if ! run_sudo test -f "$SNIP"; then', text)


class SyncAuthEnvTests(unittest.TestCase):
    def test_waits_for_acg_api_after_env_restart(self):
        text = SYNC_ENV_SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("wait_for_service_active()", text)
        self.assertIn("wait_for_http_ready()", text)
        self.assertIn("if run_sudo systemctl restart acg-api", text)
        self.assertIn("wait_for_service_active acg-api 30 1", text)
        self.assertIn("wait_for_http_ready http://127.0.0.1:8787/api/v1/health 30 1", text)
        self.assertIn('systemctl --no-pager --full status "$service"', text)
        self.assertIn('journalctl -u "$service" -n 80 --no-pager', text)
        self.assertIn('chown root:root "$ENV_FILE"', text)
        self.assertIn('chmod 600 "$ENV_FILE"', text)

    def test_syncs_sync_trigger_token(self):
        text = SYNC_ENV_SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("SYNC_TRIGGER_TOKEN", text)

    def test_syncs_owner_publish_github_env_keys(self):
        text = SYNC_ENV_SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("OWNER_PUBLISH_GITHUB_TOKEN", text)
        self.assertIn("OWNER_PUBLISH_GITHUB_OWNER", text)
        self.assertIn("OWNER_PUBLISH_GITHUB_REPO", text)
        self.assertIn("OWNER_PUBLISH_GITHUB_BRANCH", text)

    def test_syncs_cos_env_keys(self):
        text = SYNC_ENV_SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("TENCENT_COS_SECRET_ID", text)
        self.assertIn("TENCENT_COS_SECRET_KEY", text)
        self.assertIn("TENCENT_COS_BUCKET", text)
        self.assertIn("TENCENT_COS_REGION", text)
        self.assertIn("TENCENT_COS_BASE_URL", text)

    def test_syncs_smtp_notification_env_keys(self):
        text = SYNC_ENV_SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("SMTP_HOST", text)
        self.assertIn("SMTP_PORT", text)
        self.assertIn("SMTP_USER", text)
        self.assertIn("SMTP_PASS", text)
        self.assertIn("SMTP_FROM_NAME", text)
        self.assertIn("MAIL_NOTIFY_TO", text)

    def test_syncs_agnes_image_generation_env_keys(self):
        text = SYNC_ENV_SCRIPT_PATH.read_text(encoding="utf-8")

        self.assertIn("AGNES_API_KEY", text)
        self.assertIn("AGNES_BASE_URL", text)
        self.assertIn("AGNES_IMAGE_MODEL", text)
        self.assertIn("DASHSCOPE_API_KEY", text)
        self.assertIn("DASHSCOPE_BASE_URL", text)
        self.assertIn("AI_IMAGE_MODEL", text)


if __name__ == "__main__":
    unittest.main()
