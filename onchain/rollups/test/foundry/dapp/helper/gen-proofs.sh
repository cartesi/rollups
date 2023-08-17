#!/bin/bash
# (c) Cartesi and individual authors (see AUTHORS)
# SPDX-License-Identifier: Apache-2.0 (see LICENSE)

# This script must be run inside the gen-proofs docker image

set -euo pipefail

GRPC_INTERFACES_DIR=/opt/gen-proofs/grpc-interfaces
MACHINE_DIR=/tmp/gen-proofs-machine
SESSION_ID=default_session_id
INPUTS_DIR=./input
INPUT_INDEX=0

hex_to_base64() {
    xxd -r -p | base64 -w 0
}

base64_to_hex() {
    base64 -d -w 0 | xxd -p -c 64
}

get_version() {
    ./grpcurl \
        -plaintext \
        -emit-defaults \
        -import-path $GRPC_INTERFACES_DIR \
        -proto server-manager.proto \
        localhost:5001 \
        CartesiServerManager.ServerManager/GetVersion
}

wait_for_get_version_response() {
    while true
    do
        local version

        if version=`get_version`
        then
            [[ `echo "$version" | jq '.version.major'` == 0 ]]
            break
        fi
    done
}

start_session() {
    ./grpcurl \
        -plaintext \
        -import-path $GRPC_INTERFACES_DIR \
        -proto server-manager.proto \
        -d '{
                "session_id": "'$SESSION_ID'",
                "machine_directory": "'$MACHINE_DIR'",
                "active_epoch_index": 0,
                "server_cycles": {
                    "max_advance_state": 9223372036854775808,
                    "advance_state_increment": 9223372036854775808,
                    "max_inspect_state": 9223372036854775808,
                    "inspect_state_increment": 9223372036854775808
                },
                "server_deadline": {
                    "checkin": 1000000,
                    "advance_state": 1000000,
                    "advance_state_increment": 1000000,
                    "inspect_state": 1000000,
                    "inspect_state_increment": 1000000,
                    "machine": 1000000,
                    "store": 1000000,
                    "fast": 1000000
                },
                "runtime": null
            }' \
        localhost:5001 \
        CartesiServerManager.ServerManager/StartSession
}

add_input() {
    local sender=$(echo "$1" | jq -r '.sender' | hex_to_base64)
    local payload=$(echo "$1" | jq -r '.payload' | hex_to_base64)

    ./grpcurl \
        -plaintext \
        -import-path $GRPC_INTERFACES_DIR \
        -proto server-manager.proto \
        -d '{
                "session_id": "'$SESSION_ID'",
                "active_epoch_index": 0,
                "current_input_index": '$INPUT_INDEX',
                "input_payload": "'$payload'",
                "input_metadata": {
                    "msg_sender": {
                        "data": "'$sender'"
                    },
                    "block_number": 0,
                    "timestamp": 0,
                    "epoch_index": 0,
                    "input_index": '$INPUT_INDEX'
                }
            }' \
        localhost:5001 \
        CartesiServerManager.ServerManager/AdvanceState
}

add_inputs() {
    while true
    do
        local input_file="${INPUTS_DIR}/${INPUT_INDEX}.json"

        if ! [ -f "$input_file" ]
        then
            break
        fi

        local input=`cat "$input_file"`

        add_input "$input"

        INPUT_INDEX=$((INPUT_INDEX + 1))
    done
}

get_epoch_status() {
    ./grpcurl \
        -plaintext \
        -emit-defaults \
        -import-path $GRPC_INTERFACES_DIR \
        -proto server-manager.proto \
        -d '{
                "session_id": "'$SESSION_ID'",
                "epoch_index": 0
            }' \
        localhost:5001 \
        CartesiServerManager.ServerManager/GetEpochStatus
}

wait_for_inputs_to_be_processed() {
    while true
    do
        local epoch_status

        if epoch_status=`get_epoch_status`
        then
            if [ `echo "$epoch_status" | jq '.pendingInputCount'` == '"0"' ]
            then
                break
            fi
        fi
    done
}

finish_epoch() {
    ./grpcurl \
        -plaintext \
        -emit-defaults \
        -import-path $GRPC_INTERFACES_DIR \
        -proto server-manager.proto \
        -d '{
                "session_id": "'$SESSION_ID'",
                "active_epoch_index": 0,
                "processed_input_count_within_epoch": '$INPUT_INDEX',
                "storage_directory": ""
            }' \
        localhost:5001 \
        CartesiServerManager.ServerManager/FinishEpoch
}

# Start server-manager in background
server-manager --manager-address=127.0.0.1:5001 &

# Wait until server-manager is up
wait_for_get_version_response

# Start session
start_session

# Add inputs
add_inputs

# Wait for all inputs to be processed
wait_for_inputs_to_be_processed

# Activate python virtual environment
source /opt/venv/bin/activate

# Finish epoch
finish_epoch | python -m b64to16 > ./output/finish_epoch_response.json
