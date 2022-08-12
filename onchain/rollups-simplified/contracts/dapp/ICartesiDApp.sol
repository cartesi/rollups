// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title ICartesi DApp
pragma solidity ^0.8.13;

import {IConsensus} from "../consensus/IConsensus.sol";
import {OutputValidityProof} from "../library/LibOutputValidation.sol";

interface ICartesiDApp {
    // Events

    event NewConsensus(IConsensus newConsensus);

    event VoucherExecuted(uint256 voucherPosition);

    // Permissioned functions

    function migrateToConsensus(IConsensus _newConsensus) external;

    // Permissionless functions

    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        bytes calldata _claimData,
        OutputValidityProof calldata _v
    ) external returns (bool);

    function validateNotice(
        bytes calldata _notice,
        bytes calldata _claimData,
        OutputValidityProof calldata _v
    ) external view returns (bool);

    function getTemplateHash() external view returns (bytes32);

    function getConsensus() external view returns (IConsensus);
}
