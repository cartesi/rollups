# Rollup reader

Rollup reader is the service that exposes graphql endpoint for easy quering of dapps data


## Test rollup reader

The test suite requires postgres to be running in the correct configuration. The easiest way to do this is with docker:

1. Install `docker` and `docker-compose`.
    1. On ubuntu: `sudo apt install docker.io docker-compose`.
2. Make sure your user has permissions for docker.
    1. On ubuntu: ``sudo usermod -aG docker $USER``
3. Change to top-level directory of `offchain/reader` repo.
4. Build reader binary `cargo run --release`. 
5. Run `docker-compose -f tests/docker-compose.yml up --build -d`.
7. Run `RUST_LOG=error ROLLUPS_READER_BINARY_PATH=../target/release/reader cargo test`.
8. Run `docker-compose -f tests/docker-compose.yml stop`.