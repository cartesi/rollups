# Indexer

This service is responsible for inserting rollups inputs and outputs in the PostgreSQL database.
The indexer consume the inputs and the outputs from the rollups broker.

## Getting Started

This project requires Rust.
To install Rust follow the instructions [here](https://www.rust-lang.org/tools/install).

## Configuration

It is possible to configure the service with CLI arguments or environment variables.
Execute the following command to check the available options.

```
cargo run -- -h
```

## Tests

To run the tests, you need docker installed. Then, execute the command:

```
cargo test
```
