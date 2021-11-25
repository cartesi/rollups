FROM node:17-buster-slim

# RUN apt update && apt install git;

RUN apt-get update && DEBIAN_FRONTEND="noninteractive" apt-get install -y \
    git \
    # libboost-coroutine1.71.0 \
    # libboost-context1.71.0 \
    # libboost-serialization1.71.0 \
    # libboost-filesystem1.71.0 \
    # libreadline8 \
    # openssl \
    # libc-ares2 \
    # zlib1g \
    # ca-certificates \
    # libgomp1 \
    # lua5.3 \
    # lua-socket \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .

RUN yarn install --non-interactive --frozen-lockfile

RUN cp ./entrypoint-docker.sh /usr/local/bin

ENTRYPOINT ["/bin/sh", "/usr/local/bin/entrypoint-docker.sh"]


# FROM node:14-alpine

# COPY . /usr/src/app

# WORKDIR /usr/src/app

# COPY package.json yarn.lock ./
# RUN apk --no-cache --virtual build-dependencies add \
#         python \
#         make \
#         g++ \
# && yarn install --production \
# && apk del build-dependencies

# COPY $PWD/entrypoint-docker.sh /usr/local/bin

# ENTRYPOINT ["/bin/sh", "/usr/local/bin/entrypoint.sh"]

