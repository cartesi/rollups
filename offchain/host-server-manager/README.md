# Host Server Manager

This project implements the gRPC Server Manager API.
Different from the Server Manager, the Host Server Manager does not instantiate a Cartesi Server.
Instead, it makes HTTP requests directly to a DApp running in the host machine.
This project also simulates the Inspect HTTP API provided by the Inspect Container.

## Getting Started

This project requires Rust.
To install Rust follow the instructions [here](https://www.rust-lang.org/tools/install).

## Depedencies

Before building and running the project, you should download the submoules with:

```
git submodule update --init --recursive
```

## Running

To run the host-server-manager, execute the command:

```
cargo run
```

## Configuration

It is possible to configure the host-server-manager behaviour passing CLI arguments and using environment variables.
Execute the following command to check the available options.

```
cargo run -- -h
```

## Tests

To run the tests, execute the command:

```
cargo tests
```

In integration tests, it is possible to see the host-server-manager logs by setting the following variable:

```
export CARTESI_TEST_VERSBOSE=1
```
