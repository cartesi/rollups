# Execute with the context at the repository's root!
# $ docker build -f Dockerfile ../../

# Using `cargo-chef` for caching
# https://github.com/LukeMathWalker/cargo-chef

# Build Stage
FROM rust:1.56.1 AS chef
RUN rustup component add rustfmt
RUN cargo install cargo-chef
WORKDIR rollups

FROM chef AS planner
COPY artifacts artifacts
COPY grpc-interfaces grpc-interfaces
WORKDIR offchain
COPY ./offchain .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY artifacts artifacts
COPY grpc-interfaces grpc-interfaces
WORKDIR offchain
COPY --from=planner rollups/offchain/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY ./offchain .
RUN cargo build --release --bin voucher_server_main


# RUN cargo build --release
