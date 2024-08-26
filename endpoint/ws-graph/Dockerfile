FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.4.0 AS xx

FROM --platform=$BUILDPLATFORM rust:1.80-alpine AS builder
COPY --from=xx / /
RUN apk add musl-dev protobuf clang lld

COPY Cargo.toml /app/build/Cargo.toml
COPY Cargo.lock /app/build/Cargo.lock

COPY crates/core /app/build/crates/core
COPY crates/macaddress /app/build/crates/macaddress
COPY crates/ifaddr /app/build/crates/ifaddr
COPY crates/ifwatcher /app/build/crates/ifwatcher
COPY crates/batman-neighbours-core /app/build/crates/batman-neighbours-core
COPY crates/batman-neighbours-daemon /app/build/crates/batman-neighbours-daemon
COPY crates/libp2p/batman-adv /app/build/crates/libp2p/batman-adv
COPY protos /app/build/protos

COPY sdks/rust /app/build/sdks/rust

WORKDIR /app/build
RUN cargo init --bin --name ws-graph /app/build/endpoint/ws-graph
COPY endpoint/ws-graph/Cargo.toml /app/build/endpoint/ws-graph/Cargo.toml


WORKDIR /app/build/endpoint/ws-graph
RUN --mount=type=cache,target=/root/.cargo/git/db \
    --mount=type=cache,target=/root/.cargo/registry/cache \
    --mount=type=cache,target=/root/.cargo/registry/index \
    cargo fetch

ARG TARGETPLATFORM


ARG EXTRA_FEATURES

WORKDIR /app/build/endpoint/ws-graph

RUN --mount=type=cache,target=/app/build/target/x86_64-unknown-linux-musl \ 
    --mount=type=cache,target=/app/build/target/aarch64-unknown-linux-musl \
    --mount=type=cache,target=/root/.cargo/git/db \
    --mount=type=cache,target=/root/.cargo/registry/cache \
    --mount=type=cache,target=/root/.cargo/registry/index \
    xx-cargo build -r -p ws-graph -F frontend,${EXTRA_FEATURES} --target-dir /app/build/target && \
    xx-verify /app/build/target/$(xx-cargo --print-target-triple)/release/ws-graph


COPY endpoint/ws-graph/src /app/build/endpoint/ws-graph/src

RUN touch /app/build/endpoint/ws-graph/src/*

WORKDIR /app/build/endpoint/ws-graph
RUN --mount=type=cache,target=/app/build/target/x86_64-unknown-linux-musl \ 
    --mount=type=cache,target=/app/build/target/aarch64-unknown-linux-musl \
    --mount=type=cache,target=/root/.cargo/git/db \
    --mount=type=cache,target=/root/.cargo/registry/cache \
    --mount=type=cache,target=/root/.cargo/registry/index \
    xx-cargo build -r -p ws-graph -F frontend,${EXTRA_FEATURES} --target-dir /app/build/target && \
    xx-verify /app/build/target/$(xx-cargo --print-target-triple)/release/ws-graph
RUN --mount=type=cache,target=/app/build/target/x86_64-unknown-linux-musl \ 
    --mount=type=cache,target=/app/build/target/aarch64-unknown-linux-musl \
    --mount=type=cache,target=/root/.cargo/git/db \
    --mount=type=cache,target=/root/.cargo/registry/cache \
    --mount=type=cache,target=/root/.cargo/registry/index \
    cp /app/build/target/$(xx-cargo --print-target-triple)/release/ws-graph /app/build/ws-graph

FROM --platform=$BUILDPLATFORM node:20-slim AS ui-builder

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

COPY ui /app
WORKDIR /app

FROM ui-builder AS prod-deps
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --prod --forzen-lockfile

FROM ui-builder AS ui-build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

FROM alpine:latest

COPY --from=builder /app/build/ws-graph /app/ws-graph
COPY --from=ui-build /app/build /app/ui

ENV FRONTEND_PATH="/app/ui"

ARG EXTRA_FEATURES

LABEL org.opencontainers.image.ref.name="ws-graph (${EXTRA_FEATURES:-default})"
LABEL org.opencontainers.image.title="ws-graph-${EXTRA_FEATURES:-default}"
LABEL org.opencontainers.image.description="Backend component to UI framework for graphing batman-adv network topology"
LABEL org.opencontainers.image.base.name="rust:1.80-alpine"
LABEL industries.p2p.script.name="ws-graph"

ENV PORT=3000

EXPOSE 3000

CMD ["/app/ws-graph"]
