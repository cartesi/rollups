# (c) Cartesi and individual authors (see AUTHORS)
# SPDX-License-Identifier: Apache-2.0 (see LICENSE)

# syntax=docker.io/docker/dockerfile:1.4
FROM node:16.17.0-alpine3.15 as base

# this stage installs system dependencies for building the node projects
FROM base as builder

# install git and python3
RUN <<EOF
apk update
apk add --no-cache g++ git make python3
rm -rf /var/cache/apk/*
EOF

WORKDIR /app

# copy workspace files only (package.json's and yarn.lock)
COPY package.json yarn.lock ./
COPY rollups/package.json ./rollups/
COPY rollups-cli/package.json ./rollups-cli/

# this stage copies the rollups project and build it
FROM builder as rollups-builder

# build
COPY ./rollups /app/rollups
WORKDIR /app/rollups
RUN yarn install --non-interactive
RUN yarn prepack

# this stage starts from rollups and copies rollups-cli and builds it
FROM rollups-builder as cli-builder
COPY ./rollups-cli /app/rollups-cli
WORKDIR /app/rollups-cli
RUN yarn install --non-interactive
RUN yarn build
RUN npm install -g .

# this stage is runtime image for rollups (hardhat)
FROM base as hardhat

# copy yarn build
COPY --from=rollups-builder /app /app
WORKDIR /app/rollups

# expose hardhat node port
EXPOSE 8545

ENTRYPOINT ["npx", "hardhat"]
CMD ["node"]

# this stage is runtime for rollups-cli (built with esbuild)
FROM base as cli
COPY --from=cli-builder /usr/local/bin/cartesi-rollups /usr/local/bin/
ENTRYPOINT ["cartesi-rollups"]
