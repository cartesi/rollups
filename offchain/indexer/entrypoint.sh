#!/bin/sh

dockerize -wait file://${DEPLOYMENT_PATH} -timeout 300s
dockerize -wait tcp://${STATE_SERVER_HOSTNAME}:${STATE_SERVER_PORT} -wait tcp://${SERVER_MANAGER_HOSTNAME}:${SERVER_MANAGER_PORT} -timeout 300s
dockerize -wait tcp://${POSTGRES_HOSTNAME}:${POSTGRES_PORT} -timeout 30s

echo "Performing postgres database migration..."
diesel migration run --database-url postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOSTNAME}:${POSTGRES_PORT}/${POSTGRES_DB}
echo "Postgres database migration finished"
RUST_LOG=${INDEXER_LOG}  /usr/local/bin/indexer_main --deployment $DEPLOYMENT_PATH  --dapp-contract-name $DAPP_CONTRACT_NAME --indexer-config-path $INDEXER_CONFIG_PATH
