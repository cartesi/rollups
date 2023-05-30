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

FROM rust:1.67.0-bullseye AS chef

ENV CARGO_REGISTRIES_CARTESI_INDEX=https://github.com/cartesi/crates-index
RUN rustup component add rustfmt
RUN cargo install cargo-chef

FROM chef AS planner

COPY . /usr/src/offchain
WORKDIR /usr/src/offchain
RUN cargo chef prepare --bin rollups-events --recipe-path recipe.json

FROM chef AS builder

RUN <<EOF
DEBIAN_FRONTEND="noninteractive" apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends openssl
EOF

COPY --from=planner /usr/src/offchain/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . /usr/src/offchain
WORKDIR /usr/src/offchain/rollups-events

RUN cargo build --release --bin broker-tls-test

WORKDIR /usr/src/offchain/rollups-events/tests
RUN ./gen-test-certs.sh

FROM redis:6.2 as runtime

RUN <<EOF
DEBIAN_FRONTEND="noninteractive" apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y --no-install-recommends ca-certificates
EOF

COPY --from=builder /usr/src/offchain/target/release/broker-tls-test /usr/local/bin

COPY --from=builder /usr/src/offchain/rollups-events/tests/certs/* /data/certs/
COPY --from=builder /usr/src/offchain/rollups-events/tests/certs/ca.crt /usr/local/share/ca-certificates/
RUN update-ca-certificates

CMD ["redis-server", \
    "--tls-port 6379", \
    "--port 0", \
    "--tls-cert-file /data/certs/server.crt", \
    "--tls-key-file  /data/certs/server.key", \
    "--tls-ca-cert-file /data/certs/ca.crt", \
    "--tls-auth-clients no", \
    "--loglevel debug"]
