#!/bin/bash

npx hardhat node --no-deploy >> /dev/null 2>&1 &
hardhat_pid=$!
sleep 3

## testing input delegate

./offchain/target/debug/delegate_server_main >> /dev/null 2>&1 &
delegate_server_pid=$!
sleep 3

DELEGATE_TEST=1 npx hardhat test test/InputImpl.ts --network localhost

# kill input server

kill "$delegate_server_pid"

## end testing delegates

kill "$hardhat_pid"
