#!/usr/bin/env bash
# Merge deployment-managed vars into /opt/acg-api/.env (run on server). Does not remove other keys.
set -euo pipefail

ENV_FILE="/opt/acg-api/.env"
FRAGMENT="${1:-}"

run_sudo() {
  if [ -n "${SUDO_PASSWORD:-}" ]; then
    echo "$SUDO_PASSWORD" | sudo -S -p '' "$@"
  else
    sudo "$@"
  fi
}

log_service_diagnostics() {
  local service="${1:?service required}"

  echo "=== $service status ===" >&2
  run_sudo systemctl --no-pager --full status "$service" >&2 || true
  echo "=== $service journal ===" >&2
  run_sudo journalctl -u "$service" -n 80 --no-pager >&2 || true
  echo "=== listening on 8787 ===" >&2
  run_sudo sh -c 'if command -v ss >/dev/null 2>&1; then ss -tlnp; else netstat -tlnp; fi' | grep -E ':(8787)[[:space:]]' || true
}

wait_for_service_active() {
  local service="${1:?service required}"
  local attempts="${2:-30}"
  local delay="${3:-1}"
  local state=""
  local i

  for i in $(seq 1 "$attempts"); do
    state="$(run_sudo systemctl is-active "$service" || true)"
    if [ "$state" = "active" ]; then
      echo "$service is active"
      return 0
    fi
    if [ "$state" = "failed" ]; then
      echo "$service entered failed state; status:" >&2
      log_service_diagnostics "$service"
      return 1
    fi

    echo "$service is $state; waiting..." >&2
    sleep "$delay"
  done

  echo "$service did not become active; status:" >&2
  log_service_diagnostics "$service"
  return 1
}

wait_for_http_ready() {
  local url="${1:?url required}"
  local attempts="${2:-30}"
  local delay="${3:-1}"
  local i

  for i in $(seq 1 "$attempts"); do
    if curl -fsS --max-time 2 "$url" >/dev/null; then
      echo "$url is ready"
      return 0
    fi

    echo "$url is not ready; waiting..." >&2
    sleep "$delay"
  done

  echo "$url did not become ready; diagnostics:" >&2
  log_service_diagnostics acg-api
  return 1
}

run_sudo mkdir -p "$(dirname "$ENV_FILE")"
run_sudo touch "$ENV_FILE"
run_sudo chown root:root "$ENV_FILE"
run_sudo chmod 600 "$ENV_FILE"

load_fragment_to_env() {
  local frag="$1"
  [ -f "$frag" ] || return 0
  while IFS= read -r line || [ -n "$line" ]; do
    line="${line//$'\r'/}"
    [[ "$line" =~ ^#.*$ || -z "$line" ]] && continue
    key="${line%%=*}"
    val="${line#*=}"
    case "$key" in
      AUTH_OWNER_PASSWORD|AUTH_OWNER_SECURITY_ANSWER|AUTH_SESSION_DAYS|AGNES_API_KEY|AGNES_BASE_URL|AGNES_IMAGE_MODEL|DASHSCOPE_API_KEY|DASHSCOPE_BASE_URL|AI_IMAGE_MODEL|PEXELS_API_KEY|PIXABAY_API_KEY|TENCENT_COS_SECRET_ID|TENCENT_COS_SECRET_KEY|TENCENT_COS_BUCKET|TENCENT_COS_REGION|TENCENT_COS_BASE_URL|OWNER_PUBLISH_GITHUB_TOKEN|OWNER_PUBLISH_GITHUB_OWNER|OWNER_PUBLISH_GITHUB_REPO|OWNER_PUBLISH_GITHUB_BRANCH|SMTP_HOST|SMTP_PORT|SMTP_USER|SMTP_PASS|SMTP_FROM_NAME|MAIL_NOTIFY_TO|SYNC_TRIGGER_TOKEN)
        export "$key"="$val"
        ;;
    esac
  done <"$frag"
  rm -f "$frag"
}

merge_keys_python() {
  if ! command -v python3 >/dev/null 2>&1; then
    echo "error: python3 is required on the server to merge env" >&2
    exit 1
  fi
  local src merged
  src=$(mktemp)
  merged=$(mktemp)
  run_sudo cat "$ENV_FILE" >"$src" 2>/dev/null || : >"$src"
  python3 - "$src" "$merged" <<'PY'
import os, re, sys

src_path, out_path = sys.argv[1], sys.argv[2]
allowed = (
    "AUTH_OWNER_PASSWORD",
    "AUTH_OWNER_SECURITY_ANSWER",
    "AUTH_SESSION_DAYS",
    "AGNES_API_KEY",
    "AGNES_BASE_URL",
    "AGNES_IMAGE_MODEL",
    "DASHSCOPE_API_KEY",
    "DASHSCOPE_BASE_URL",
    "AI_IMAGE_MODEL",
    "PEXELS_API_KEY",
    "PIXABAY_API_KEY",
    "TENCENT_COS_SECRET_ID",
    "TENCENT_COS_SECRET_KEY",
    "TENCENT_COS_BUCKET",
    "TENCENT_COS_REGION",
    "TENCENT_COS_BASE_URL",
    "OWNER_PUBLISH_GITHUB_TOKEN",
    "OWNER_PUBLISH_GITHUB_OWNER",
    "OWNER_PUBLISH_GITHUB_REPO",
    "OWNER_PUBLISH_GITHUB_BRANCH",
    "SMTP_HOST",
    "SMTP_PORT",
    "SMTP_USER",
    "SMTP_PASS",
    "SMTP_FROM_NAME",
    "MAIL_NOTIFY_TO",
    "SYNC_TRIGGER_TOKEN",
)
updates = {k: os.environ.get(k, "").strip() for k in allowed}
updates = {k: v for k, v in updates.items() if v}
try:
    with open(src_path, "r", encoding="utf-8") as f:
        lines = f.readlines()
except FileNotFoundError:
    lines = []
out, seen = [], set()
for line in lines:
    m = re.match(r"^([A-Z0-9_]+)=", line)
    if m and m.group(1) in updates:
        if m.group(1) not in seen:
            out.append(f"{m.group(1)}={updates[m.group(1)]}\n")
            seen.add(m.group(1))
        continue
    out.append(line)
for k, v in updates.items():
    if k not in seen:
        if out and not out[-1].endswith("\n"):
            out.append("\n")
        out.append(f"{k}={v}\n")
with open(out_path, "w", encoding="utf-8") as f:
    f.writelines(out)
PY
  run_sudo cp "$merged" "$ENV_FILE"
  run_sudo chown root:root "$ENV_FILE"
  run_sudo chmod 600 "$ENV_FILE"
  rm -f "$src" "$merged"
}

if [ -n "$FRAGMENT" ]; then
  load_fragment_to_env "$FRAGMENT"
fi

merge_keys_python

if run_sudo systemctl restart acg-api; then
  echo "env synced; acg-api restart requested"
else
  echo "error: acg-api restart failed" >&2
  log_service_diagnostics acg-api
  exit 1
fi
if ! wait_for_service_active acg-api 30 1; then
  exit 1
fi
if ! wait_for_http_ready http://127.0.0.1:8787/api/v1/health 30 1; then
  exit 1
fi
