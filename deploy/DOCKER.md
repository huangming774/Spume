# Docker 部署全流程

这套流程把原来的 `pull-deploy.sh`（git 拉取 + Go 二进制 + systemd + nginx）换成
Docker 镜像 + GitHub Actions 推 GHCR + 服务器定时拉取的模式。覆盖范围是
Rust API（`api/`）和主站 SPA 静态站（`main/`，已合并 /blog、/build、/projects 路由），
数据库用外部托管的 Postgres（如 Supabase），不在 Docker 里起数据库容器。

## 架构

```
GitHub push (main)
  └─> GitHub Actions (.github/workflows/docker.yml)
        ├─> 构建 ghcr.io/huangming774/spume-api   (api/Dockerfile)
        └─> 构建 ghcr.io/huangming774/spume-web   (deploy/docker/Dockerfile.web)

服务器 (每 5 分钟 systemd timer)
  └─> deploy/docker/pull-deploy.sh
        ├─> docker compose pull   (拉最新镜像)
        ├─> docker compose up -d  (镜像变了才重建容器)
        └─> 健康检查 + 清理旧镜像

docker compose 里跑两个容器：
  web  (nginx)  ← 80/443 对外
    ├─ /api/*        → 反代到 api:8080（容器内部网络，不对外）
    ├─ /ai-assistant → 静态文件
    └─ 其余路径       → 回落 /index.html（SPA 路由）
  api  (Rust)   ← 只在容器网络内，连外部 Postgres / R2 / 其它云服务
```

## 新增文件一览

| 文件 | 作用 |
| --- | --- |
| `api/Dockerfile`、`api/.dockerignore` | Rust API 多阶段构建 |
| `deploy/docker/Dockerfile.web` | 前端静态站构建（跑 `tools/vercel-build.mjs`）+ nginx 运行 |
| `.dockerignore` | 前端镜像构建上下文过滤（仓库根目录） |
| `deploy/docker/nginx.conf` | SPA fallback、`/api/` 反代、TLS |
| `docker-compose.yml` | `api` + `web` 服务编排 |
| `deploy/docker/pull-deploy.sh` | 服务器端拉取部署脚本（替代旧 `deploy/pull-deploy.sh`） |
| `deploy/docker/taozhiyy-docker-deploy.service` / `.timer` | 定时拉取部署的 systemd 单元（替代旧 `taozhiyy-pull-deploy.*`） |
| `.github/workflows/docker.yml` | push 到 main 时构建并推镜像到 GHCR |
| `deploy/docker/cert-webroot/` | certbot `--webroot` 验证目录 |

## 本地构建测试

不需要先推送 GitHub，本地就能构建并跑起来验证：

```bash
# 需要先准备好 api/.env（从 api/.env.example 复制后填好真实值）
cp api/.env.example api/.env
# 编辑 api/.env，至少填 DATABASE_URL，否则健康检查过不了

# 本地没有真实 TLS 证书时，先生成自签证书做测试：
mkdir -p deploy/docker/certs
openssl req -x509 -nodes -newkey rsa:2048 -days 30 \
  -keyout deploy/docker/certs/privkey.pem \
  -out deploy/docker/certs/fullchain.pem \
  -subj "/CN=localhost"

TLS_CERT_DIR=./deploy/docker/certs docker compose up -d --build
```

打开 https://localhost （自签证书，浏览器会警告，点「继续访问」即可）验证页面和
`/api/v1/health` 是否正常。查看日志：`docker compose logs -f`。

## 发布到服务器

### 一次性初始化

1. 服务器上安装 Docker 和 Docker Compose 插件（`docker compose version` 能跑即可）。

2. 克隆仓库到 `/opt/taozhiyy`（和脚本里的 `COMPOSE_DIR` 默认值一致）：

   ```bash
   sudo git clone https://github.com/huangming774/Spume.git /opt/taozhiyy
   ```

3. 配置环境变量：

   ```bash
   cd /opt/taozhiyy
   sudo cp api/.env.example api/.env
   sudo chmod 600 api/.env
   sudo $EDITOR api/.env   # 填 DATABASE_URL、SESSION_SECRET、API 密钥等
   ```

4. TLS 证书。推荐用宿主机上的 certbot 签发（续期也在宿主机做，不影响容器）：

   ```bash
   # 首次签发（nginx 容器还没跑起来时，用 standalone 模式）
   sudo certbot certonly --standalone -d taozhiyy.top -d www.taozhiyy.top

   # 之后续期用 webroot 模式（nginx 容器已经跑起来时）
   sudo certbot renew \
     --webroot -w /opt/taozhiyy/deploy/docker/cert-webroot \
     --deploy-hook "docker compose -f /opt/taozhiyy/docker-compose.yml exec -T web nginx -s reload"
   ```

   把续期命令加进宿主机 crontab（certbot 自带的 `certbot.timer` 一般已经够用，
   只需要确保 `--deploy-hook` 里的 reload 命令对得上）。

