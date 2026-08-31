# 阶段 1：前端（proto 生成 + vite 构建）
FROM node:26-bookworm-slim AS web-builder
WORKDIR /app
COPY package.json package-lock.json ./
COPY web/package.json web/package.json
RUN npm ci
RUN npm install -g @bufbuild/buf
COPY proto ./proto
COPY web ./web
ENV PATH="/app/node_modules/.bin:$PATH"
RUN cd proto && buf generate
RUN cd web && npm run build

# 阶段 2：Rust 构建
FROM rust:1-bookworm AS builder
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl unzip \
    && rm -rf /var/lib/apt/lists/* \
    && curl -fsSL https://github.com/protocolbuffers/protobuf/releases/download/v36.0/protoc-36.0-linux-x86_64.zip -o /tmp/protoc.zip \
    && unzip -q /tmp/protoc.zip -d /usr/local \
    && rm /tmp/protoc.zip
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY proto ./proto
COPY --from=web-builder /app/web/dist ./web/dist
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release -p lemma-server \
    && cp target/release/lemma-server /app/lemma-server

# 阶段 3：运行时（单二进制镜像）
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
RUN useradd --system --create-home lemma
COPY --from=builder /app/lemma-server /usr/local/bin/lemma-server
USER lemma
EXPOSE 1025
ENTRYPOINT ["lemma-server"]
