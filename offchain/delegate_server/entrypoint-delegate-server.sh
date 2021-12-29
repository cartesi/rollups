#!/bin/sh

dockerize -wait file://${DEPLOYMENT_PATH} -timeout 60s

/usr/local/bin/output_server_main --sf-config $STATE_FOLD_CONFIG_PATH rollups
