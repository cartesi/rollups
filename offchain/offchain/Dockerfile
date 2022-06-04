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

# buld application
WORKDIR /usr/src/app/offchain
COPY . /usr/src/app
RUN cargo build --release --bin offchain_main

## runtime
FROM debian:bullseye-slim AS runtime

COPY --from=builder /usr/src/app/offchain/target/release/offchain_main /usr/local/bin

ENTRYPOINT ["offchain_main"]
