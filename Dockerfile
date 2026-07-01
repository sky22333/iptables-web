# syntax=docker/dockerfile:1

FROM node:22-alpine AS web
WORKDIR /web
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm install
COPY frontend/ ./
RUN npm run build

FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock* backend/build.rs ./
COPY backend/migrations ./migrations
COPY backend/src ./src
COPY --from=web /web/dist ../frontend/dist
ENV SKIP_WEB_BUILD=1
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/realm-web /usr/local/bin/realm-web
RUN mkdir -p /app/data
ENV DATA_DIR=/app/data
EXPOSE 888
ENTRYPOINT ["realm-web"]
