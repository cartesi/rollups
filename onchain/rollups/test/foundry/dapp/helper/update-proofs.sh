#!/usr/bin/env bash

set -euo pipefail

# Color numbers
GREEN=32
MAGENTA=35
CYAN=36

# Step counter
STEP=1

# Number of inputs
NINPUTS=4

# Echo with color
echo2() {
    printf "\033[0;$1m"
    shift
    echo "$@"
    printf "\033[0;00m"
}

# Echo with color and step counter
echo3() {
    color=$1
    message=$2
    shift 2
    echo2 "$color" "$((STEP++)). $message" "$@"
}

# Echo an error message before exiting
failure() {
  local lineno=$1
  local msg=$2
  echo2 $MAGENTA "Failed at ${lineno}: ${msg}"
}

# Generate shell script for off-chain machine
# based on the inputs JSON file
genscript() {
    pushd ../../../../../ >/dev/null
    npx ts-node rollups/test/foundry/dapp/helper/genScript.ts
    popd >/dev/null
}

# Run the Cartesi Machine responsible for generating
# the epoch status JSON file
runmachine() {
    docker run --rm \
        --name gen-proofs \
        -v "`pwd`/gen-proofs.sh:/opt/gen-proofs/gen-proofs.sh" \
        -v "`pwd`/output:/opt/gen-proofs/output" \
        -w /opt/gen-proofs \
        cartesi/server-manager-gen-proofs:devel \
        ./gen-proofs.sh
}

# Decode strings in epoch status from Base64 to hexadecimal
b64to16() {
    python3 -m b64to16 output/finish_epoch_response_64.json > finish_epoch_response.json
}

# Generate Solidity library with proofs from epoch status
genlib() {
    pushd ../../../../../ >/dev/null
    npx ts-node rollups/test/foundry/dapp/helper/genProofLibrary.ts
    popd >/dev/null
}

# Install a trap to help debugging
trap 'failure ${LINENO} "${BASH_COMMAND}"' ERR

# Go to the helper folder
cd "${BASH_SOURCE%/*}"

# Check for command line arguments
if [ $# -ge 1 ] && [ $1 == "--setup" ]
then
    echo2 $CYAN "Setting up..."
    echo

    echo3 $GREEN "Building Docker image..."
    echo

    # Build Docker image
    docker build -t cartesi/server-manager-gen-proofs:devel .
    echo

    echo3 $GREEN "Installing Python packages..."
    echo

    # Install Python packages with pip3
    pip3 install -r requirements.txt
    echo

    echo3 $GREEN "Generating dummy inputs..."
    echo

    # Generate a dummy inputs JSON
    inputs="[]"
    sender="0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
    payload="0x"
    for (( i=0; i<$NINPUTS; i++ ))
    do
        inputs=`echo "$inputs" | jq ".[$i].sender = \"$sender\""`
        inputs=`echo "$inputs" | jq ".[$i].payload = \"$payload\""`
    done
    echo "$inputs" > inputs.json

    echo3 $GREEN "Generating script to be run on docker image..."
    echo

    genscript
    echo

    # Give execute permission to script
    chmod +x gen-proofs.sh

    echo3 $GREEN "Running docker image to generate epoch status..."
    echo

    runmachine
    echo

    echo3 $GREEN "Processing and formatting epoch status..."
    echo

    b64to16

    echo3 $GREEN "Generating Solidity library with proofs..."
    echo

    genlib
    echo

    echo2 $CYAN "All set up!"
    echo
fi

echo2 $CYAN "Updating proofs..."
echo

echo3 $GREEN "Running yarn and prettier..."
echo

yarn
echo

yarn prettier --write
echo

echo3 $GREEN "Running forge tests..."
echo

# Run the tests with forge and store the output
forge_output=`forge test -vv --match-contract CartesiDAppTest || true`

echo3 $GREEN "Processing forge output and updating inputs JSON..."
echo

# Process the forge output with awk and generate a jq filter
jq_filter=`echo "${forge_output}" | awk -f jqFilter.awk`

# Run the jq filter on inputs.json
jq_output=`jq "${jq_filter}" inputs.json`

# Update inputs.json
echo "${jq_output}" > inputs.json

echo3 $GREEN "Generating script to be run on docker image..."
echo

genscript
echo

echo3 $GREEN "Running docker image to generate epoch status..."
echo

runmachine
echo

echo3 $GREEN "Processing and formatting epoch status..."
echo

b64to16

echo3 $GREEN "Generating Solidity library with proofs..."
echo

genlib
echo

echo2 $CYAN "Proofs were updated!"
