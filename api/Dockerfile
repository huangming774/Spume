FROM rust:1-bookworm AS build
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src
RUN cargo build --release --locked --bin taozhiyy-api

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /app/target/release/taozhiyy-api /usr/local/bin/taozhiyy-api
ENV PORT=80
EXPOSE 80
ENTRYPOINT ["/usr/local/bin/taozhiyy-api"]
