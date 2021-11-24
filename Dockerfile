FROM node:14

RUN npm install -g node-pre-gyp

ENV APP_ROOT /app

RUN mkdir ${APP_ROOT}
WORKDIR ${APP_ROOT}
ADD . ${APP_ROOT}

RUN yarn install

EXPOSE 8545

CMD [ "yarn test"]
