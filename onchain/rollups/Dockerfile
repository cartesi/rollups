FROM node:14-buster-slim

RUN apt-get update && DEBIAN_FRONTEND="noninteractive" apt-get install -y \
    git \
    && rm -rf /var/lib/apt/lists/*

ENV BASE /opt/cartesi

WORKDIR $BASE/share/blockchain
COPY onchain/package.json onchain/package.json
COPY onchain/yarn.lock onchain/yarn.lock
COPY onchain/rollups/package.json onchain/rollups/package.json
COPY onchain/rollups/tsconfig.json onchain/rollups/tsconfig.json

COPY grpc-interfaces ./grpc-interfaces

ADD onchain/rollups/wait-for-file.sh /
RUN chmod +x /wait-for-file.sh

COPY onchain/rollups/hardhat.config.ts onchain/rollups/hardhat.config.ts
COPY onchain/rollups/contracts onchain/rollups/contracts

RUN mkdir -p onchain/rollups/src/proto/

RUN yarn install --non-interactive

EXPOSE 8545

ENTRYPOINT ["npx", "hardhat"]
CMD ["node" ]
