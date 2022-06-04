# syntax=docker.io/docker/dockerfile:1.4
FROM node:16.15.0-bullseye-slim as base

FROM base as builder

WORKDIR /app

# build app
COPY . .

RUN yarn preinstall
RUN yarn install --non-interactive

# runtime
FROM base

WORKDIR /app

# copy yarn build
COPY --from=builder /app .

CMD ["yarn run migrate && yarn start"]

EXPOSE 4000
