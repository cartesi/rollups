// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// @title Cartesi DApp Factory
pragma solidity ^0.8.0;

import {ICartesiDAppFactory} from "./ICartesiDAppFactory.sol";
import {CartesiDApp} from "./CartesiDApp.sol";
import {IDiamondCut} from "./interfaces/IDiamondCut.sol";
import {IERC173} from "./interfaces/IERC173.sol";
import {DiamondInit, DiamondConfig} from "./upgrade_initializers/DiamondInit.sol";
import {IBank} from "./IBank.sol";

contract CartesiDAppFactory is ICartesiDAppFactory {
    IDiamondCut public immutable diamondCutFacet;
    DiamondInit public immutable diamondInit;
    IBank public immutable feeManagerBank;
    IDiamondCut.FacetCut[] public diamondCut;

    struct FactoryConfig {
        IDiamondCut diamondCutFacet;
        DiamondInit diamondInit;
        IBank feeManagerBank;
        IDiamondCut.FacetCut[] diamondCut;
    }

    constructor(FactoryConfig memory _fConfig) {
        diamondCutFacet = _fConfig.diamondCutFacet;
        diamondInit = _fConfig.diamondInit;
        feeManagerBank = _fConfig.feeManagerBank;
        for (uint256 i; i < _fConfig.diamondCut.length; ++i) {
            diamondCut.push(_fConfig.diamondCut[i]);
        }
    }

    function newApplication(
        AppConfig calldata _appConfig
    ) public returns (CartesiDApp) {
        CartesiDApp application = new CartesiDApp(
            address(this),
            address(diamondCutFacet)
        );
        DiamondConfig memory dConfig = DiamondConfig({
            templateHash: _appConfig.templateHash,
            inputDuration: _appConfig.inputDuration,
            challengePeriod: _appConfig.challengePeriod,
            inputLog2Size: _appConfig.inputLog2Size,
            feePerClaim: _appConfig.feePerClaim,
            feeManagerBank: address(feeManagerBank),
            feeManagerOwner: _appConfig.feeManagerOwner,
            validators: _appConfig.validators
        });
        IDiamondCut(address(application)).diamondCut(
            diamondCut,
            address(diamondInit),
            abi.encodeWithSelector(DiamondInit.init.selector, dConfig)
        );
        IERC173(address(application)).transferOwnership(
            _appConfig.diamondOwner
        );
        emit ApplicationCreated(application, _appConfig);
        return application;
    }
}
