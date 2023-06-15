# Host Runner

This project implements the gRPC server-manager API.
Different from the server-manager, the host-runner does not instantiate a Cartesi machine.
Instead, it receives HTTP requests directly from a DApp running in the host machine.

## Getting Started

This project requires Rust.
To install Rust follow the instructions [here](https://www.rust-lang.org/tools/install).

## Depedencies

Before building and running the project, you should download the submoules with:

```
git submodule update --init --recursive
```

## Running

To run the host-runner, execute the command:

```
cargo run
```

## Configuration

It is possible to configure the host-runner behavior by passing CLI arguments and using environment variables.
Execute the following command to check the available options.

```
cargo run -- -h
```

## Tests

To run the tests, execute the command:

```
cargo test
```

In integration tests, it is possible to see the host-runner logs by setting the following variable:

```
export CARTESI_TEST_VERBOSE=1
```
