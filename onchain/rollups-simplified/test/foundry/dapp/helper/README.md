# Updating Proofs

## Dependencies

* Docker
* Python 3.8 or newer
* jq

## Setup

1. Clone the `machine-emulator` repository anywhere you want. In this example, we'll clone it in `~/Cartesi/`.

```sh
cd ~/Cartesi/
git clone https://github.com/cartesi-corp/machine-emulator.git
```

2. Check out to the `feature/gen-proofs` branch

```sh
cd machine-emulator
git checkout feature/gen-proofs
```

3. Build the `gen-proofs` Docker image

```sh
docker build -t cartesi/server-manager-gen-proofs:devel .
```

4. Install the following Python package

```sh
pip3 install base64-to-hex-converter
```

5. (Mac users) Install GNU sed

```sh
brew install gnu-sed
PATH="/opt/homebrew/opt/gnu-sed/libexec/gnubin:$PATH"
```

## Procedure

Now, everytime you think the proofs might need to be updated, just run the following command.
Feel free to change the path to the `machine-emulator` repository in the command if you haven't cloned it in `~/Cartesi/`.

```sh
./updateProofs.sh ~/Cartesi/machine-emulator/tools/gen-proofs
```
