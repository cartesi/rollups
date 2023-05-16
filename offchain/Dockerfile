# Copyright Cartesi Pte. Ltd.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations under
# the License.

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

# define runtime image
FROM debian:bullseye-20230502-slim as runtime
RUN addgroup --system --gid 102 cartesi && \
    adduser --system --uid 102 --ingroup cartesi --disabled-login --no-create-home --home /nonexistent --gecos "cartesi user" --shell /bin/false cartesi
RUN mkdir -p /var/opt/cartesi
RUN chown cartesi:cartesi /var/opt/cartesi
WORKDIR /var/opt/cartesi
ENV PATH="/opt/cartesi/bin:${PATH}"

## runtimes
FROM runtime AS state_server
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends ca-certificates
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=grpc_health_probe /go/bin/grpc-health-probe /usr/local/bin
COPY --from=state_server_build /usr/src/app/offchain/target/release/rollups_state_server /opt/cartesi/bin/rollups_state_server
USER cartesi
ENTRYPOINT ["/opt/cartesi/bin/rollups_state_server"]

FROM runtime AS indexer
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends libpq5
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=indexer_build /usr/src/app/offchain/target/release/indexer /opt/cartesi/bin/indexer
USER cartesi
ENTRYPOINT ["/opt/cartesi/bin/indexer"]

FROM runtime AS dispatcher
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends ca-certificates
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=dispatcher_build /usr/src/app/offchain/target/release/rollups_dispatcher /opt/cartesi/bin/rollups_dispatcher
COPY onchain/rollups/export/abi /opt/cartesi/share/deployments/
USER cartesi
ENTRYPOINT ["/opt/cartesi/bin/rollups_dispatcher"]

FROM runtime AS advance_runner
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends ca-certificates curl
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=advance_runner_build /usr/src/app/offchain/target/release/advance-runner /opt/cartesi/bin/advance-runner
USER cartesi
ENTRYPOINT ["/opt/cartesi/bin/advance-runner"]

FROM runtime AS inspect_server
COPY --from=inspect_server_build /usr/src/app/offchain/target/release/inspect-server /opt/cartesi/bin/inspect-server
USER cartesi
ENTRYPOINT ["/opt/cartesi/bin/inspect-server"]

FROM runtime AS graphql_server
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends libpq5
rm -rf /var/lib/apt/lists/*
EOF
COPY --from=graphql_server_build /usr/src/app/offchain/target/release/graphql-server /opt/cartesi/bin/graphql-server
USER cartesi
ENTRYPOINT ["/opt/cartesi/bin/graphql-server"]

FROM scratch AS deployments

WORKDIR /opt/cartesi/share/deployments
COPY onchain/rollups/export/abi .
