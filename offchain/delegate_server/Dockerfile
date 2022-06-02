# syntax=docker.io/docker/dockerfile:1.4
# deps install
FROM rust:1.61.0-bullseye AS chef

RUN rustup component add rustfmt
RUN cargo install cargo-chef

# cargo chef prepare stage
FROM chef AS planner

WORKDIR /usr/src/app
COPY . .
WORKDIR /usr/src/app/offchain
RUN cargo chef prepare --recipe-path recipe.json

# cargo chef cook stage
FROM chef AS builder

WORKDIR /usr/src/app
COPY . .
WORKDIR /usr/src/app/offchain
COPY --from=planner /usr/src/app/offchain/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# build
RUN cargo build --release --bin output_server_main

## runtime
FROM debian:bullseye-slim AS runtime

RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y openssl wget jq
rm -rf /var/lib/apt/lists/*
EOF

ENV DOCKERIZE_VERSION v0.6.1
RUN <<EOF
wget https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz
tar -C /usr/local/bin -xzvf dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz
rm dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz
EOF

WORKDIR /usr/app
COPY --from=builder /usr/src/app/offchain/target/release/output_server_main /usr/local/bin

COPY ./offchain/delegate_server/entrypoint.sh /
ENTRYPOINT ["/entrypoint.sh"]
