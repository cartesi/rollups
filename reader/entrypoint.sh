#!/bin/sh
dockerize -wait tcp://${DB_HOST}:${DB_PORT} -timeout 60s

yarn run migrate
yarn start
