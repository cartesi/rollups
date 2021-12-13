#!/bin/sh

# addresses
dockerize -wait file:///opt/cartesi/share/blockchain/localhost.json -timeout 60s
ROLLUPS_CONTRACT_ADDRESS=$(jq -r ".contracts.RollupsImpl.address" /opt/cartesi/share/blockchain/localhost.json)

# wait for statefold_server
dockerize -wait tcp://statefold_server:50051 -timeout 30s

# config files
LOGIC_CONFIG_PATH="/opt/cartesi/share/config/logic-config.toml"
SF_CONFIG_PATH="/opt/cartesi/share/config/sf-config.toml"
BS_CONFIG_PATH="/opt/cartesi/share/config/bs-config.toml"
TM_CONFIG_PATH="/opt/cartesi/share/config/tm-config.toml"

/usr/local/bin/offchain_main --rollups-contract-address $ROLLUPS_CONTRACT_ADDRESS \
  --logic-config-path $LOGIC_CONFIG_PATH \
  --sf-config $SF_CONFIG_PATH \
  --bs-config $BS_CONFIG_PATH \
  --tm-config $TM_CONFIG_PATH \
