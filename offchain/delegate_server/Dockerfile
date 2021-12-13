# Execute with the context at the repository's root!
# $ docker build -f Dockerfile ../../

# Using `cargo-chef` for caching
# https://github.com/LukeMathWalker/cargo-chef

## Build Stage
# Start with deps
FROM rust:1.56.1 AS chef
RUN rustup component add rustfmt
RUN cargo install cargo-chef
WORKDIR rollups

# cargo chef prepare stage
FROM chef AS planner
WORKDIR offchain
COPY ./offchain .
RUN cargo chef prepare --recipe-path recipe.json

# cargo chef cook stage
FROM chef AS builder
COPY artifacts artifacts
COPY grpc-interfaces grpc-interfaces
WORKDIR offchain
COPY --from=planner rollups/offchain/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# build
COPY ./offchain .
RUN cargo build --release --bin output_server_main
# for alpine, run this instead
# RUN cargo build --release --target x86_64-unknown-linux-musl --bin output_server_main

## Run application
# debian
FROM debian:buster-slim  AS runtime
RUN apt-get update && apt-get install -y openssl wget jq && rm -rf /var/lib/apt/lists/*
ENV DOCKERIZE_VERSION v0.6.1
RUN wget https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    && tar -C /usr/local/bin -xzvf dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    && rm dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz

WORKDIR app
COPY --from=builder /rollups/offchain/target/release/output_server_main /usr/local/bin

# alpine
# FROM alpine AS runtime
# RUN addgroup -S myuser && adduser -S myuser -G myuser
# COPY --from=builder /rollups/offchain/target/x86_64-unknown-linux-musl/release/output_server_main /usr/local/bin
# USER myuser

COPY ./offchain/delegate_server/entrypoint-delegate-server.sh .
ENTRYPOINT ["/bin/sh", "./entrypoint-delegate-server.sh"]
