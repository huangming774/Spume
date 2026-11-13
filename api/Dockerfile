# syntax=docker/dockerfile:1
#
# Taozhiyy API (Rust/Axum) 生产镜像。
# 构建上下文是 api/ 目录本身：
#   docker build -t taozhiyy-api ./api
#
# 数据库迁移脚本 (migrations/001_init.sql) 通过 include_str! 在编译期
# 打进二进制，运行期不需要额外文件。配置走环境变量（见 api/.env.example），
# 也可以把 .env 挂到 /opt/acg-api/.env（config.rs 会按 Go 版约定读取该路径）。

# ---------- 构建阶段 ----------
# rust:1.83 起支持 Cargo.lock v4（edition 2021 也在此版本范围内）。
# 若升级依赖后报 "requires a newer version of rustc"，只需改这一行的版本号。
FROM rust:1.83-bookworm-slim AS builder

WORKDIR /usr/src/taozhiyy-api

# 先只拷贝依赖清单，利用 Docker 层缓存加速：依赖没变时跳过编译。
COPY Cargo.toml Cargo.lock ./
# 占位 main.rs 让 cargo 把依赖编译进缓存层；随后用真实源码覆盖再编译一次。
RUN mkdir -p src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY src ./src
COPY migrations ./migrations

# 触发最终编译：只改业务代码时，上面的依赖缓存层仍然命中。
RUN touch src/main.rs && cargo build --release --locked

# ---------- 运行阶段 ----------
FROM debian:bookworm-slim AS runtime

# ca-certificates：reqwest/lettre 走 rustls 时用得上（证书链校验）。
# curl：容器健康检查 + 排障用。
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# 非 root 运行，和 Go 版 /opt/acg-api 的部署路径约定保持一致。
RUN useradd --system --create-home --home-dir /opt/acg-api taozhiyy

WORKDIR /opt/acg-api

COPY --from=builder /usr/src/taozhiyy-api/target/release/taozhiyy-api /opt/acg-api/taozhiyy-api

RUN chown -R taozhiyy:taozhiyy /opt/acg-api
USER taozhiyy

# 对齐 config.rs 的 bind_addr(":8080") -> 0.0.0.0:8080 默认监听地址。
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/api/v1/health | grep -q '"status":"ok"' || exit 1

ENTRYPOINT ["/opt/acg-api/taozhiyy-api"]
