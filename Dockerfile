FROM node:14-buster-slim
RUN apt-get update && DEBIAN_FRONTEND="noninteractive" apt-get install -y \
    git \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /opt/cartesi/share/blockchain/

WORKDIR /usr/src/app
COPY . .

RUN yarn install --non-interactive --frozen-lockfile

RUN cp ./entrypoint-docker.sh /usr/local/bin

ENTRYPOINT ["/bin/sh", "/usr/local/bin/entrypoint-docker.sh"]
