// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.8;

/// @title Consensus interface
///
/// @notice This contract defines a generic interface for consensuses.
/// We use the word "consensus" to designate a contract that provides claims
/// in the base layer regarding the state of off-chain machines running in
/// the execution layer. How this contract is able to reach consensus, who is
/// able to submit claims, and how are claims stored in the base layer are
/// some of the implementation details left unspecified by this interface.
///
/// From the point of view of a DApp, these claims are necessary to validate
/// on-chain action allowed by the off-chain machine in the form of vouchers
/// and notices. Each claim is composed of three parts: an epoch hash, a first
/// index, and a last index. We'll explain each of these parts below.
///
/// First, let us define the word "epoch". For finality reasons, we need to
/// divide the stream of inputs being fed into the off-chain machine into
/// batches of inputs, which we call "epoches". At the end of every epoch,
/// we summarize the state of the off-chain machine in a single hash, called
/// "epoch hash". Please note that this interface does not define how this
/// stream of inputs is being chopped up into epoches.
///
/// The other two parts are simply the indices of the first and last inputs
/// accepted during the epoch. Logically, the first index MUST BE less than
/// or equal to the last index. As a result, every epoch MUST accept at least
/// one input. This assumption stems from the fact that the state of a machine
/// can only change after an input is fed into it.
///
/// Examples of possible implementations of this interface include:
///
/// * An authority consensus, controlled by a single address who has full
///   control over epoch boundaries, claim submission, asset management, etc.
///
/// * A quorum consensus, controlled by a limited set of validators, that
///   vote on the state of the machine at the end of every epoch. Also, epoch
///   boundaries are determined by the timestamp in the base layer, and assets
///   are split equally amongst the validators.
///
/// * An NxN consensus, which allows anyone to submit and dispute claims
///   in the base layer. Epoch boundaries are determined in the same fashion
///   as in the quorum example.
///
interface IConsensus {
    /// @notice An application has joined the consensus' validation set.
    /// @param application The application
    /// @dev MUST be triggered on a successful call to `join`.
    event ApplicationJoined(address application);

    /// @notice Get a specific claim regarding a specific DApp.
    ///         The encoding of `_proofContext` might vary
    ///         depending on the implementation.
    /// @param _dapp The DApp address
    /// @param _proofContext Data for retrieving the desired claim
    /// @return epochHash_ The claimed epoch hash
    /// @return firstInputIndex_ The index of the first input of the epoch in the input box
    /// @return lastInputIndex_ The index of the last input of the epoch in the input box
    function getClaim(
        address _dapp,
        bytes calldata _proofContext
    )
        external
        view
        returns (
            bytes32 epochHash_,
            uint256 firstInputIndex_,
            uint256 lastInputIndex_
        );

    /// @notice Signal the consensus that the message sender wants to join its validation set.
    /// @dev MUST fire an `ApplicationJoined` event with the message sender as argument.
    function join() external;
}
