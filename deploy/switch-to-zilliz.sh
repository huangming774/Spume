#!/usr/bin/env bash
# 把 /opt/acg-api/.env 里的 UPSTASH_VECTOR_* 换成 ZILLIZ_*（在服务器上运行）。
#
# 用法：
#   bash switch-to-zilliz.sh                          # 交互询问端点和 API Key（不回显）
#   bash switch-to-zilliz.sh <ZILLIZ_URI> <ZILLIZ_TOKEN> [ZILLIZ_COLLECTION]
#
# ZILLIZ_URI 是 cloud.zilliz.com 集群的公共端点，例如 https://xxxx.zillizcloud.com。
# 集合名默认 taozhiyy_search。旧的 UPSTASH_VECTOR_* 行会被注释掉（不删除），
# 改动前会备份到 /opt/acg-api/.env.bak-zilliz-<时间戳>。
set -euo pipefail

ENV_FILE="/opt/acg-api/.env"
HEALTH_URL="${HEALTH_URL:-http://127.0.0.1:8787/api/v1/health}"

ZILLIZ_URI="${1:-${ZILLIZ_URI:-}}"
ZILLIZ_TOKEN="${2:-${ZILLIZ_TOKEN:-}}"
ZILLIZ_COLLECTION="${3:-${ZILLIZ_COLLECTION:-taozhiyy_search}}"

if [ -z "$ZILLIZ_URI" ]; then
  read -r -p "Zilliz 集群公共端点（如 https://xxxx.zillizcloud.com）: " ZILLIZ_URI
fi
if [ -z "$ZILLIZ_TOKEN" ]; then
  read -r -s -p "Zilliz API Key: " ZILLIZ_TOKEN
  echo
fi
ZILLIZ_URI="${ZILLIZ_URI%/}"
if [ -z "$ZILLIZ_URI" ] || [ -z "$ZILLIZ_TOKEN" ]; then
  echo "error: ZILLIZ_URI 和 ZILLIZ_TOKEN 都不能为空" >&2
  exit 1
fi

run_sudo() {
  if [ -n "${SUDO_PASSWORD:-}" ]; then
    echo "$SUDO_PASSWORD" | sudo -S -p '' "$@"
  else
    sudo "$@"
  fi
}

run_sudo mkdir -p "$(dirname "$ENV_FILE")"
run_sudo touch "$ENV_FILE"
run_sudo chown root:root "$ENV_FILE"
run_sudo chmod 600 "$ENV_FILE"

BACKUP="$ENV_FILE.bak-zilliz-$(date +%Y%m%d%H%M%S)"
run_sudo cp "$ENV_FILE" "$BACKUP"
run_sudo chown root:root "$BACKUP"
run_sudo chmod 600 "$BACKUP"
echo "backed up to $BACKUP"

src=$(mktemp)
merged=$(mktemp)
run_sudo cat "$ENV_FILE" >"$src" 2>/dev/null || : >"$src"

python3 - "$src" "$merged" "$ZILLIZ_URI" "$ZILLIZ_TOKEN" "$ZILLIZ_COLLECTION" <<'PY'
import re, sys

src_path, out_path, uri, token, collection = sys.argv[1:6]
drop = {"UPSTASH_VECTOR_REST_URL", "UPSTASH_VECTOR_REST_TOKEN"}
add = {
    "ZILLIZ_URI": uri,
    "ZILLIZ_TOKEN": token,
    "ZILLIZ_COLLECTION": collection,
}
try:
    with open(src_path, "r", encoding="utf-8") as f:
        lines = f.readlines()
except FileNotFoundError:
    lines = []
out, seen = [], set()
for line in lines:
    m = re.match(r"^([A-Z0-9_]+)=", line)
    if m and m.group(1) in drop:
        # 注释而不是删除：老 Go 后端还读这个文件时可以随时恢复。
        out.append("# " + line.rstrip("\n") + "  # replaced by ZILLIZ_* on 2026-11-13\n")
        continue
    if m and m.group(1) in add:
        if m.group(1) not in seen:
            out.append(f"{m.group(1)}={add[m.group(1)]}\n")
            seen.add(m.group(1))
        continue
    out.append(line)
for k, v in add.items():
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
echo "env updated: ZILLIZ_URI / ZILLIZ_TOKEN / ZILLIZ_COLLECTION written, UPSTASH_VECTOR_* commented out"

if run_sudo systemctl restart acg-api; then
  echo "acg-api restart requested"
else
  echo "error: acg-api restart failed" >&2
  exit 1
fi

for i in $(seq 1 30); do
  if curl -fsS --max-time 2 "$HEALTH_URL" >/dev/null 2>&1; then
    echo "$HEALTH_URL is ready"
    echo "done. 记得触发一次 POST /api/internal/index-content 建集合并写入索引。"
    exit 0
  fi
  sleep 1
done
echo "warning: $HEALTH_URL 还没就绪，服务可能仍在启动，请手动确认" >&2
exit 1
