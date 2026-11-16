# syntax=docker/dockerfile:1
#
# Spume API (Rust/Axum) 生产镜像。
# 构建上下文是 api/ 目录本身：
#   docker build -t taozhiyy-api ./api
#
# 监听端口由 PORT 决定，没有 PORT 时用 ACG_API_ADDR，默认 :8080。
# Vercel 容器可以在环境里把 PORT 设为 80。

FROM rust:1.83-bookworm-slim AS builder

WORKDIR /usr/src/taozhiyy-api

COPY _core/Cargo.toml _core/Cargo.lock ./
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

COPY --from=builder /usr/src/taozhiyy-api/target/release/taozhiyy-api /opt/acg-api/taozhiyy-api

RUN chown -R taozhiyy:taozhiyy /opt/acg-api
USER taozhiyy

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/api/v1/health | grep -q '"status":"ok"' || exit 1

ENTRYPOINT ["/opt/acg-api/taozhiyy-api"]
