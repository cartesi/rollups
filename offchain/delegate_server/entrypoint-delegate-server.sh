#!/bin/sh

# config files
SF_CONFIG_PATH="/opt/cartesi/share/config/sf-config.toml"

/usr/local/bin/voucher_server_main --sf-config $SF_CONFIG_PATH rollups
#ENTRYPOINT ["/usr/local/bin/voucher_server_main"]
