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
COPY crates/libp2p/addr-filter /app/build/crates/libp2p/addr-filter
COPY protos /app/build/protos

COPY sdks/rust /app/build/sdks/rust

WORKDIR /app/build
RUN cargo init --bin --name data-collector /app/build/endpoint/data-collector
COPY endpoint/data-collector/Cargo.toml /app/build/endpoint/data-collector/Cargo.toml


WORKDIR /app/build/endpoint/data-collector
RUN --mount=type=cache,target=/root/.cargo/git/db \
    --mount=type=cache,target=/root/.cargo/registry/cache \
    --mount=type=cache,target=/root/.cargo/registry/index \
    cargo fetch

ARG TARGETPLATFORM


ARG EXTRA_FEATURES

WORKDIR /app/build/endpoint/data-collector

RUN --mount=type=cache,target=/app/build/target/x86_64-unknown-linux-musl \
    --mount=type=cache,target=/app/build/target/aarch64-unknown-linux-musl \
    --mount=type=cache,target=/root/.cargo/git/db \
    --mount=type=cache,target=/root/.cargo/registry/cache \
    --mount=type=cache,target=/root/.cargo/registry/index \
    xx-cargo build -r -p data-collector --target-dir /app/build/target && \
    xx-verify /app/build/target/$(xx-cargo --print-target-triple)/release/data-collector


COPY endpoint/data-collector/src /app/build/endpoint/data-collector/src

RUN touch /app/build/endpoint/data-collector/src/*

WORKDIR /app/build/endpoint/data-collector
RUN --mount=type=cache,target=/app/build/target/x86_64-unknown-linux-musl \
    --mount=type=cache,target=/app/build/target/aarch64-unknown-linux-musl \
    --mount=type=cache,target=/root/.cargo/git/db \
    --mount=type=cache,target=/root/.cargo/registry/cache \
    --mount=type=cache,target=/root/.cargo/registry/index \
    xx-cargo build -r -p data-collector --target-dir /app/build/target && \
    xx-verify /app/build/target/$(xx-cargo --print-target-triple)/release/data-collector
RUN --mount=type=cache,target=/app/build/target/x86_64-unknown-linux-musl \
    --mount=type=cache,target=/app/build/target/aarch64-unknown-linux-musl \
    --mount=type=cache,target=/root/.cargo/git/db \
    --mount=type=cache,target=/root/.cargo/registry/cache \
    --mount=type=cache,target=/root/.cargo/registry/index \
    cp /app/build/target/$(xx-cargo --print-target-triple)/release/data-collector /app/build/data-collector


FROM alpine:latest

COPY --from=builder /app/build/data-collector /app/data-collector

ARG EXTRA_FEATURES

LABEL org.opencontainers.image.ref.name="data-collector"
LABEL org.opencontainers.image.title="data-collector"
LABEL org.opencontainers.image.description="Backend component to gather data from gossipsub and put it into prometheus"
LABEL org.opencontainers.image.base.name="rust:1.80-alpine"
LABEL industries.p2p.app.name="data-collector"

ENV PORT=8080

EXPOSE 8080

CMD ["/app/data-collector"]
