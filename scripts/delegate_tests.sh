#!/bin/bash

pushd onchain/rollups >/dev/null

npx hardhat node --no-deploy >> /dev/null 2>&1 &
hardhat_pid=$!
sleep 3

popd >/dev/null

## testing input delegate

./scripts/run_input_server.sh --sf-safety-margin 0 >> /dev/null 2>&1 &
delegate_server_pid=$!
sleep 3

pushd onchain/rollups >/dev/null

DELEGATE_TEST=1 npx hardhat test test/InputFacet.ts --network localhost

popd >/dev/null

# kill input server

pkill -P "$delegate_server_pid"


## testing output delegate

./scripts/run_output_server.sh --sf-safety-margin 0 >> /dev/null 2>&1 &
delegate_server_pid=$!
sleep 3

pushd onchain/rollups >/dev/null

DELEGATE_TEST=1 npx hardhat test test/OutputFacet.ts --network localhost

popd >/dev/null

# kill output server

pkill -P "$delegate_server_pid"


# testing validator manager delegate

./scripts/run_validator_manager_server.sh --sf-safety-margin 0 >> /dev/null 2>&1 &
delegate_server_pid=$!
sleep 3

pushd onchain/rollups >/dev/null

DELEGATE_TEST=1 npx hardhat test test/ValidatorManagerFacet.ts  --network localhost

popd >/dev/null

# kill validator manager server

pkill -P "$delegate_server_pid"


## testing fee manager delegate

./scripts/run_fee_manager_server.sh --sf-safety-margin 0 >> /dev/null 2>&1 &
delegate_server_pid=$!
sleep 3

pushd onchain/rollups >/dev/null

DELEGATE_TEST=1 npx hardhat test test/FeeManagerFacet.ts --network localhost

popd >/dev/null

# kill fee manager server

pkill -P "$delegate_server_pid"


## testing rollups delegate

./scripts/run_rollups_server.sh --sf-safety-margin 0 >> /dev/null 2>&1 &
delegate_server_pid=$!
sleep 3

pushd onchain/rollups >/dev/null

DELEGATE_TEST=1 npx hardhat test test/RollupsFacet.ts --network localhost

popd >/dev/null

# kill rollups server

pkill -P "$delegate_server_pid"


## end testing delegates

kill "$hardhat_pid"
