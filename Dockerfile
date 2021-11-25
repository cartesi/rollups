FROM node:14-alpine

COPY . /usr/src/app

WORKDIR /usr/src/app

RUN apk add git;

RUN yarn install --non-interactive --frozen-lockfile

COPY $PWD/entrypoint-docker.sh /usr/local/bin

ENTRYPOINT ["/bin/sh", "/usr/local/bin/entrypoint.sh"]


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

