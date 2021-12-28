#!/bin/sh

dockerize -wait file://${DEPLOYMENT_PATH} -timeout 300s
dockerize -wait tcp://${STATE_SERVER_HOSTNAME}:${STATE_SERVER_PORT} -wait tcp://${SERVER_MANAGER_HOSTNAME}:${SERVER_MANAGER_PORT} -timeout 300s
dockerize -wait tcp://${POSTGRES_HOSTNAME}:${POSTGRES_PORT} -timeout 30s

PW_CONFIG_PATH="/opt/cartesi/share/config/polling-config.toml"

/usr/local/bin/indexer_main --polling-config-path $PW_CONFIG_PATH
