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
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends cmake
export ARCH=$(uname -m | sed 's/aarch64/aarch_64/')
curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v3.20.1/protoc-3.20.1-linux-$ARCH.zip
unzip protoc-3.20.1-linux-$ARCH.zip -d $HOME/.local
export PATH="$PATH:$HOME/.local/bin"
cargo chef cook --release --recipe-path recipe.json
EOF

# build application
WORKDIR /usr/src/app/offchain
COPY . /usr/src/app
RUN cargo build --release --bin reader

## runtime
FROM rust:1.61.0-bullseye AS runtime

RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends libpq5
cargo install diesel_cli --no-default-features --features "postgres"
rm -rf /var/lib/apt/lists/*
EOF


COPY --from=builder /usr/src/app/offchain/target/release/reader /usr/local/bin

ENTRYPOINT ["reader"]
