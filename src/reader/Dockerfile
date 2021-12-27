FROM node:12

WORKDIR reader

COPY . .

RUN yarn preinstall

RUN yarn install

ENTRYPOINT ["/bin/sh", "./entrypoint-app.sh"]

EXPOSE 4000
