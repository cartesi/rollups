#!/bin/sh

dockerize -wait file://${DEPLOYMENT_PATH} -timeout 300s

/usr/local/bin/output_server_main --sf-config $STATE_FOLD_CONFIG_PATH rollups
