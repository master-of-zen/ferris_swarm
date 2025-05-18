FROM rust:1.87-alpine3.20 AS builder


RUN apk add --no-cache \
    protobuf-dev \
    pkgconf \
    musl-dev \
    build-base

WORKDIR /usr/src/ferris_swarm
COPY . .

RUN cargo build --release --bin ferris_swarm_node --locked
RUN cargo build --release --bin ferris_swarm_client --locked

FROM jrottenberg/ffmpeg:6.1-alpine AS runtime

WORKDIR /app

COPY --from=builder /usr/src/ferris_swarm/target/release/ferris_swarm_node /usr/local/bin/ferris_swarm_node
COPY --from=builder /usr/src/ferris_swarm/target/release/ferris_swarm_client /usr/local/bin/ferris_swarm_client

COPY config.toml /app/config.toml

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 50051

ENTRYPOINT ["entrypoint.sh"]
CMD ["node"]