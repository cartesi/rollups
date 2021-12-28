FROM node:14-buster-slim

RUN apt-get update && DEBIAN_FRONTEND="noninteractive" apt-get install -y \
    git \
    && rm -rf /var/lib/apt/lists/*

ENV BASE /opt/cartesi

WORKDIR $BASE/share/blockchain
COPY package.json .
COPY tsconfig.json .
COPY yarn.lock .

COPY grpc-interfaces ./grpc-interfaces

ADD wait-for-file.sh /
RUN chmod +x /wait-for-file.sh

COPY hardhat.config.ts .
COPY contracts ./contracts
COPY src/tasks ./src/tasks

RUN mkdir -p src/proto/

RUN yarn install --non-interactive

EXPOSE 8545

ENTRYPOINT ["npx", "hardhat"]
CMD ["node" ]
