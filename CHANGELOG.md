# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added support to Cartesi Machine snapshots
- [Script](onchain/rollups/test/foundry/dapp/helper/README.md) for updating proofs used in unit tests
- Authority consensus model implementation (single validator)
- Simple claim storage implementation (one claim per DApp)
- Library that defines several constants related to the canonical off-chain machine
- Added integration tests for proxy

### Changed

- Simplified the on-chain architecture (not backwards-compatible)
- Adopted [Foundry](https://book.getfoundry.sh/) for contract testing (Hardhat is still being used for deployment)
- `CartesiDApp` does not implement [EIP-2535](https://eips.ethereum.org/EIPS/eip-2535) anymore
- Made each Portal a contract of their own, and shared amongst all the DApps
- Made inputs added by Portals more compact by using the [packed ABI encoding](https://docs.soliditylang.org/en/latest/abi-spec.html#non-standard-packed-mode) instead of the standard one
- Made ERC-20 deposits more generic by allowing L1 transfers to fail, and adding a boolean field signaling whether it was successful or not
- Made ERC-721 deposits more generic by adding an arbitrary data field to be interpreted by L2
- Validators are now paid by working time (new Consensus Payment System), and not by number of claims (old Fee Manager Facet)
- Moved the input boxes of every DApp into a single, permissionless contract
- Input boxes are now append-only -- they are not cleared every new epoch (old Input Facet)
- Modularized the consensus layer (a DApp can now seamlessly change its consensus model)
- Modularized the claim storage layer (a consensus can now seamlessly change how it stores claims)
- Renamed `inputIndex` field from `OutputValidityProof` struct as `epochInputIndex`
- Voucher bitmask position is now determined by the input index (in the input box) and output index
- Validators need now to specify the range of inputs of each claim they submit on-chain

### Removed

- Setup Input
- Quorum consensus model implementation (up to 8 validators)
- Bank contract
- DApp configuration parameters related to the off-chain machine specs (now defined as constants)
- Removed `epochIndex` field from `OutputValidityProof` struct

## [0.8.2] 2023-01-04

### Changed

- Fixed proxy start up after restart
- Changed `sm_pending_inputs_max_retries` default to 600 (retries while polling server-manager for pending inputs)

## [0.8.1] 2022-12-02

### Added

- Support of gnosis chiado testnet

### Changed

- Fixed epoch finalization when running rollups in host mode

## [0.8.0] 2022-11-29

### Added

- Added the rollups-events crate that works as an abstraction to producing and consuming events.
- Added the server-manager-broker-proxy service to consume events from the broker and manage the server-manager.
- Added request id to server-manager calls
- Added server-manager session config as CLI arguments
- Store DApp deployment information in JSON instead of plain text
- Compatibility with networks without EIP1559 transactions

### Changed

- Modified the dispatcher to produce rollups events instead of managing the server-manager.
- Bumped grpc-interfaces to version 0.9.0

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

[0.8.2]: https://github.com/cartesi/rollups/releases/tag/v0.8.2
[0.8.1]: https://github.com/cartesi/rollups/releases/tag/v0.8.1
[0.8.0]: https://github.com/cartesi/rollups/releases/tag/v0.8.0
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
