#!/bin/bash

# run hardhat node
pushd onchain/rollups >/dev/null
npx hardhat node --no-deploy >> /dev/null 2>&1 &
hardhat_pid=$!
echo "Running hardhat node (pid=${hardhat_pid})"
sleep 15
popd >/dev/null

tests=(
    "input;InputFacet"
    "output;OutputFacet"
    "validator_manager;ValidatorManagerFacet"
    "fee_manager;FeeManagerFacet"
    "rollups;RollupsFacet"
)

for test in "${tests[@]}"
do
    # turn e.g. 'input;InputFacet' into
    # array ['input', 'InputFacet']
    IFS=";" read -r -a arr <<< "${test}"

    server_script_name="${arr[0]}"
    test_script_name="${arr[1]}"

    server_script_path="./scripts/run_${server_script_name}_server.sh"
    test_script_path="test/${test_script_name}.ts"

    # run state server
    "${server_script_path}" --sf-safety-margin 0 >> /dev/null 2>&1 &
    state_server_pid=$!
    echo "Running ${server_script_path} (pid=${state_server_pid})"
    sleep 3

    # run test
    pushd onchain/rollups >/dev/null
    STATE_FOLD_TEST=1 npx hardhat test "${test_script_path}" --network localhost
    if [ "$?" -ne "0" ]; then
        pkill -P "${state_server_pid}"
        kill "${hardhat_pid}"
        exit 1
    fi
    popd >/dev/null

    # kill state server
    pkill -P "${state_server_pid}"
    # in the case that state server did not launch successfully, exits with error code 1
    if [ "$?" -ne "0" ]; then
        kill "${hardhat_pid}"
        exit 1
    fi
done

# kill hardhat node
kill "${hardhat_pid}"
