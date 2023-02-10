# Advance Runner

This service consumes rollups input events from the broker and use them to advance the server-manager state.
When the epoch finishes, the advance-runner gets the claim from the server-manager and produces the rollups claim event.

## Getting Started

This project requires Rust.
To install Rust follow the instructions [here](https://www.rust-lang.org/tools/install).

## Dependencies

Before building and running the project, you should download the submodules with:

```
git submodule update --init --recursive
```

## Configuration

It is possible to configure the service with CLI arguments or environment variables.
Execute the following command to check the available options.

```
cargo run -- -h
```

## Tests

To run the tests, you need docker installed. The, execute the command:

```
cargo test
```
