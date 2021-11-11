#!/bin/sh

# addresses
SENDER_ADDRESS=0x2A20380DcA5bC24D052acfbf79ba23e988ad0050
ROLLUPS_CONTRACT_ADDRESS=0x2A20380DcA5bC24D052acfbf79ba23e988ad0050

# config files
LOGIC_CONFIG_PATH="./logic-config.toml"
SF_CONFIG_PATH="./sf-config.toml"
BS_CONFIG_PATH="./bs-config.toml"
TM_CONFIG_PATH="./tm-config.toml"
cargo run -- --sender $SENDER_ADDRESS --rollups-contract-address $ROLLUPS_CONTRACT_ADDRESS \
  --logic-config-path $LOGIC_CONFIG_PATH \
  --sf-config $SF_CONFIG_PATH \
  --bs-config $BS_CONFIG_PATH \
  --tm-config $TM_CONFIG_PATH \
