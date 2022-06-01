FROM node:14-buster-slim

RUN apt-get update && DEBIAN_FRONTEND="noninteractive" apt-get install -y wget && rm -rf /var/lib/apt/lists/*
ENV DOCKERIZE_VERSION v0.6.1
RUN wget https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    && tar -C /usr/local/bin -xzvf dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    && rm dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz

WORKDIR reader

COPY . .

RUN yarn preinstall

RUN yarn install

ENTRYPOINT ["/bin/sh", "./entrypoint-app.sh"]

EXPOSE 4000
