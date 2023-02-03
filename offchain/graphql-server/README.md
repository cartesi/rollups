# GraphQL Server

This service exposes a GraphQL endpoint for easy quering of rollups data.

## Generating GraphQL Schema

Run the following command to generate the GraphQL schema based on the Rust code.

```
cargo run --bin generate-schema
```

## Running

To run the graphql server locally, you need to setup a PosgreSQL database as described in the data crate [README.md](../data/README.md).
Then, run the following command.

```
cargo run --bin graphql-server -- --postgres-password pw
```
