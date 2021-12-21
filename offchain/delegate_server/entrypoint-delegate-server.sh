#!/bin/sh

dockerize -wait file:///opt/cartesi/share/blockchain/dapp.json -timeout 60s

# config files
SF_CONFIG_PATH="/opt/cartesi/share/config/sf-config.toml"

/usr/local/bin/output_server_main --sf-config $SF_CONFIG_PATH rollups
