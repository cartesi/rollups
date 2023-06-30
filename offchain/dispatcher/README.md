# Dispatcher

This service generates rollups inputs from state changes in the blockchain detected by the state-server.
These inputs are sent to the broker to be eventually used by the advance-runner.

The dispatcher also submits rollups claims consumed from the broker to the blockchain using the [tx-manager crate](https://github.com/cartesi/tx-manager).
