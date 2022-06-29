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
curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v3.20.1/protoc-3.20.1-linux-x86_64.zip
unzip protoc-3.20.1-linux-x86_64.zip -d $HOME/.local
export PATH="$PATH:$HOME/.local/bin"
cargo chef cook --release --recipe-path recipe.json
EOF

# buld application
WORKDIR /usr/src/app/offchain
COPY . /usr/src/app
RUN cargo build --release --bin indexer_main

## runtime
FROM debian:bullseye-slim AS runtime

RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends libpq5
rm -rf /var/lib/apt/lists/*
EOF

COPY --from=builder /usr/src/app/offchain/target/release/indexer_main /usr/local/bin

ENTRYPOINT ["indexer_main"]
