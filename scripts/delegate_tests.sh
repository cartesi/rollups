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

## end testing delegates

kill "$hardhat_pid"
