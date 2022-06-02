#!/bin/sh

# addresses
dockerize -wait file://${DEPLOYMENT_PATH} -timeout 300s

# wait for services
dockerize -wait tcp://${STATE_SERVER_HOSTNAME}:${STATE_SERVER_PORT} \
          -wait tcp://${SERVER_MANAGER_HOSTNAME}:${SERVER_MANAGER_PORT} \
          -timeout 300s

/usr/local/bin/offchain_main "$@"
