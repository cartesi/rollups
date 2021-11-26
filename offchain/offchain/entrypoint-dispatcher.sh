#!/bin/sh
# addresses
SENDER_ADDRESS=0x2A20380DcA5bC24D052acfbf79ba23e988ad0050
ROLLUPS_CONTRACT_ADDRESS=0x2A20380DcA5bC24D052acfbf79ba23e988ad0050

# config files
LOGIC_CONFIG_PATH="/opt/cartesi/share/config/logic-config.toml"
SF_CONFIG_PATH="/opt/cartesi/share/config/sf-config.toml"
BS_CONFIG_PATH="/opt/cartesi/share/config/bs-config.toml"
TM_CONFIG_PATH="/opt/cartesi/share/config/tm-config.toml"

/usr/local/bin/offchain_main --sender $SENDER_ADDRESS --rollups-contract-address $ROLLUPS_CONTRACT_ADDRESS \
  --logic-config-path $LOGIC_CONFIG_PATH \
  --sf-config $SF_CONFIG_PATH \
  --bs-config $BS_CONFIG_PATH \
  --tm-config $TM_CONFIG_PATH \
