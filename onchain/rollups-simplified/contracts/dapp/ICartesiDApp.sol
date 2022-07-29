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

import {LibOutputValidation} from "../library/LibOutputValidation.sol";

interface ICartesiDApp {
    function executeVoucher(
        address _destination,
        bytes calldata _payload,
        LibOutputValidation.OutputValidityProof calldata _v
    ) external returns (bool);

    function validateNotice(
        bytes calldata _notice,
        LibOutputValidation.OutputValidityProof calldata _v
    ) external view returns (bool);

    function migrateToConsensus(address _consensus) external;

    function finalizeEpoch() external;

    function getEpoch() external view returns (uint256);

    event NewConsensus(address newConsensus);

    event VoucherExecuted(uint256 voucherPosition);
}
