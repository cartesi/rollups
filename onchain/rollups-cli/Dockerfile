# syntax=docker.io/docker/dockerfile:1.4
FROM node:16.15.0-alpine3.15 as base

FROM base as builder

WORKDIR /app

# build app
COPY . .
RUN yarn install
RUN yarn build
RUN npm install -g .

FROM base

COPY --from=builder /usr/local/bin/cartesi-rollups /usr/local/bin/

ENTRYPOINT ["cartesi-rollups"]
