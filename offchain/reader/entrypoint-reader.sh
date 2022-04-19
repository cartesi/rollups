#!/bin/bash
dockerize -wait tcp://${POSTGRES_HOSTNAME}:${POSTGRES_PORT} -timeout 60s

RUST_LOG=info  /usr/local/bin/reader --postgres-user=${POSTGRES_USER} --postgres-password=${POSTGRES_PASSWORD} --postgres-hostname=${POSTGRES_HOSTNAME} --postgres-port=${POSTGRES_PORT}  --graphql-host=${READER_GRAPHQL_HOST} --graphql-port=${READER_GRAPHQL_PORT}
