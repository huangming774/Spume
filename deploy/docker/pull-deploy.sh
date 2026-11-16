#!/usr/bin/env bash
# Docker 版 pull-deploy：替代旧的 pull-deploy.sh（git 拉取 + systemd 跑 Go 二进制）。
# GitHub Actions 每次 push 后把镜像推到 GHCR，这台服务器每 5 分钟由 systemd timer
# 调用本脚本：拉最新镜像 → 有更新才重建容器 → 健康检查 → 清理旧镜像。
#
# docker compose up -d 本身就会对比镜像 digest，镜像没变时不会重启容器，
# 所以这里不需要自己做摘要比较。
#
# 用法（服务器上）：
#   COMPOSE_DIR=/opt/taozhiyy /usr/local/sbin/taozhiyy-docker-deploy.sh
set -euo pipefail

COMPOSE_DIR="${COMPOSE_DIR:-/opt/taozhiyy}"
LOCK_FILE="${LOCK_FILE:-/tmp/taozhiyy-docker-deploy.lock}"
HEALTH_URL="${HEALTH_URL:-https://www.miku.coffee/api/v1/health}"

log() {
  printf '[%s] %s\n' "$(date '+%F %T')" "$*"
}

# 防止 timer 触发的部署互相重叠。
exec 9>"$LOCK_FILE"
if ! flock -n 9; then
  log "已有部署任务在跑，跳过本次"
  exit 0
fi

cd "$COMPOSE_DIR"

if [ ! -f docker-compose.yml ]; then
  echo "找不到 $COMPOSE_DIR/docker-compose.yml" >&2
  exit 1
fi

log "拉取最新镜像..."
docker compose pull

log "应用最新镜像（无更新时不会重建容器）..."
if ! docker compose up -d --remove-orphans; then
  log "ERROR: docker compose up -d 失败"
  docker compose ps
  docker compose logs --tail=100
  exit 1
fi

log "健康检查：$HEALTH_URL"
healthy=0
for _ in $(seq 1 30); do
  if curl -fsS --max-time 5 "$HEALTH_URL" | grep -q '"status":"ok"'; then
    healthy=1
    break
  fi
  sleep 2
done

if [ "$healthy" -ne 1 ]; then
  log "ERROR: 健康检查未通过"
  docker compose ps
  docker compose logs --tail=100
  exit 1
fi

log "清理旧镜像..."
docker image prune -f >/dev/null

log "部署完成"
