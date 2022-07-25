# syntax=docker.io/docker/dockerfile:1.4
FROM node:16.15.0-alpine3.15 as base

FROM base as builder

# install git and python3
RUN <<EOF
apk update
apk add --no-cache g++ git make python3
rm -rf /var/cache/apk/*
EOF

WORKDIR /app

# build dependencies
COPY package.json yarn.lock ./
COPY rollups/package.json ./rollups/
COPY rollups-cli/package.json ./rollups-cli/
RUN yarn install

# build app
COPY . .
RUN yarn install

RUN cd rollups && yarn build

WORKDIR /app/rollups-cli
RUN yarn build
RUN npm install -g .

FROM base

COPY --from=builder /usr/local/bin/cartesi-rollups /usr/local/bin/

ENTRYPOINT ["cartesi-rollups"]
