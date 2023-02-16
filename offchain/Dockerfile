# syntax=docker.io/docker/dockerfile:1.4
# deps install
FROM rust:1.67.0-bullseye AS chef

ENV CARGO_REGISTRIES_CARTESI_INDEX=https://github.com/cartesi/crates-index
RUN rustup component add rustfmt
RUN cargo install cargo-chef

# cargo chef prepare stage
FROM chef AS planner

WORKDIR /usr/src/app/offchain
COPY . /usr/src/app
RUN cargo chef prepare --recipe-path recipe.json

# cargo chef cook stage
FROM chef AS builder

RUN <<EOF
DEBIAN_FRONTEND="noninteractive" apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends cmake libprotobuf-dev protobuf-compiler
EOF

WORKDIR /usr/src/app/offchain
COPY --from=planner /usr/src/app/offchain/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

WORKDIR /usr/src/app/offchain
COPY . /usr/src/app

FROM builder as state_server_build
RUN cargo build --release --bin rollups_state_server

FROM builder as indexer_build
RUN cargo build --release --bin rollups_indexer

FROM builder as dispatcher_build
RUN cargo build --release --bin rollups_dispatcher

FROM builder as inspect_server_build
RUN cargo build --release --bin inspect-server

FROM builder as server_manager_broker_proxy_build
RUN PATH="$PATH:$HOME/.local/bin" cargo build --release --bin server-manager-broker-proxy

FROM builder as reader_build
RUN cargo build --release --bin reader

# grpc readiness probe
FROM golang:1.19.0-bullseye as grpc_health_probe
RUN go install github.com/grpc-ecosystem/grpc-health-probe@v0.4.11

## runtimes
FROM debian:bullseye-slim AS state_server
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends ca-certificates
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=grpc_health_probe /go/bin/grpc-health-probe /usr/local/bin
COPY --from=state_server_build /usr/src/app/offchain/target/release/rollups_state_server /usr/local/bin
ENTRYPOINT ["rollups_state_server"]

FROM rust:1.67.0-bullseye AS indexer
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends libpq5
cargo install diesel_cli --no-default-features --features "postgres"
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=indexer_build /usr/src/app/offchain/target/release/rollups_indexer /usr/local/bin
COPY --from=indexer_build /usr/src/app/offchain/data/migrations/ /usr/local/bin/migrations/
COPY onchain/rollups/export/abi /usr/local/share/cartesi/deployments/
ENTRYPOINT ["rollups_indexer"]

FROM debian:bullseye-slim AS dispatcher
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends ca-certificates
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=dispatcher_build /usr/src/app/offchain/target/release/rollups_dispatcher /usr/local/bin
COPY onchain/rollups/export/abi /usr/local/share/cartesi/deployments/
ENTRYPOINT ["rollups_dispatcher"]

FROM debian:bullseye-slim AS server_manager_broker_proxy
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends ca-certificates curl
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=server_manager_broker_proxy_build /usr/src/app/offchain/target/release/server-manager-broker-proxy /usr/local/bin
ENTRYPOINT ["server-manager-broker-proxy"]

FROM debian:bullseye-slim AS inspect_server
COPY --from=inspect_server_build /usr/src/app/offchain/target/release/inspect-server /usr/local/bin
ENTRYPOINT ["inspect-server"]

FROM rust:1.67.0-bullseye AS reader
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends libpq5
cargo install diesel_cli --no-default-features --features "postgres"
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=reader_build /usr/src/app/offchain/target/release/reader /usr/local/bin
ENTRYPOINT ["reader"]

FROM scratch AS deployments

WORKDIR /usr/local/share/cartesi/deployments
COPY onchain/rollups/export/abi .
