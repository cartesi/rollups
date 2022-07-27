# syntax=docker.io/docker/dockerfile:1.4

# Copyright 2022 Cartesi Pte. Ltd.
#
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software distributed
# under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
# CONDITIONS OF ANY KIND, either express or implied. See the License for the
# specific language governing permissions and limitations under the License.

# deps install
FROM rust:1.61.0-bullseye AS chef

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

WORKDIR /usr/src/app/offchain
COPY --from=planner /usr/src/app/offchain/recipe.json recipe.json
RUN <<EOF
export ARCH=$(uname -m | sed 's/aarch64/aarch_64/')
curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v3.20.1/protoc-3.20.1-linux-$ARCH.zip
unzip protoc-3.20.1-linux-$ARCH.zip -d $HOME/.local
export PATH="$PATH:$HOME/.local/bin"
cargo chef cook --release --recipe-path recipe.json
EOF

# buld application
WORKDIR /usr/src/app/offchain
COPY . /usr/src/app
RUN cargo build --release --bin inspect-server

## runtime
FROM debian:bullseye-slim AS runtime

COPY --from=builder /usr/src/app/offchain/target/release/inspect-server /usr/local/bin

ENTRYPOINT ["inspect-server"]
