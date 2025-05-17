FROM rust:1.87-alpine3.20 AS builder


RUN apk add --no-cache \
    protobuf-dev \
    pkgconf \
    musl-dev \
    build-base

WORKDIR /usr/src/ferris_swarm

COPY Cargo.toml Cargo.lock ./
COPY build.rs ./build.rs
COPY proto ./proto

RUN mkdir -p src/bin src/ffmpeg && \
    touch src/lib.rs && \
    touch src/chunk.rs && \
    touch src/config.rs && \
    touch src/error.rs && \
    touch src/logging.rs && \
    touch src/settings.rs && \
    touch src/ffmpeg/mod.rs && \
    touch src/ffmpeg/concat.rs && \
    touch src/ffmpeg/segment.rs && \
    echo "fn main() { panic!(\"Dummy main for ferris_swarm_node, used for dep caching only\"); }" > src/bin/node.rs && \
    echo "fn main() { panic!(\"Dummy main for ferris_swarm_client, used for dep caching only\"); }" > src/bin/client.rs

RUN cargo build --release --bin ferris_swarm_node --locked
RUN cargo build --release --bin ferris_swarm_client --locked

COPY src ./src

RUN cargo build --release --bin ferris_swarm_node --locked
RUN cargo build --release --bin ferris_swarm_client --locked

# Stage 2: Create the runtime image using a pre-built FFmpeg image
FROM jrottenberg/ffmpeg:6.1-alpine AS runtime

WORKDIR /app

COPY --from=builder /usr/src/ferris_swarm/target/release/ferris_swarm_node /usr/local/bin/ferris_swarm_node
COPY --from=builder /usr/src/ferris_swarm/target/release/ferris_swarm_client /usr/local/bin/ferris_swarm_client

COPY config.toml /app/config.toml

# Copy the entrypoint script and make it executable
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 50051

ENTRYPOINT ["entrypoint.sh"]

# Default command passed to entrypoint.sh if nothing else is specified on 'docker run'
CMD ["node"]