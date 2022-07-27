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

    # run delegate server
    "${server_script_path}" --sf-safety-margin 0 >> /dev/null 2>&1 &
    delegate_server_pid=$!
    echo "Running ${server_script_path} (pid=${delegate_server_pid})"
    sleep 3

    # run test
    pushd onchain/rollups >/dev/null
    DELEGATE_TEST=1 npx hardhat test "${test_script_path}" --network localhost
    if [ "$?" -ne "0" ]; then
        pkill -P "${delegate_server_pid}"
        kill "${hardhat_pid}"
        exit 1
    fi
    popd >/dev/null

    # kill delegate server
    pkill -P "${delegate_server_pid}"
    # in the case that delegate server did not launch successfully, exits with error code 1
    if [ "$?" -ne "0" ]; then
        kill "${hardhat_pid}"
        exit 1
    fi
done

# kill hardhat node
kill "${hardhat_pid}"
