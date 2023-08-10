// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

pragma solidity ^0.8.17;

import "./CanonicalConstants.sol";
import "./Tree.sol";
import "./Machine.sol";

// import "./Merkle.sol";

library Commitment {
    using Commitment for Tree.Node;

    function proveFinalState(
        Tree.Node root,
        uint64 level,
        Machine.Hash finalState,
        bytes32[] calldata hashProof
    ) internal pure {
        root.proveHash(
            uint64(1 << ArbitrationConstants.height(level)),
            finalState,
            hashProof
        );
    }

    function proveHash(
        Tree.Node root,
        uint256 position,
        Machine.Hash hash,
        bytes32[] calldata hashProof
    ) internal pure {
        // TODO: call Merkle library
    }
}
