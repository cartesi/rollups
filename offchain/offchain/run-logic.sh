#!/bin/sh

# addresses
SENDER_ADDRESS=0x2A20380DcA5bC24D052acfbf79ba23e988ad0050
DESCARTES_CONTRACT_ADDRESS=0x2A20380DcA5bC24D052acfbf79ba23e988ad0050

# config files
LOGIC_CONFIG_PATH="./logic-config.toml"
SF_CONFIG_PATH="./sf-config.toml"
BS_CONFIG_PATH="./bs-config.toml"
TM_CONFIG_PATH="./tm-config.toml"
cargo run -- --sender $SENDER_ADDRESS --descartes-contract-address $DESCARTES_CONTRACT_ADDRESS \
  --logic-config-path $LOGIC_CONFIG_PATH \
  --sf-config $SF_CONFIG_PATH \
  --bs-config $BS_CONFIG_PATH \
  --tm-config $TM_CONFIG_PATH \

# --descartes-contract-address <descartes-contract-address>
#            Address of deployed descartes contract [env: DESCARTES_CONTRACT_ADDRESS=]
#
#        --gas-multiplier <gas-multiplier>                               Tx gas multiplier [env: GAS_MULTIPLIER=]
#        --gas-price-multiplier <gas-price-multiplier>
#            Tx gas price multiplier [env: GAS_PRICE_MULTIPLIER=]
#
#        --initial-epoch <initial-epoch>
#            Initial epoch of state fold indexing [env: INITIAL_EPOCH=]
#
#        --logic-config-path <logic-config-path>
#            Path to logic .toml config [env: LOGIC_CONFIG_PATH=]
#
#        --mm-endpoint <mm-endpoint>
#            URL of rollups machine manager gRPC endpoint [env: MM_ENDPOINT=]
#
#        --rate <rate>                                                   Tx resubmit rate (for Tx Manager) [env: RATE=]
#        --sender <sender>                                               Signer address [env: SENDER=]
#        --session-id <session-id>
#            Session ID for rollups machine manager [env: SESSION_ID=]
#
#        --sf-concurrent-events-fetch <sf-concurrent-events-fetch>
#            Concurrent events fetch for state fold access [env: SF_CONCURRENT_EVENTS_FETCH=]
#
#        --sf-config <sf-config>
#            Path to state fold .toml config [env: SF_CONFIG=]
#
#        --sf-genesis-block <sf-genesis-block>
#            Genesis block number for state fold access [env: SF_GENESIS_BLOCK=]
#
#        --sf-query-limit-error-codes <sf-query-limit-error-codes>...
#            Query limit error codes for state fold access [env: SF_QUERY_LIMIT_ERROR_CODES=]
#
#        --sf-safety-margin <sf-safety-margin>
#            Safety margin for state fold [env: SF_SAFETY_MARGIN=]
#
#        --signer-http-endpoint <signer-http-endpoint>
#            URL of transaction signer http endpoint [env: SIGNER_HTTP_ENDPOINT=]
#
#        --state-fold-grpc-endpoint <state-fold-grpc-endpoint>
#            URL of state fold server gRPC endpoint [env: STATE_FOLD_GRPC_ENDPOINT=]
#
#        --tm-config <tm-config>
#            Path to transaction manager .toml config [env: TM_CONFIG=]
#
#        --tm-max-delay <tm-max-delay>
#            Max delay (secs) between retries [env: TM_MAX_DELAY=]
#
#        --tm-max-retries <tm-max-retries>
#            Max retries for a transaction [env: TM_MAX_RETRIES=]
#
#        --tm-timeout <tm-timeout>
#            Timeout value (secs) for a transaction [env: TM_TIMEOUT=]
#
#    -u, --url <url>                                                     Provider http endpoint [env: URL=]
#        --ws-endpoint <ws-endpoint>
#            URL of websocket provider endpoint [env: WS_ENDPOINT=]
#
#        --ws-url <ws-url>                                               Provider websocket endpoint [env: WS_URL=]
#➜  offchain git:(feature/create-run-script) ✗ nvim run-logic.sh
#fish: Job 1, 'nvim run-logic.sh' has stopped
#➜  offchain git:(feature/create-run-script) ✗ ./run-logic.sh
#   Compiling offchain v0.1.0 (/Users/felipeargento/Cartesi/descartes-v2/offchain/offchain)
#   Compiling tx-manager v0.1.0 (https://github.com/cartesi/tx-manager?rev=29a1abf#29a1abf5)
#   Compiling state-fold v0.1.0 (https://github.com/cartesi/state-fold?rev=64f9c0e#64f9c0ee)
#warning: unused import: `state_fold`
# --> offchain/src/error.rs:4:5
#  |
#4 | use state_fold;
#  |     ^^^^^^^^^^
#  |
#  = note: `#[warn(unused_imports)]` on by default
#
#warning: unused imports: `Address`, `U64`
# --> offchain/src/logic/instantiate_state_fold.rs:9:27
#  |
#9 | use ethers::core::types::{Address, U64};
#  |                           ^^^^^^^  ^^^
#
#warning: `offchain` (lib) generated 2 warnings
#    Finished dev [unoptimized + debuginfo] target(s) in 5.88s
#     Running `/Users/felipeargento/Cartesi/descartes-v2/offchain/target/debug/offchain --sender help`
#Error: BadConfiguration { err: "Fail to initialize logic config: File error: Sender address string ill-formed: Invalid character 'h' at position 0" }
#➜  offchain git:(feature/create-run-script) ✗ fg
#Send job 1, 'nvim run-logic.sh' to foreground
#fish: Job 1, 'nvim run-logic.sh' has stopped
#➜  offchain git:(feature/create-run-script) ✗ ./run-logic.sh
#   Compiling offchain v0.1.0 (/Users/felipeargento/Cartesi/descartes-v2/offchain/offchain)
#   Compiling tx-manager v0.1.0 (https://github.com/cartesi/tx-manager?rev=29a1abf#29a1abf5)
#   Compiling state-fold v0.1.0 (https://github.com/cartesi/state-fold?rev=64f9c0e#64f9c0ee)
#warning: unused import: `state_fold`
# --> offchain/src/error.rs:4:5
#  |
#4 | use state_fold;
#  |     ^^^^^^^^^^
#  |
#  = note: `#[warn(unused_imports)]` on by default
#
#warning: unused imports: `Address`, `U64`
# --> offchain/src/logic/instantiate_state_fold.rs:9:27
#  |
#9 | use ethers::core::types::{Address, U64};
#  |                           ^^^^^^^  ^^^
#
#warning: `offchain` (lib) generated 2 warnings
#    Finished dev [unoptimized + debuginfo] target(s) in 5.69s
#     Running `/Users/felipeargento/Cartesi/descartes-v2/offchain/target/debug/offchain --sender 7b036bb43a02259ed60dfa7fbf098bd3da16d1195a3b526cb3273ce65c06cc94`
#Error: BadConfiguration { err: "Fail to initialize logic config: File error: Sender address string ill-formed: Invalid input length" }
#➜  offchain git:(feature/create-run-script) ✗ fg
#Send job 1, 'nvim run-logic.sh' to foreground
#fish: Job 1, 'nvim run-logic.sh' has stopped
#➜  offchain git:(feature/create-run-script) ✗ ./run-logic.sh
#   Compiling offchain v0.1.0 (/Users/felipeargento/Cartesi/descartes-v2/offchain/offchain)
#   Compiling tx-manager v0.1.0 (https://github.com/cartesi/tx-manager?rev=29a1abf#29a1abf5)
#   Compiling state-fold v0.1.0 (https://github.com/cartesi/state-fold?rev=64f9c0e#64f9c0ee)
#warning: unused import: `state_fold`
# --> offchain/src/error.rs:4:5
#  |
#4 | use state_fold;
#  |     ^^^^^^^^^^
#  |
#  = note: `#[warn(unused_imports)]` on by default
#
#warning: unused imports: `Address`, `U64`
# --> offchain/src/logic/instantiate_state_fold.rs:9:27
#  |
#9 | use ethers::core::types::{Address, U64};
#  |                           ^^^^^^^  ^^^
#
#warning: `offchain` (lib) generated 2 warnings
#    Finished dev [unoptimized + debuginfo] target(s) in 5.66s
#     Running `/Users/felipeargento/Cartesi/descartes-v2/offchain/target/debug/offchain --sender 0x7b036bb43a02259ed60dfa7fbf098bd3da16d1195a3b526cb3273ce65c06cc94`
#Error: BadConfiguration { err: "Fail to initialize logic config: File error: Sender address string ill-formed: Invalid input length" }
#➜  offchain git:(feature/create-run-script) ✗ fg
#Send job 1, 'nvim run-logic.sh' to foreground
#fish: Job 1, 'nvim run-logic.sh' has stopped
#➜  offchain git:(feature/create-run-script) ✗ ./run-logic.sh
#   Compiling offchain v0.1.0 (/Users/felipeargento/Cartesi/descartes-v2/offchain/offchain)
#   Compiling tx-manager v0.1.0 (https://github.com/cartesi/tx-manager?rev=29a1abf#29a1abf5)
#   Compiling state-fold v0.1.0 (https://github.com/cartesi/state-fold?rev=64f9c0e#64f9c0ee)
#warning: unused import: `state_fold`
# --> offchain/src/error.rs:4:5
#  |
#4 | use state_fold;
#  |     ^^^^^^^^^^
#  |
#  = note: `#[warn(unused_imports)]` on by default
#
#warning: unused imports: `Address`, `U64`
# --> offchain/src/logic/instantiate_state_fold.rs:9:27
#  |
#9 | use ethers::core::types::{Address, U64};
#  |                           ^^^^^^^  ^^^
#
#warning: `offchain` (lib) generated 2 warnings
#    Finished dev [unoptimized + debuginfo] target(s) in 5.74s
#     Running `/Users/felipeargento/Cartesi/descartes-v2/offchain/target/debug/offchain --sender 7b036bb43a02259ed60dfa7fbf098bd3da16d1195a3b526cb3273ce65c06cc94`
#Error: BadConfiguration { err: "Fail to initialize logic config: File error: Sender address string ill-formed: Invalid input length" }
#➜  offchain git:(feature/create-run-script) ✗ fg
#Send job 1, 'nvim run-logic.sh' to foreground
#fish: Job 1, 'nvim run-logic.sh' has stopped
#➜  offchain git:(feature/create-run-script) ✗ ./run-logic.sh
#   Compiling offchain v0.1.0 (/Users/felipeargento/Cartesi/descartes-v2/offchain/offchain)
#   Compiling tx-manager v0.1.0 (https://github.com/cartesi/tx-manager?rev=29a1abf#29a1abf5)
#   Compiling state-fold v0.1.0 (https://github.com/cartesi/state-fold?rev=64f9c0e#64f9c0ee)
#warning: unused import: `state_fold`
# --> offchain/src/error.rs:4:5
#  |
#4 | use state_fold;
#  |     ^^^^^^^^^^
#  |
#  = note: `#[warn(unused_imports)]` on by default
#
#warning: unused imports: `Address`, `U64`
# --> offchain/src/logic/instantiate_state_fold.rs:9:27
#  |
#9 | use ethers::core::types::{Address, U64};
#  |                           ^^^^^^^  ^^^
#
#warning: `offchain` (lib) generated 2 warnings
#    Finished dev [unoptimized + debuginfo] target(s) in 7.26s
#     Running `/Users/felipeargento/Cartesi/descartes-v2/offchain/target/debug/offchain --sender 0x2A20380DcA5bC24D052acfbf79ba23e988ad0050`
#Error: BadConfiguration { err: "Fail to initialize logic config: File error: Must specify rollups contract address" }
#➜  offchain git:(feature/create-run-script) ✗ fg
#Send job 1, 'nvim run-logic.sh' to foreground
#fish: Job 1, 'nvim run-logic.sh' has stopped
#➜  offchain git:(feature/create-run-script) ✗           Address of deployed descartes contract [env: DESCARTES_CONTRACT_ADDRESS=]
#
#        --gas-multiplier <gas-multiplier>                               Tx gas multiplier [env: GAS_MULTIPLIER=]
#        --gas-price-multiplier <gas-price-multiplier>
#            Tx gas price multiplier [env: GAS_PRICE_MULTIPLIER=]
#
#        --initial-epoch <initial-epoch>
#            Initial epoch of state fold indexing [env: INITIAL_EPOCH=]
#
#        --logic-config-path <logic-config-path>
#            Path to logic .toml config [env: LOGIC_CONFIG_PATH=]
#
#        --mm-endpoint <mm-endpoint>
#            URL of rollups machine manager gRPC endpoint [env: MM_ENDPOINT=]
#
#        --rate <rate>                                                   Tx resubmit rate (for Tx Manager) [env: RATE=]
#        --sender <sender>                                               Signer address [env: SENDER=]
#        --session-id <session-id>
#            Session ID for rollups machine manager [env: SESSION_ID=]
#
#        --sf-concurrent-events-fetch <sf-concurrent-events-fetch>
#            Concurrent events fetch for state fold access [env: SF_CONCURRENT_EVENTS_FETCH=]
#
#        --sf-config <sf-config>
#            Path to state fold .toml config [env: SF_CONFIG=]
#
#        --sf-genesis-block <sf-genesis-block>
#            Genesis block number for state fold access [env: SF_GENESIS_BLOCK=]
#
#        --sf-query-limit-error-codes <sf-query-limit-error-codes>...
#            Query limit error codes for state fold access [env: SF_QUERY_LIMIT_ERROR_CODES=]
#
#        --sf-safety-margin <sf-safety-margin>
#            Safety margin for state fold [env: SF_SAFETY_MARGIN=]
#
#        --signer-http-endpoint <signer-http-endpoint>
#            URL of transaction signer http endpoint [env: SIGNER_HTTP_ENDPOINT=]
#
#        --state-fold-grpc-endpoint <state-fold-grpc-endpoint>
#            URL of state fold server gRPC endpoint [env: STATE_FOLD_GRPC_ENDPOINT=]
#
#        --tm-config <tm-config>
#            Path to transaction manager .toml config [env: TM_CONFIG=]
#
#        --tm-max-delay <tm-max-delay>
#            Max delay (secs) between retries [env: TM_MAX_DELAY=]
#
#        --tm-max-retries <tm-max-retries>
#            Max retries for a transaction [env: TM_MAX_RETRIES=]
#
#        --tm-timeout <tm-timeout>
#            Timeout value (secs) for a transaction [env: TM_TIMEOUT=]
#
#    -u, --url <url>                                                     Provider http endpoint [env: URL=]
#        --ws-endpoint <ws-endpoint>
#            URL of websocket provider endpoint [env: WS_ENDPOINT=]
#
#        --ws-url <ws-url>                                               Provider websocket endpoint [env: WS_URL=]
#?  offchain git:(feature/create-run-script) ? nvim run-logic.sh
#fish: Job 1, 'nvim run-logic.sh' has stopped
#?  offchain git:(feature/create-run-script) ? ./run-logic.sh
#   Compiling offchain v0.1.0 (/Users/felipeargento/Cartesi/descartes-v2/offchain/offchain)
#   Compiling tx-manager v0.1.0 (https://github.com/cartesi/tx-manager?rev=29a1abf#29a1abf5)
#   Compiling state-fold v0.1.0 (https://github.com/cartesi/state-fold?rev=64f9c0e#64f9c0ee)
#warning: unused import: `state_fold`
# --> offchain/src/error.rs:4:5
#  |
#4 | use state_fold;
#  |     ^^^^^^^^^^
#  |
#  = note: `#[warn(unused_imports)]` on by default
#
#warning: unused imports: `Address`, `U64`
# --> offchain/src/logic/instantiate_state_fold.rs:9:27
#  |
#9 | use ethers::core::types::{Address, U64};
#  |                           ^^^^^^^  ^^^
#
#warning: `offchain` (lib) generated 2 warnings
#    Finished dev [unoptimized + debuginfo] target(s) in 5.88s
#     Running `/Users/felipeargento/Cartesi/descartes-v2/offchain/target/debug/offchain --sender help`
#Error: BadConfiguration { err: "Fail to initialize logic config: File error: Sender address string ill-formed: Invalid character 'h' at position 0" }
#?  offchain git:(feature/create-run-script) ? fg
#Send job 1, 'nvim run-logic.sh' to foreground
#fish: Job 1, 'nvim run-logic.sh' has stopped
#?  offchain git:(feature/create-run-script) ? ./run-logic.sh
#   Compiling offchain v0.1.0 (/Users/felipeargento/Cartesi/descartes-v2/offchain/offchain)
#   Compiling tx-manager v0.1.0 (https://github.com/cartesi/tx-manager?rev=29a1abf#29a1abf5)
#   Compiling state-fold v0.1.0 (https://github.com/cartesi/state-fold?rev=64f9c0e#64f9c0ee)
#warning: unused import: `state_fold`
# --> offchain/src/error.rs:4:5
#  |
#4 | use state_fold;
#  |     ^^^^^^^^^^
#  |
#  = note: `#[warn(unused_imports)]` on by default
#
#warning: unused imports: `Address`, `U64`
# --> offchain/src/logic/instantiate_state_fold.rs:9:27
#  |
#9 | use ethers::core::types::{Address, U64};
#  |                           ^^^^^^^  ^^^
#
#warning: `offchain` (lib) generated 2 warnings
#    Finished dev [unoptimized + debuginfo] target(s) in 5.69s
#     Running `/Users/felipeargento/Cartesi/descartes-v2/offchain/target/debug/offchain --sender 7b036bb43a02259ed60dfa7fbf098bd3da16d1195a3b526cb3273ce65c06cc94`
#Error: BadConfiguration { err: "Fail to initialize logic config: File error: Sender address string ill-formed: Invalid input length" }
#?  offchain git:(feature/create-run-script) ? fg
#Send job 1, 'nvim run-logic.sh' to foreground
#fish: Job 1, 'nvim run-logic.sh' has stopped
#?  offchain git:(feature/create-run-script) ? ./run-logic.sh
#   Compiling offchain v0.1.0 (/Users/felipeargento/Cartesi/descartes-v2/offchain/offchain)
#   Compiling tx-manager v0.1.0 (https://github.com/cartesi/tx-manager?rev=29a1abf#29a1abf5)
#   Compiling state-fold v0.1.0 (https://github.com/cartesi/state-fold?rev=64f9c0e#64f9c0ee)
#warning: unused import: `state_fold`
# --> offchain/src/error.rs:4:5
#  |
#4 | use state_fold;
#  |     ^^^^^^^^^^
#  |
#  = note: `#[warn(unused_imports)]` on by default
#
#warning: unused imports: `Address`, `U64`
# --> offchain/src/logic/instantiate_state_fold.rs:9:27
#  |
#9 | use ethers::core::types::{Address, U64};
#  |                           ^^^^^^^  ^^^
#
#warning: `offchain` (lib) generated 2 warnings
#    Finished dev [unoptimized + debuginfo] target(s) in 5.66s
#     Running `/Users/felipeargento/Cartesi/descartes-v2/offchain/target/debug/offchain --sender 0x7b036bb43a02259ed60dfa7fbf098bd3da16d1195a3b526cb3273ce65c06cc94`
#Error: BadConfiguration { err: "Fail to initialize logic config: File error: Sender address string ill-formed: Invalid input length" }
#?  offchain git:(feature/create-run-script) ? fg
#Send job 1, 'nvim run-logic.sh' to foreground
#fish: Job 1, 'nvim run-logic.sh' has stopped
#?  offchain git:(feature/create-run-script) ? ./run-logic.sh
#   Compiling offchain v0.1.0 (/Users/felipeargento/Cartesi/descartes-v2/offchain/offchain)
#   Compiling tx-manager v0.1.0 (https://github.com/cartesi/tx-manager?rev=29a1abf#29a1abf5)
#   Compiling state-fold v0.1.0 (https://github.com/cartesi/state-fold?rev=64f9c0e#64f9c0ee)
#warning: unused import: `state_fold`
# --> offchain/src/error.rs:4:5
#  |
#4 | use state_fold;
#  |     ^^^^^^^^^^
#  |
#  = note: `#[warn(unused_imports)]` on by default
#
#warning: unused imports: `Address`, `U64`
# --> offchain/src/logic/instantiate_state_fold.rs:9:27
#  |
#9 | use ethers::core::types::{Address, U64};
#  |                           ^^^^^^^  ^^^
#
#warning: `offchain` (lib) generated 2 warnings
#    Finished dev [unoptimized + debuginfo] target(s) in 5.74s
#     Running `/Users/felipeargento/Cartesi/descartes-v2/offchain/target/debug/offchain --sender 7b036bb43a02259ed60dfa7fbf098bd3da16d1195a3b526cb3273ce65c06cc94`
#Error: BadConfiguration { err: "Fail to initialize logic config: File error: Sender address string ill-formed: Invalid input length" }
#?  offchain git:(feature/create-run-script) ? fg
#Send job 1, 'nvim run-logic.sh' to foreground
#fish: Job 1, 'nvim run-logic.sh' has stopped
#?  offchain git:(feature/create-run-script) ? ./run-logic.sh
#   Compiling offchain v0.1.0 (/Users/felipeargento/Cartesi/descartes-v2/offchain/offchain)
#   Compiling tx-manager v0.1.0 (https://github.com/cartesi/tx-manager?rev=29a1abf#29a1abf5)
#   Compiling state-fold v0.1.0 (https://github.com/cartesi/state-fold?rev=64f9c0e#64f9c0ee)
#warning: unused import: `state_fold`
# --> offchain/src/error.rs:4:5
#  |
#4 | use state_fold;
#  |     ^^^^^^^^^^
#  |
#  = note: `#[warn(unused_imports)]` on by default
#
#warning: unused imports: `Address`, `U64`
# --> offchain/src/logic/instantiate_state_fold.rs:9:27
#  |
#9 | use ethers::core::types::{Address, U64};
#  |                           ^^^^^^^  ^^^
#
#warning: `offchain` (lib) generated 2 warnings
#    Finished dev [unoptimized + debuginfo] target(s) in 7.26s
#     Running `/Users/felipeargento/Cartesi/descartes-v2/offchain/target/debug/offchain --sender 0x2A20380DcA5bC24D052acfbf79ba23e988ad0050`
#Error: BadConfiguration { err: "Fail to initialize logic config: File error: Must specify rollups contract address" }
#?  offchain git:(feature/create-run-script) ? fg
#Send job 1, 'nvim run-logic.sh' to foreground
#fish: Job 1, 'nvim run-logic.sh' has stopped
#?  offchain git:(feature/create-run-script) ?
