# syntax=docker.io/docker/dockerfile:1.4
# deps install
FROM rust:1.61.0-bullseye AS chef

RUN rustup component add rustfmt
RUN cargo install cargo-chef

# cargo chef prepare stage
FROM chef AS planner

WORKDIR /usr/src/app/offchain
COPY . /usr/src/app
RUN cargo chef prepare --recipe-path recipe.json

# cargo chef cook stage
FROM chef AS builder

WORKDIR /usr/src/app/offchain
COPY --from=planner /usr/src/app/offchain/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# build application
WORKDIR /usr/src/app/offchain
COPY . /usr/src/app
RUN cargo build --release --bin output_server_main

# grpc readiness probe
FROM golang:1.18.3-bullseye as grpc_health_probe
RUN go install github.com/grpc-ecosystem/grpc-health-probe@v0.4.11

## runtime
FROM debian:bullseye-slim AS runtime

RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends ca-certificates
rm -rf /var/lib/apt/lists/*
EOF

COPY --from=grpc_health_probe /go/bin/grpc-health-probe /usr/local/bin
COPY --from=builder /usr/src/app/offchain/target/release/output_server_main /usr/local/bin

ENTRYPOINT ["output_server_main"]
