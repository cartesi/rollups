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

FROM builder as dispatcher_build
RUN cargo build --release --bin rollups_dispatcher

FROM builder as indexer_build
RUN cargo build --release --bin indexer

FROM builder as inspect_server_build
RUN cargo build --release --bin inspect-server

FROM builder as advance_runner_build
RUN PATH="$PATH:$HOME/.local/bin" cargo build --release --bin advance-runner

FROM builder as graphql_server_build
RUN cargo build --release --bin graphql-server

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

FROM debian:bullseye-slim AS indexer
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends libpq5
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=indexer_build /usr/src/app/offchain/target/release/indexer /usr/local/bin
ENTRYPOINT ["indexer"]

FROM debian:bullseye-slim AS dispatcher
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends ca-certificates
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=dispatcher_build /usr/src/app/offchain/target/release/rollups_dispatcher /usr/local/bin
COPY onchain/rollups/export/abi /usr/local/share/cartesi/deployments/
ENTRYPOINT ["rollups_dispatcher"]

FROM debian:bullseye-slim AS advance_runner
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends ca-certificates curl
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=advance_runner_build /usr/src/app/offchain/target/release/advance-runner /usr/local/bin
ENTRYPOINT ["advance-runner"]

FROM debian:bullseye-slim AS inspect_server
COPY --from=inspect_server_build /usr/src/app/offchain/target/release/inspect-server /usr/local/bin
ENTRYPOINT ["inspect-server"]

FROM rust:1.67.0-bullseye AS graphql_server
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends libpq5
cargo install diesel_cli --no-default-features --features "postgres"
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=graphql_server_build /usr/src/app/offchain/target/release/graphql-server /usr/local/bin
ENTRYPOINT ["graphql-server"]

FROM scratch AS deployments

WORKDIR /usr/local/share/cartesi/deployments
COPY onchain/rollups/export/abi .
