#!/bin/bash

npx hardhat node --no-deploy >> /dev/null 2>&1 &
hardhat_pid=$!
sleep 3

## testing input delegate

./scripts/run_input_server.sh --sf-safety-margin 0 >> /dev/null 2>&1 &
delegate_server_pid=$!
sleep 3

DELEGATE_TEST=1 npx hardhat test test/InputImpl.ts --network localhost

# kill input server

pkill -P "$delegate_server_pid"


## testing output delegate

./scripts/run_output_server.sh --sf-safety-margin 0 >> /dev/null 2>&1 &
delegate_server_pid=$!
sleep 3

DELEGATE_TEST=1 npx hardhat test test/OutputImpl.ts --network localhost


# kill output server

pkill -P "$delegate_server_pid"


## testing fee manager delegate

./scripts/run_fee_manager_server.sh --sf-safety-margin 0 >> /dev/null 2>&1 &
delegate_server_pid=$!
sleep 3

DELEGATE_TEST=1 npx hardhat test test/FeeManagerImpl.ts --network localhost


# kill fee manager server

pkill -P "$delegate_server_pid"


## testing rollups delegate

./scripts/run_rollups_server.sh --sf-safety-margin 0 >> /dev/null 2>&1 &
delegate_server_pid=$!
sleep 3

DELEGATE_TEST=1 npx hardhat test test/RollupsImpl.ts --network localhost


# kill rollups server

pkill -P "$delegate_server_pid"


## end testing delegates

kill "$hardhat_pid"
