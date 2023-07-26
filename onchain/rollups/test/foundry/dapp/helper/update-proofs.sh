#!/usr/bin/env bash
# (c) Cartesi and individual authors (see AUTHORS)
# SPDX-License-Identifier: Apache-2.0 (see LICENSE)

set -euo pipefail

# Color numbers
GREEN=32
MAGENTA=35
CYAN=36

# Step counter
STEP=1

# Echo with color
echo2() {
    printf "\033[0;$1m"
    shift
    echo "$@"
    printf "\033[0;00m"
}

# Echo with color and step counter
echo3() {
    local color=$1
    local message=$2
    shift 2
    echo2 "$color" "$((STEP++)). $message" "$@"
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

    echo3 $GREEN "Building Docker image..."
    echo

    docker build -t cartesi/server-manager-gen-proofs:devel .
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

echo3 $GREEN "Building contracts..."
echo

forge build
echo

echo3 $GREEN "Generating inputs..."
echo

forge test -vv \
    --match-contract CartesiDAppTest \
    --match-test setUp > /dev/null || true

echo3 $GREEN "Running Docker image..."
echo

docker run --rm \
    --name gen-proofs \
    -v "`pwd`/gen-proofs.sh:/opt/gen-proofs/gen-proofs.sh" \
    -v "`pwd`/input:/opt/gen-proofs/input" \
    -v "`pwd`/output:/opt/gen-proofs/output" \
    -w /opt/gen-proofs \
    cartesi/server-manager-gen-proofs:devel \
    ./gen-proofs.sh

echo

echo2 $CYAN "Proofs were updated!"
