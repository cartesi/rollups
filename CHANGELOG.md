# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] 2022-11-02

### Changed

- Increase machine deadline
- Minor documentation updates

## [0.6.1] 2022-10-04

### Changed

- Fix dispatcher's configuration for server-manager threads
- Improve documentation of GraphQL API

## [0.6.0] 2022-09-13

### Added

- Deploy to Arbitrum Goerli and Optimism Goerli
- Add queue to serialize concurrent requests in inspect-server

### Changed

- Send inspect-server logs to stdout instead of stderr

## [0.5.0] 2022-08-17

### Added

- Inspect server
- Add path prefix option to inspect server
- Validate notice function to OutputFacet

### Changed

- Remove hardhat-rollups
- Fix indexer to store proofs only when epoch is finished

## [0.4.0] 2022-07-04

### Changed

- Update dependencies to latest emulator SDK with improved exception handling (grpc-interfaces 0.8)

## [0.3.0] 2022-06-14

### Added

- Factory contract to deploy rollups diamond
- Mermaid diagram of the on-chain rollups on README
- Deploy to several testnets (avax_fuji, bsc_testnet, goerli, kovan, polygon_mumbai, rinkeby, ropsten)
- New container with hardhat and deployed contracts for test environment
- New command line tool to deploy DApps

### Changed

- Moved logic from `erc721Deposit` function to `onERC721Received`
- Renamed `ERC721Deposited` event to `ERC721Received` and added `operator` field
- Validators who lost a dispute are removed from the validator set, and cannot redeem fees from previous claims
- Changed the visibility of `Bank`'s state variables to private
- Changed the visibility of `LibClaimsMask`'s functions to internal
- Improved docker entrypoints and configuration
- Gas optimizations

### Deprecated

### Removed

- `erc721Deposit` function (call `safeTransferFrom` from the ERC-721 contract instead)
- `erc20Withdrawal` function call (vouchers now call `transfer` from the ERC-20 contract directly instead)

### Security

## [0.2.0] 2022-04-28

### Added

- FeeManager facet and Bank contract
- Altruistic and Non-altruistic behavior for Validator Node
- Template Hash
- Setup Input
- NFT Portal
- New hardhat tasks

### Changed

- Updated architecture to Diamonds design pattern
- Bumped solc version to 0.8.13
- Separated npm workspaces for `rollups` and `hardhat-rollups`

### Removed

- Specific ERC-20 Portal
- Deprecated mock contracts

[0.7.0]: https://github.com/cartesi/rollups/releases/tag/v0.7.0
[0.6.1]: https://github.com/cartesi/rollups/releases/tag/v0.6.1
[0.6.0]: https://github.com/cartesi/rollups/releases/tag/v0.6.0
[0.5.0]: https://github.com/cartesi/rollups/releases/tag/v0.5.0
[0.4.0]: https://github.com/cartesi/rollups/releases/tag/v0.4.0
[0.3.0]: https://github.com/cartesi/rollups/releases/tag/v0.3.0
[0.2.0]: https://github.com/cartesi/rollups/releases/tag/v0.2.0
[0.1.3]: https://github.com/cartesi/rollups/releases/tag/v0.1.3
[0.1.2]: https://github.com/cartesi/rollups/releases/tag/v0.1.2
[0.1.1]: https://github.com/cartesi/rollups/releases/tag/v0.1.1
[0.1.0]: https://github.com/cartesi/rollups/releases/tag/v0.1.0
