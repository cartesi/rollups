FROM rust:1.70.0-bookworm AS chef

ENV CARGO_REGISTRIES_CARTESI_INDEX=https://github.com/cartesi/crates-index
RUN rustup component add rustfmt
RUN cargo install cargo-chef

# cargo chef prepare stage
FROM chef AS planner

WORKDIR /usr/src/app/offchain
COPY . /usr/src/app
RUN cargo chef prepare --recipe-path recipe.json --bin machine-hash

# cargo chef cook stage
FROM chef AS builder

RUN <<EOF
DEBIAN_FRONTEND="noninteractive" apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends cmake libprotobuf-dev protobuf-compiler
EOF

WORKDIR /usr/src/app/offchain
COPY --from=planner /usr/src/app/offchain/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

WORKDIR /usr/src/app/offchain/machine-hash
COPY . /usr/src/app

RUN cargo install --path . --root /usr/local
ENTRYPOINT [ "machine-hash" ]