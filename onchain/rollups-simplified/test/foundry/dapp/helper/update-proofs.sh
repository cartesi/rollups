#!/usr/bin/env bash

set -euo pipefail

# Color numbers
GREEN=32
MAGENTA=35
CYAN=36

# Echo with color
echo2() {
    printf "\033[0;$1m"
    shift
    echo "$@"
    printf "\033[0;00m"
}

# Echo an error message before exiting
failure() {
  local lineno=$1
  local msg=$2
  echo2 $MAGENTA "Failed at ${lineno}: ${msg}"
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

    echo2 $GREEN "1. Building Docker image..."
    echo

    # Build Docker image
    docker build -t cartesi/server-manager-gen-proofs:devel .
    echo

    echo2 $GREEN "2. Installing Python packages..."
    echo

    # Install Python packages with pip3
    pip3 install -r requirements.txt
    echo
    
    echo2 $CYAN "All set up!"

    # Do not update proofs, just set up.
    exit 0
fi

echo2 $CYAN "Updating proofs..."
echo

echo2 $GREEN "1. Running forge tests..."
echo

# Run the tests with forge and store the output
forge_output=`forge test -vv --match-contract CartesiDAppTest || true`

echo2 $GREEN "2. Processing forge output and updating inputs JSON..."
echo

# Process the forge output with awk and generate a jq filter
jq_filter=`echo "${forge_output}" | awk -f jqFilter.awk`

# Run the jq filter on inputs.json
jq_output=`jq "${jq_filter}" inputs.json`

# Update inputs.json
echo "${jq_output}" > inputs.json

echo2 $GREEN "3. Generating script to be run on docker image..."
echo

# Generate script with inputs
npx ts-node genScript.ts
echo

# Give execute permission to script
chmod +x gen-proofs.sh

echo2 $GREEN "4. Running docker image to generate epoch status..."
echo

# Run docker to generate proofs
docker run -it --rm \
    --name gen-proofs \
    -v "`pwd`/gen-proofs.sh:/opt/gen-proofs/gen-proofs.sh" \
    -v "`pwd`/output:/opt/gen-proofs/output" \
    -w /opt/gen-proofs \
    cartesi/server-manager-gen-proofs:devel \
    ./gen-proofs.sh
echo

echo2 $GREEN "5. Processing and formatting epoch status..."
echo

# Decode strings in epoch status from Base64 to hexadecimal
# Format the output with jq so that git diffs are smoother
python3 -m b64to16 output/epoch-status.json | jq > epoch-status.json

echo2 $GREEN "6. Generating Solidity libraries for each output..."
echo

# Generate Solidity libraries with proofs
npx ts-node genProofLibraries.ts
echo

echo2 $CYAN "Proofs were updated!"
