# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.1] - 2023-06-14
### Changed
- Renamed voucher address to destination

## [0.9.0] - 2023-05-05
### Changed
- Bumped grpc-interfaces to 0.12.0
- Updated server-manager gRPC interface
- Bumped external dependencies
- Bumped machine-emulator-tools to 0.11.0
- Bumped Rust version to 1.69

## [0.8.0] - 2023-02-17
### Changed
- Bumped grpc-interfaces to 0.10.0
- Bump machine-emulator-tools to 0.10.0
- Bumped Rust and Rust crates

## [0.7.0] - 2022-11-28
### Changed
- Bumped grpc-interfaces to 0.9.0
- Use distribution protoc instead of downloading from source
- Bumped Rust and Rust crates
- Bumped workflow Ubuntu to 22.04

## [0.6.0] - 2022-09-08
### Added
- Added gRPC healthcheck method

### Changed
- Bump dependencies

## [0.5.0] - 2022-07-20
### Added
- Implement gRPC inspect method

### Removed
- Removed HTTP inspect API

## [0.4.0] - 2022-06-28
### Changed
- Bumped machine-emulator-tools to 0.7.0
- Bumped grpc-interfaces to 0.8.0
- Changed exception handling

## [0.3.0] - 2022-04-20
### Added
- Compute hashes of vouchers and notices

### Changed
- Refactor Rollup HTTP API according to new OpenAPI interface
- Bumped server-manager gRPC API to 0.2.0

### Removed
- Removed cucumber from integration tests

## [0.2.1] - 2022-01-18
### Changed
- Change default port config

## [0.2.0] - 2022-01-13
### Changed
- Rename advance metadata fields according to openapi-interfaces

## [Previous Versions]
- [0.1.0]

[Unreleased]: https://github.com/cartesi/host-server-manager/compare/v0.9.1...HEAD
[0.9.1]: https://github.com/cartesi/host-server-manager/releases/tag/v0.9.1
[0.9.0]: https://github.com/cartesi/host-server-manager/releases/tag/v0.9.0
[0.8.0]: https://github.com/cartesi/host-server-manager/releases/tag/v0.8.0
[0.7.0]: https://github.com/cartesi/host-server-manager/releases/tag/v0.7.0
[0.6.0]: https://github.com/cartesi/host-server-manager/releases/tag/v0.6.0
[0.5.0]: https://github.com/cartesi/host-server-manager/releases/tag/v0.5.0
[0.4.0]: https://github.com/cartesi/host-server-manager/releases/tag/v0.4.0
[0.3.0]: https://github.com/cartesi/host-server-manager/releases/tag/v0.3.0
[0.2.1]: https://github.com/cartesi/host-server-manager/releases/tag/v0.2.1
[0.2.0]: https://github.com/cartesi/host-server-manager/releases/tag/v0.2.0
[0.1.0]: https://github.com/cartesi/host-server-manager/releases/tag/v0.1.0
