# syntax=docker.io/docker/dockerfile:1.4
FROM node:16.15.0-alpine3.15

# install git and python3
RUN <<EOF
apk update
apk add --no-cache make g++ git make python3
rm -rf /var/cache/apk/*
EOF

WORKDIR /app

# build app
COPY . .
RUN yarn install --non-interactive

# expose hardhat node port
EXPOSE 8545

ENTRYPOINT ["npx", "hardhat"]
CMD ["node"]
