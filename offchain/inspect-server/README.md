# Inspect Server

This server receives HTTP inspect-state requests and sends them to the server-manager.
The specification of the HTTP inspect API can be found in the [openapi-interfaces](https://github.com/cartesi/openapi-interfaces/) repository.

## Running

To run the inspect-server locally you need to setup an instance of the server-manager.
This can be done setting up an example in the [rollups-examples](https://github.com/cartesi/rollups-examples) repository.

1. Assuming you are running the server-manager on local port 5001, you could run the inspect server on port 5002 as such:

```shell
cargo run -- --inspect-server-address localhost:5002 --server-manager-address localhost:5001 --session-id default_rollups_id
```

2. Then, you could submit an inspect request with payload "mypayload" by sending a GET HTTP request as follows:

```shell
curl http://localhost:5002/inspect/mypayload
```
