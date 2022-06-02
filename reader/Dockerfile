# syntax=docker.io/docker/dockerfile:1.4
FROM node:16.15.0-bullseye-slim as base

FROM base as builder

# install wget
RUN <<EOF
apt-get update
DEBIAN_FRONTEND="noninteractive" apt-get install -y wget
rm -rf /var/lib/apt/lists/*
EOF

ENV DOCKERIZE_VERSION v0.6.1
RUN <<EOF
wget https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz
tar -C /usr/local/bin -xzvf dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz
rm dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz
EOF

WORKDIR /app

# build app
COPY . .

RUN yarn preinstall
RUN yarn install --non-interactive

# runtime
FROM base

WORKDIR /app

COPY --from=builder /usr/local/bin/dockerize /usr/local/bin/dockerize
COPY entrypoint.sh /

# copy yarn build
COPY --from=builder /app .

ENTRYPOINT ["/entrypoint.sh"]

EXPOSE 4000
