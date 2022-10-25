# set `logVouchers` in `CartesiDApp.t.sol` to `true` before running this script
# mac users, install GNU `sed` by running:
# brew install gnu-sed; PATH="/opt/homebrew/opt/gnu-sed/libexec/gnubin:$PATH"

# active voucher array
vouchers=( "voucher 3" "voucher 4" "voucher 5" )
# keywords to find and replace in gen-proofs.sh
keywords=( "fourth" "fifth" "sixth" )
# get length of array
length=${#vouchers[@]}

HELPER_FOLDER=`pwd` # home folder
PATH_TO_REPOS=../../../../../../../
PATH_TO_GEN_PROOFS=../../../../../../../machine-emulator/tools/gen-proofs

forge test -vv --match-contract CartesiDAppTest > logs

# exit when any command fails (including when no proof updates are needed)
set -e -euo pipefail
# echo an error message before exiting
failure() {
  local lineno=$1
  local msg=$2
  echo "Failed at $lineno: $msg"
  # clean
  rm logs
}
trap 'failure ${LINENO} "$BASH_COMMAND"' ERR

i=0
while [ $i -lt $length ]
do
    # search for vouchers in logs
    # get the line after voucher, which is the value of address
    address=`grep -A 3 "${vouchers[$i]}" logs | sed -n '2'p`
    # remove 2 spaces and '0x'
    address=${address:4}
    # get the line after address, which is the payload
    payload=`grep -A 3 "${vouchers[$i]}" logs | sed -n '3'p`
    # remove 2 spaces and '0x'
    payload=${payload:4}

    # replace new values
    sed -i -e "/${keywords[$i]}/{n;n;s/.*/PAYLOAD=$payload/}" dup-gen-proofs.sh
    sed -i -e "/${keywords[$i]}/{n;n;n;s/.*/MSG_SENDER=$address/}" dup-gen-proofs.sh

    i=$(( $i + 1 ))
done

# check if gen-proofs.sh in repo machine-emulator exists
GEN_PROOFS=$PATH_TO_GEN_PROOFS/gen-proofs.sh
if ! [ -f "$GEN_PROOFS" ]; then # if not exists
    # clone repo to the same root folder as `rollups`
    pushd $PATH_TO_REPOS
    git clone https://github.com/cartesi-corp/machine-emulator.git
    cd machine-emulator
    git checkout feature/gen-proofs
    cd tools/gen-proofs
    # build docker image
    docker build -t cartesi/server-manager-gen-proofs:devel .
    # back to the helper folder
    popd
fi

# replace GEN_PROOFS with updated address and payload
cp dup-gen-proofs.sh $GEN_PROOFS

# run docker to generate proofs
pushd $PATH_TO_GEN_PROOFS
docker run -it --rm \
    --name gen-proofs \
    -v $PWD/gen-proofs.sh:/opt/gen-proofs/gen-proofs.sh \
    -v $PWD/output:/opt/gen-proofs/output \
    -w /opt/gen-proofs \
    cartesi/server-manager-gen-proofs:devel \
    ./gen-proofs.sh

# decode from base64 to 16
# check if decoder exists. If not, install
pip3 list | grep base64-to-hex-converter
if [ $? -gt 0 ]; then
    pip3 install base64-to-hex-converter
fi
python3 -m b64to16 output/epoch-status.json > $HELPER_FOLDER/voucherProofs.json

# back to the helper folder
popd

# generate Solidity version of proofs
npx ts-node genProof.ts

# clean
rm logs