5. 首次启动：

   ```bash
   cd /opt/taozhiyy
   sudo docker compose pull
   sudo docker compose up -d
   ```

6. 安装定时拉取部署（替代旧的 `taozhiyy-pull-deploy.timer`）：

   ```bash
   sudo install -m 0755 deploy/docker/pull-deploy.sh /usr/local/sbin/taozhiyy-docker-deploy.sh
   sudo install -m 0644 deploy/docker/taozhiyy-docker-deploy.service /etc/systemd/system/
   sudo install -m 0644 deploy/docker/taozhiyy-docker-deploy.timer /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable --now taozhiyy-docker-deploy.timer
   ```

### 日常发布

1. 本地改代码，push 到 `main`。
2. GitHub Actions 自动构建并推两个镜像到 GHCR。
3. 服务器的 timer 最多 5 分钟后自动拉到新镜像并重建容器（`docker compose up -d`
   发现镜像没变时不会重启容器，所以定时跑本身很轻）。
4. 想立即手动触发，不等 timer：

   ```bash
   sudo /usr/local/sbin/taozhiyy-docker-deploy.sh
   ```

### 回滚

GitHub Actions 每次构建都会打一个 `sha-<commit>` 标签，永不覆盖。要回滚到某个
历史版本，改 `docker-compose.yml` 里两个 service 的 `image:` 标签为对应的
`sha-<commit>`，然后 `docker compose up -d`：

```yaml
image: ghcr.io/huangming774/spume-api:sha-cde79e6
image: ghcr.io/huangming774/spume-web:sha-cde79e6
```

回滚后记得把 `image:` 改回 `:latest`，或者用 `git checkout` 切回目标 commit 对应的
`docker-compose.yml` 再 `docker compose up -d`。

## 从旧部署方式迁移

原来那套是 `deploy/pull-deploy.sh` + `deploy/taozhiyy-pull-deploy.timer` + 宿主机
nginx + `/opt/acg-api/acg-api`（Go 二进制）+ systemd `acg-api.service`。切到 Docker
时按这个顺序做，可以做到几乎无停机：

1. 新流程先在非 80/443 端口试跑一遍（改 `docker-compose.yml` 里 `ports` 为
   `8081:80`、`8443:443`，`docker compose up -d --build`），确认功能正常。

2. 停掉旧的定时部署，避免它和新的 Docker 部署抢地盘：

   ```bash
   sudo systemctl disable --now taozhiyy-pull-deploy.timer
   sudo systemctl disable --now acg-api.service
   ```

3. 备份旧的静态文件目录（可选，保险起见）：

   ```bash
   sudo tar -czf /home/ubuntu/taozhiyy-www-backup-$(date +%Y%m%d).tar.gz /var/www/taozhiyy
   ```

4. 修改宿主机 nginx 配置：原来直接 `root /var/www/taozhiyy` + `proxy_pass
   http://127.0.0.1:8787` 的 server block 现在不需要了，改成把 80/443 直接交给
   Docker 的端口映射（`ports: "80:80" "443:443"`）。如果宿主机 nginx 还要留着做
   其它站点的入口，可以把它改成只反代到 `127.0.0.1:8081`（web 容器映射出来的端口），
   但更简单的做法是让 Docker 直接占 80/443，宿主机 nginx 停掉或只留 HTTPS 证书路径。

5. 把 `ports` 改回正式的 `80:80`/`443:443`，跑一次：

   ```bash
   cd /opt/taozhiyy
   sudo /usr/local/sbin/taozhiyy-docker-deploy.sh
   ```

6. 确认 `https://taozhiyy.top/api/v1/health` 返回 `"status":"ok"`，页面正常后，
   安装并启用新的 `taozhiyy-docker-deploy.timer`（见「一次性初始化」第 6 步）。

## 常见排障

- **镜像拉不下来 / 403**：GHCR 镜像默认是 private，服务器上 `docker login ghcr.io`
  需要用有 `read:packages` 权限的 token；或者在 GitHub 仓库的 Packages 设置里把
  `spume-api`、`spume-web` 两个包改成 public。
- **健康检查一直 degraded**：`docker compose logs api` 看是哪项服务没连上（db、
  redis、r2、vector 分别对应 `DATABASE_URL`、`UPSTASH_REDIS_REST_URL` 等环境变量），
  `api/.env` 少配某项时该项会显示 `missing`（不算错误），只有配置了却连不上才会
  `down` 并把整体状态拉成 `degraded`。
- **改了 `nginx.conf` 不生效**：`nginx.conf` 是打进 web 镜像的，不是挂载的，需要
  重新构建镜像（`docker compose build web && docker compose up -d web`）而不是只
  restart 容器。
- **证书换了 nginx 不认**：证书是只读挂载的，宿主机上 certbot 续期后要 reload 容器里的
  nginx：`docker compose exec -T web nginx -s reload`（certbot 的 `--deploy-hook`
  已经配好了这一步）。
