#!/bin/bash
# Copyright 2022 Cartesi Pte. Ltd.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations under
# the License.

# This script must be run inside the gen-proofs docker image

GRPC_INTERFACES_DIR=/opt/gen-proofs/grpc-interfaces
MACHINE_DIR=/tmp/gen-proofs-machine

hex_to_base64() {
    xxd -r -p | base64 -w 0
}

base64_to_hex() {
    base64 -d -w 0 | xxd -p -c 64
}

# Start server-manager in background
/opt/cartesi/bin/server-manager --manager-address=127.0.0.1:5001 &
sleep 1

# Start session
SESSION_ID=default_session_id
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

# Send first advance state
INPUT_INDEX=0
PAYLOAD=8dc21a77
MSG_SENDER=16Fdde9A2750C66Ed3B465E136ea299D92BD24Ed
./grpcurl \
    -plaintext \
    -import-path $GRPC_INTERFACES_DIR \
    -proto server-manager.proto \
    -d '{
            "session_id": "'$SESSION_ID'",
            "active_epoch_index": 0,
            "current_input_index": '$INPUT_INDEX',
            "input_payload": "'$(echo -n $PAYLOAD | hex_to_base64)'",
            "input_metadata": {
                "msg_sender": {
                    "data": "'$(echo -n $MSG_SENDER | hex_to_base64)'"
                },
                "block_number": 0,
                "timestamp": 0,
                "epoch_index": 0,
                "input_index": '$INPUT_INDEX'
            }
        }' \
    localhost:5001 \
    CartesiServerManager.ServerManager/AdvanceState
sleep 1

# Send second advance state
INPUT_INDEX=$((INPUT_INDEX + 1))
PAYLOAD=ae312c8b68656c6c6f000000000000000000000000000000000000000000000000000000
MSG_SENDER=16Fdde9A2750C66Ed3B465E136ea299D92BD24Ed
./grpcurl \
    -plaintext \
    -import-path $GRPC_INTERFACES_DIR \
    -proto server-manager.proto \
    -d '{
            "session_id": "'$SESSION_ID'",
            "active_epoch_index": 0,
            "current_input_index": '$INPUT_INDEX',
            "input_payload": "'$(echo -n $PAYLOAD | hex_to_base64)'",
            "input_metadata": {
                "msg_sender": {
                    "data": "'$(echo -n $MSG_SENDER | hex_to_base64)'"
                },
                "block_number": 0,
                "timestamp": 0,
                "epoch_index": 0,
                "input_index": '$INPUT_INDEX'
            }
        }' \
    localhost:5001 \
    CartesiServerManager.ServerManager/AdvanceState
sleep 1

# Send third advance state
INPUT_INDEX=$((INPUT_INDEX + 1))
PAYLOAD=e12ee5ab
MSG_SENDER=16Fdde9A2750C66Ed3B465E136ea299D92BD24Ed
./grpcurl \
    -plaintext \
    -import-path $GRPC_INTERFACES_DIR \
    -proto server-manager.proto \
    -d '{
            "session_id": "'$SESSION_ID'",
            "active_epoch_index": 0,
            "current_input_index": '$INPUT_INDEX',
            "input_payload": "'$(echo -n $PAYLOAD | hex_to_base64)'",
            "input_metadata": {
                "msg_sender": {
                    "data": "'$(echo -n $MSG_SENDER | hex_to_base64)'"
                },
                "block_number": 0,
                "timestamp": 0,
                "epoch_index": 0,
                "input_index": '$INPUT_INDEX'
            }
        }' \
    localhost:5001 \
    CartesiServerManager.ServerManager/AdvanceState
sleep 1

