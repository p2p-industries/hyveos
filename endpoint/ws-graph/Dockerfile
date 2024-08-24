FROM rust:1.80-alpine AS builder
RUN apk add musl-dev protobuf

COPY . /app/build
RUN rm /app/build/Cargo.lock

# This is a hack to avoid cloning the private repository with libp2p-quic in it
RUN rm -r /app/build/crates/p2p-stack
RUN rm -r /app/build/crates/bridge
RUN rm -r /app/build/crates/stack


WORKDIR /app/build/endpoint/ws-graph
RUN cargo build -r -p ws-graph

FROM alpine:latest

COPY --from=builder /app/build/target/release/ws-graph /app/ws-graph

CMD ["/app/ws-graph"]
