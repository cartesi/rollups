# Inspect Server

This server receives HTTP inspect-state requests and sends them to the server-manager.
The specification of the HTTP inspect API can be found in the [openapi-interfaces](https://github.com/cartesi/openapi-interfaces/) repository.

## Getting Started

This project requires Rust.
To install Rust follow the instructions [here](https://www.rust-lang.org/tools/install).

## Dependencies

Before building and running the project, you should download the submodules with:

```
git submodule update --init --recursive
```

## Configuration

It is possible to configure the inspect-server with CLI arguments, environment variables or a config file.
Execute the following command to check the available options.

```
cargo run -- -h
```

## Running

To run the inspect-server locally you need to setup an instance of the server-manager.
This can be done setting up an example in the [rollups-examples](../../../../../rollups-examples) repo.

1. Assuming you are running the server-manager on local port 5001, you could run the inspect server on port 5002 as such:

```
cargo run -- --inspect-server-address localhost:5002 --server-manager-address localhost:5001 --session-id default_rollups_id
```

2. Then, you could submit an inspect request with payload "mypayload" by sending a GET HTTP request as follows:

```
curl http://localhost:5002/inspect/mypayload
```

## Tests

To run the tests, execute the command:

```
cargo test
```
