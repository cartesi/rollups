#!/bin/sh
# Change to the correct directory
cd /usr/src/app;

# Run hardhat
npx hardhat node --network hardhat &
pid=$!

npx hardhat deploy --network hardhat --export /opt/cartesi/share/blockchain/dapp.json


wait $pid


# yarn test;

# Keep node alive
# set -e
# if [ "${1#-}" != "${1}" ] || [ -z "$(command -v "${1}")" ]; then
#   set -- node "$@"
# fi
# exec "$@"
