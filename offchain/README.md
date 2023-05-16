# Cartesi Node Reference Implementation

This file is a work in progress. It will soon document how to build, test and overall handle a Cartesi Node.

## Redis TLS Configuration

To connect the `Broker` to a Redis server via TLS, the server's URL must use the `rediss://` scheme (with two "s"es).
This is currently the only way to tell `Broker` to use a TLS connection.
