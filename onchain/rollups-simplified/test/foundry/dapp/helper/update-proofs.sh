#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ge 1 ]]
then
    machine_emulator_repo=$1
    shift
else
    echo "Usage: $0 <path to local clone of machine-emulator repository>" >&2
    exit 1
fi

# Go to the helper folder
cd "${BASH_SOURCE%/*}"

# Get absolute path of helper folder
HELPER_FOLDER=`pwd`

# Create a temporary file for storing the test output
LOG_FILE=`mktemp`

echo "Updating proofs..."

echo
echo "1. Running forge tests..."

# Run the tests and pipe the output to a file
forge test -vv --match-contract CartesiDAppTest > "${LOG_FILE}" || true

# Echo an error message before exiting
failure() {
  local lineno=$1
  local msg=$2
  echo "Failed at ${lineno}: ${msg}"
}

# Install a trap to help debugging
trap 'failure ${LINENO} "${BASH_COMMAND}"' ERR

echo
echo "2. Processing logs and updating vouchers JSON..."

# Process the log file with awk and generate a jq filter
jq_filter=`awk -f jqFilter.awk -- "${LOG_FILE}"`

# Remove log file
rm "${LOG_FILE}"

# Run the jq filter on vouchers.json
jq_output=`jq "${jq_filter}" vouchers.json`

# Update vouchers.json
echo "${jq_output}" > vouchers.json

echo
echo "3. Generating script to be run on docker image..."
echo

# Generate script with vouchers
npx ts-node genScript.ts | sed 's/^/* /g'

echo
echo "4. Running docker image to generate epoch status..."

# Go to gen-proofs folder
pushd "${machine_emulator_repo}/tools/gen-proofs" >/dev/null

# Copy script to gen-proofs folder
cp "${HELPER_FOLDER}/gen-proofs.sh" gen-proofs.sh

# Run docker to generate proofs
docker run -it --rm \
    --name gen-proofs \
    -v "${PWD}/gen-proofs.sh:/opt/gen-proofs/gen-proofs.sh" \
    -v "${PWD}/output:/opt/gen-proofs/output" \
    -w /opt/gen-proofs \
    cartesi/server-manager-gen-proofs:devel \
    ./gen-proofs.sh >/dev/null

echo
echo "5. Processing epoch status and updating voucher proofs..."

# Decode strings in epoch status from Base64 to hexadecimal
# Format the output with jq so that git diffs are smoother
python3 -m b64to16 output/epoch-status.json | jq > "${HELPER_FOLDER}/voucherProofs.json"

# Go back to the helper folder
popd >/dev/null

echo
echo "6. Generating Solidity contracts for each proof..."
echo

# Generate Solidity libraries with proofs
npx ts-node genProofLibraries.ts | sed 's/^/* /g'

echo
echo "Proofs were updated!"
