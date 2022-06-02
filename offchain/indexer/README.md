# Rollup indexer

Rollup indexer is the service that monitors happenings on blockchain and saves data retrieved from sever manager and state server.


## Test rollup indexer

The test suite requires postgres to be running in the correct configuration. The easiest way to do this is with docker:

1. Install `docker` and `docker-compose`.
    1. On ubuntu: `sudo apt install docker.io docker-compose`.
1. Make sure your user has permissions for docker.
    1. On ubuntu: ``sudo usermod -aG docker $USER``
1. Change to top-level directory of `offchain/data/indexer` repo.
1. Run `docker-compose -f tests/docker-compose.yml up --build -d`.
1. Run `cargo test`.
1. Run `docker-compose -f tests/docker-compose.yml stop`.