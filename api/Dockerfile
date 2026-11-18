# syntax=docker/dockerfile:1
#
# Spume API (Rust/Axum) 生产镜像。
# 构建上下文是 api/ 目录本身：
#   docker build -t taozhiyy-api ./api
#
# 监听端口由 PORT 决定，没有 PORT 时用 ACG_API_ADDR，默认 :8080。
# Vercel 容器可以在环境里把 PORT 设为 80。

# 标签命名是 <版本>-slim-bookworm；版本与 rust-toolchain.toml 锁定一致（sqlx 0.9 要求 1.94+）。
FROM rust:1.94-slim-bookworm AS builder

WORKDIR /usr/src/taozhiyy-api

# crate 实际在 _core/ 下，保持目录布局，避免迁移等相对路径失效。
COPY _core ./_core
WORKDIR /usr/src/taozhiyy-api/_core

# 先用空 main 预编译依赖层，源码变动时不重编依赖。
RUN mkdir -p src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY _core/src ./src
COPY _core/migrations ./migrations

RUN touch src/main.rs && cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --create-home --home-dir /opt/acg-api taozhiyy

WORKDIR /opt/acg-api

COPY --from=builder /usr/src/taozhiyy-api/_core/target/release/taozhiyy-api /opt/acg-api/taozhiyy-api

RUN chown -R taozhiyy:taozhiyy /opt/acg-api
USER taozhiyy

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/api/v1/health | grep -q '"status":"ok"' || exit 1

ENTRYPOINT ["/opt/acg-api/taozhiyy-api"]
