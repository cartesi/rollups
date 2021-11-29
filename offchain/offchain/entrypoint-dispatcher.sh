#!/bin/sh
# addresses
ROLLUPS_CONTRACT_ADDRESS="0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9"

# config files
LOGIC_CONFIG_PATH="/opt/cartesi/share/config/logic-config.toml"
SF_CONFIG_PATH="/opt/cartesi/share/config/sf-config.toml"
BS_CONFIG_PATH="/opt/cartesi/share/config/bs-config.toml"
TM_CONFIG_PATH="/opt/cartesi/share/config/tm-config.toml"

#while ! test -f "/tmp/$1"; do
#  sleep 10
#  echo "Still waiting"
#done
sleep 40

/usr/local/bin/offchain_main --rollups-contract-address $ROLLUPS_CONTRACT_ADDRESS \
  --logic-config-path $LOGIC_CONFIG_PATH \
  --sf-config $SF_CONFIG_PATH \
  --bs-config $BS_CONFIG_PATH \
  --tm-config $TM_CONFIG_PATH \