# Send fourth advance state
INPUT_INDEX=$((INPUT_INDEX + 1))
PAYLOAD=a9059cbb000000000000000000000000811085f5b5d1b29598e73ca51de3d712f5d3103a0000000000000000000000000000000000000000000000000000000000000007
MSG_SENDER=e1e96bd9f18eff2fc9029a3f650ef0b5f192240e
./grpcurl \
    -plaintext \
    -import-path $GRPC_INTERFACES_DIR \
    -proto server-manager.proto \
    -d '{
            "session_id": "'$SESSION_ID'",
            "active_epoch_index": 0,
            "current_input_index": '$INPUT_INDEX',
            "input_payload": "'$(echo -n $PAYLOAD | hex_to_base64)'",
            "input_metadata": {
                "msg_sender": {
                    "data": "'$(echo -n $MSG_SENDER | hex_to_base64)'"
                },
                "block_number": 0,
                "timestamp": 0,
                "epoch_index": 0,
                "input_index": '$INPUT_INDEX'
            }
        }' \
    localhost:5001 \
    CartesiServerManager.ServerManager/AdvanceState
sleep 1

# Send fifth advance state
INPUT_INDEX=$((INPUT_INDEX + 1))
PAYLOAD=522f6815000000000000000000000000811085f5b5d1b29598e73ca51de3d712f5d3103a0000000000000000000000000000000000000000000000000000000000000007
MSG_SENDER=fbb040e412a06ff913be35702c98479c26b069ce
./grpcurl \
    -plaintext \
    -import-path $GRPC_INTERFACES_DIR \
    -proto server-manager.proto \
    -d '{
            "session_id": "'$SESSION_ID'",
            "active_epoch_index": 0,
            "current_input_index": '$INPUT_INDEX',
            "input_payload": "'$(echo -n $PAYLOAD | hex_to_base64)'",
            "input_metadata": {
                "msg_sender": {
                    "data": "'$(echo -n $MSG_SENDER | hex_to_base64)'"
                },
                "block_number": 0,
                "timestamp": 0,
                "epoch_index": 0,
                "input_index": '$INPUT_INDEX'
            }
        }' \
    localhost:5001 \
    CartesiServerManager.ServerManager/AdvanceState
sleep 1

# Send sixth advance state
INPUT_INDEX=$((INPUT_INDEX + 1))
PAYLOAD=42842e0e000000000000000000000000fbb040e412a06ff913be35702c98479c26b069ce000000000000000000000000c1dc99f7837de1bb7fac121461e7ec955639560453dc9bf46bebdca9be947ee80674b58899973aac1948a8396714431da6d4f167
MSG_SENDER=7700fe820276be034d46d34d9f093800baab9c62
./grpcurl \
    -plaintext \
    -import-path $GRPC_INTERFACES_DIR \
    -proto server-manager.proto \
    -d '{
            "session_id": "'$SESSION_ID'",
            "active_epoch_index": 0,
            "current_input_index": '$INPUT_INDEX',
            "input_payload": "'$(echo -n $PAYLOAD | hex_to_base64)'",
            "input_metadata": {
                "msg_sender": {
                    "data": "'$(echo -n $MSG_SENDER | hex_to_base64)'"
                },
                "block_number": 0,
                "timestamp": 0,
                "epoch_index": 0,
                "input_index": '$INPUT_INDEX'
            }
        }' \
    localhost:5001 \
    CartesiServerManager.ServerManager/AdvanceState
sleep 15 # apple silicon needs more sleep time

# Finish epoch
# Exit the script if the call fails
./grpcurl \
    -plaintext \
    -import-path $GRPC_INTERFACES_DIR \
    -proto server-manager.proto \
    -d '{
            "session_id": "'$SESSION_ID'",
            "active_epoch_index": 0,
            "processed_input_count": '$((INPUT_INDEX + 1))',
            "storage_directory": ""
        }' \
    localhost:5001 \
    CartesiServerManager.ServerManager/FinishEpoch || exit 1

# Get epoch status
./grpcurl \
    -plaintext \
    -import-path $GRPC_INTERFACES_DIR \
    -proto server-manager.proto \
    -d '{
            "session_id": "'$SESSION_ID'",
            "epoch_index": 0
        }' \
    localhost:5001 \
    CartesiServerManager.ServerManager/GetEpochStatus > ./output/epoch-status.json
