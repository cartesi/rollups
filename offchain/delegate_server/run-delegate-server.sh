#!/bin/sh

# config files
SF_CONFIG_PATH="../offchain/sf-config.toml"

cargo run --bin output_server_main -- --sf-config $SF_CONFIG_PATH rollups
