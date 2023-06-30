# Host Runner

This project implements the gRPC server-manager API.
Different from the server-manager, the host-runner does not instantiate a Cartesi machine.
Instead, it receives HTTP requests directly from a DApp running in the host machine.

## Tests

As a complement to the usual [test procedure](../README.md#tests), it is possible to enable verbose logging for integration testing by setting the following environment variable:

```shell
export CARTESI_TEST_VERBOSE=1
```
