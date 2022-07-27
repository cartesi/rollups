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
RUN <<EOF
export ARCH=$(uname -m | sed 's/aarch64/aarch_64/')
curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v3.20.1/protoc-3.20.1-linux-$ARCH.zip
unzip protoc-3.20.1-linux-$ARCH.zip -d $HOME/.local
export PATH="$PATH:$HOME/.local/bin"
cargo chef cook --release --recipe-path recipe.json
EOF

# build application
WORKDIR /usr/src/app/offchain
COPY . /usr/src/app
RUN cargo build --release --bin rollups_state_server

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
COPY --from=builder /usr/src/app/offchain/target/release/rollups_state_server /usr/local/bin

ENTRYPOINT ["rollups_state_server"]
