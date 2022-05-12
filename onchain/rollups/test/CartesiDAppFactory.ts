// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import { deployments, ethers } from "hardhat";
import { expect, use } from "chai";
import { solidity } from "ethereum-waffle";
import { Signer } from "ethers";
import {
    CartesiDApp,
    CartesiDAppFactory,
    CartesiDAppFactory__factory,
    DiamondCutFacet__factory,
} from "../src/types";
import { deployDiamond } from "./utils";

use(solidity);

describe("Cartesi DApp Factory", () => {
    let factory: CartesiDAppFactory;
    let diamond: CartesiDApp;
    let signers: Signer[];

    // constants
    const DAY = 60 * 60 * 24;

    // diamond options
    let diamondOwner: string;
    let templateHash =
        "0x00000000000000000000000000000000000000000000000000000000deadbeef";
    let inputDuration = 1 * DAY;
    let challengePeriod = 7 * DAY;
    let inputLog2Size = 8;
    let feePerClaim = 10;
    let feeManagerOwner: string;
    let feeManagerBank: string;
    let validators: string[];

    beforeEach(async () => {
        signers = await ethers.getSigners();

        // dummy addresses
        diamondOwner = await signers[1].getAddress();
        feeManagerOwner = await signers[2].getAddress();
        feeManagerBank = await signers[3].getAddress();

        validators = [];
        for (const signer of signers) {
            const signerAddress = await signer.getAddress();
            validators.push(signerAddress);
            if (validators.length == 8) break;
        }

        diamond = await deployDiamond({
            diamondOwner,
            templateHash,
            inputDuration,
            challengePeriod,
            inputLog2Size,
            feePerClaim,
            feeManagerBank,
            feeManagerOwner,
            validators,
        });

        const factoryDeployment = await deployments.get("CartesiDAppFactory");
        factory = CartesiDAppFactory__factory.connect(
            factoryDeployment.address,
            signers[0]
        );
    });

    it("Check factory state variables", async () => {
        const { DiamondCutFacet, DiamondInit } = await deployments.all();
        expect(await factory.diamondCutFacet()).to.equal(
            DiamondCutFacet.address
        );
        expect(await factory.diamondInit()).to.equal(DiamondInit.address);
        expect(await factory.feeManagerBank()).to.equal(feeManagerBank);
    });

    it("Check events", async () => {
        let eventFilter = factory.filters.ApplicationCreated(diamond.address);
        let events = await factory.queryFilter(eventFilter);
        expect(events.length).to.equal(1);
        const { application, config } = events[0].args;
        expect(application).to.equal(diamond.address);
        expect(config.diamondOwner).to.equal(diamondOwner);
        expect(config.templateHash).to.equal(templateHash);
        expect(config.inputDuration).to.equal(inputDuration);
        expect(config.challengePeriod).to.equal(challengePeriod);
        expect(config.inputLog2Size).to.equal(inputLog2Size);
        expect(config.feePerClaim).to.equal(feePerClaim);
        expect(config.feeManagerOwner).to.equal(feeManagerOwner);
        expect(config.validators).to.deep.equal(validators);
    });

    it("Check diamond cut", async () => {
        let diamondCutFacet = DiamondCutFacet__factory.connect(
            diamond.address,
            signers[0]
        );
        let eventFilter = diamondCutFacet.filters.DiamondCut();
        let events = await diamondCutFacet.queryFilter(eventFilter);
        const { DiamondInit } = await deployments.all();
        let found = false;
        for (let event of events) {
            let eventArgs = event.args;
            if (eventArgs.init == DiamondInit.address) {
                expect(found).to.be.false;
                found = true;
                let diamondCut = eventArgs.diamondCut;
                for (
                    let facetCuts = 0;
                    facetCuts < diamondCut.length;
                    facetCuts++
                ) {
                    let factoryFacetCut = await factory.diamondCut(facetCuts);
                    expect(
                        factoryFacetCut.facetAddress,
                        `Facet Cut #${facetCuts} (address)`
                    ).to.equal(diamondCut[facetCuts].facetAddress);
                    expect(
                        factoryFacetCut.action,
                        `Facet Cut #${facetCuts} (action)`
                    ).to.equal(diamondCut[facetCuts].action);
                }
            }
        }
        expect(found).to.be.true;
    });
});
