#!/bin/sh

# config files
SF_CONFIG_PATH="/opt/cartesi/share/config/sf-config.toml"

sleep 40

/usr/local/bin/output_server_main --sf-config $SF_CONFIG_PATH rollups
#ENTRYPOINT ["/usr/local/bin/voucher_server_main"]
