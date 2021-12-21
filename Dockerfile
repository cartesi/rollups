FROM node:16-alpine

RUN apk add --no-cache git

ENV BASE /opt/cartesi

WORKDIR $BASE/share/blockchain
COPY package.json .
COPY tsconfig.json .
COPY yarn.lock .

ADD wait-for-file.sh /
RUN chmod +x /wait-for-file.sh

COPY hardhat.config.ts .
COPY contracts ./contracts
COPY src/tasks ./src/tasks

RUN yarn install --non-interactive

EXPOSE 8545

ENTRYPOINT ["npx", "hardhat"]
CMD ["node" ]
