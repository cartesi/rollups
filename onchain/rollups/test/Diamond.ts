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
import { Signer, BigNumber } from "ethers";
import { DeployOptions } from "hardhat-deploy/types";
import {
    Foo1Facet,
    Foo1Facet__factory,
    Foo2Facet,
    Foo2Facet__factory,
    DS1Facet,
    DS1Facet__factory,
    DS2Facet,
    DS2Facet__factory,
    DS3Facet,
    DS3Facet__factory,
    DS4Facet,
    DS4Facet__factory,
    DS5Facet,
    DS5Facet__factory,
    DS6Facet,
    DS6Facet__factory,
    DS7Facet,
    DS7Facet__factory,
    DS8Facet,
    DS8Facet__factory,
    DS9Facet,
    DS9Facet__factory,
    DS1Init,
    DS1Init__factory,
    DS2Upgrade,
    DS2Upgrade__factory,
    DS3Upgrade,
    DS3Upgrade__factory,
    DS4Upgrade,
    DS4Upgrade__factory,
    DS6Init,
    DS6Init__factory,
    DS6Upgrade,
    DS6Upgrade__factory,
    DS6Downgrade,
    DS6Downgrade__factory,
    DS7Init,
    DS7Init__factory,
    DS7Upgrade,
    DS7Upgrade__factory,
    DS7Downgrade,
    DS7Downgrade__factory,
    DS9Upgrade,
    DS9Upgrade__factory,
    DiamondCutFacet,
    DiamondCutFacet__factory,
    DiamondLoupeFacet,
    DiamondLoupeFacet__factory,
} from "../src/types";
import { IDiamondCut } from "../src/types";
import {
    getFacetCuts,
    getFunctionSelectors,
    getAddFacetCut,
    getReplaceFacetCut,
    getRemoveFacetCut,
    FacetCutAction,
} from "../src/utils";

use(solidity);

describe("EIP-2535 Diamond", () => {
    let deployer: Signer;
    let owner: Signer;
    let user: Signer;
    let diamond: string;
    let diamondCutFacet: DiamondCutFacet;
    let diamondLoupeFacet: DiamondLoupeFacet;
    let foo1Facet: Foo1Facet;
    let foo2Facet: Foo2Facet;
    let ds1Facet: DS1Facet;
    let ds2Facet: DS2Facet;
    let ds3Facet: DS3Facet;
    let ds4Facet: DS4Facet;
    let ds5Facet: DS5Facet;
    let ds6Facet: DS6Facet;
    let ds7Facet: DS7Facet;
    let ds8Facet: DS8Facet;
    let ds9Facet: DS9Facet;
    let ds1Init: DS1Init;
    let ds2Upgrade: DS2Upgrade;
    let ds3Upgrade: DS3Upgrade;
    let ds4Upgrade: DS4Upgrade;
    let ds6Init: DS6Init;
    let ds6Upgrade: DS6Upgrade;
    let ds6Downgrade: DS6Downgrade;
    let ds7Init: DS7Init;
    let ds7Upgrade: DS7Upgrade;
    let ds7Downgrade: DS7Downgrade;
    let ds9Upgrade: DS9Upgrade;

    const { AddressZero } = ethers.constants;

    async function diamondCut(
        facetCuts: IDiamondCut.FacetCutStruct[],
        init?: string,
        payload?: string
    ) {
        return await diamondCutFacet.diamondCut(
            facetCuts,
            init || AddressZero,
            payload || "0x"
        );
    }

    beforeEach(async () => {
        await deployments.fixture();

        const { DiamondCutFacet, DiamondLoupeFacet } = await deployments.all();

        [deployer, owner, user] = await ethers.getSigners();

        const opts: DeployOptions = {
            from: await deployer.getAddress(),
            log: true,
        };

        // Deploy test facets
        await deployments.deploy("Foo1Facet", opts);
        await deployments.deploy("Foo2Facet", opts);
        await deployments.deploy("DS1Facet", opts);
        await deployments.deploy("DS2Facet", opts);
        await deployments.deploy("DS3Facet", opts);
        await deployments.deploy("DS4Facet", opts);
        await deployments.deploy("DS5Facet", opts);
        await deployments.deploy("DS6Facet", opts);
        await deployments.deploy("DS7Facet", opts);
        await deployments.deploy("DS8Facet", opts);
        await deployments.deploy("DS9Facet", opts);

        // Deploy upgrade initializers
        var { address } = await deployments.deploy("DS1Init", opts);
        ds1Init = DS1Init__factory.connect(address, user);
        var { address } = await deployments.deploy("DS2Upgrade", opts);
        ds2Upgrade = DS2Upgrade__factory.connect(address, user);
        var { address } = await deployments.deploy("DS3Upgrade", opts);
        ds3Upgrade = DS3Upgrade__factory.connect(address, user);
        var { address } = await deployments.deploy("DS4Upgrade", opts);
        ds4Upgrade = DS4Upgrade__factory.connect(address, user);
        var { address } = await deployments.deploy("DS6Init", opts);
        ds6Init = DS6Init__factory.connect(address, user);
        var { address } = await deployments.deploy("DS6Upgrade", opts);
        ds6Upgrade = DS6Upgrade__factory.connect(address, user);
        var { address } = await deployments.deploy("DS6Downgrade", opts);
        ds6Downgrade = DS6Downgrade__factory.connect(address, user);
        var { address } = await deployments.deploy("DS7Init", opts);
        ds7Init = DS7Init__factory.connect(address, user);
        var { address } = await deployments.deploy("DS7Upgrade", opts);
        ds7Upgrade = DS7Upgrade__factory.connect(address, user);
        var { address } = await deployments.deploy("DS7Downgrade", opts);
        ds7Downgrade = DS7Downgrade__factory.connect(address, user);
        var { address } = await deployments.deploy("DS9Upgrade", opts);
        ds9Upgrade = DS9Upgrade__factory.connect(address, user);

        // Deploy diamond
        const diamondDeployment = await deployments.deploy("CartesiDApp", {
            ...opts,
            args: [await owner.getAddress(), DiamondCutFacet.address],
        });
        diamond = diamondDeployment.address;

        // Connect facets
        diamondCutFacet = DiamondCutFacet__factory.connect(diamond, owner);
        diamondLoupeFacet = DiamondLoupeFacet__factory.connect(diamond, user);
        foo1Facet = Foo1Facet__factory.connect(diamond, user);
        foo2Facet = Foo2Facet__factory.connect(diamond, user);
        ds1Facet = DS1Facet__factory.connect(diamond, user);
        ds2Facet = DS2Facet__factory.connect(diamond, user);
        ds3Facet = DS3Facet__factory.connect(diamond, user);
        ds4Facet = DS4Facet__factory.connect(diamond, user);
        ds5Facet = DS5Facet__factory.connect(diamond, user);
        ds6Facet = DS6Facet__factory.connect(diamond, user);
        ds7Facet = DS7Facet__factory.connect(diamond, user);
        ds8Facet = DS8Facet__factory.connect(diamond, user);
        ds9Facet = DS9Facet__factory.connect(diamond, user);
    });

    it("facets(): query all facets and function signatures", async () => {
        const { DiamondCutFacet, DiamondLoupeFacet } = await deployments.all();

        await diamondCut([
            getAddFacetCut(
                DiamondLoupeFacet.address,
                getFunctionSelectors(diamondLoupeFacet)
            ),
        ]);

        expect(
            await diamondLoupeFacet.facets(),
            "Diamond facets"
        ).to.have.deep.members([
            [DiamondCutFacet.address, getFunctionSelectors(diamondCutFacet)],
            [
                DiamondLoupeFacet.address,
                getFunctionSelectors(diamondLoupeFacet),
            ],
        ]);
    });

    it("facetFunctionSelectors(): for registered facets", async () => {
        const { DiamondCutFacet, DiamondLoupeFacet } = await deployments.all();

        await diamondCut([
            getAddFacetCut(
                DiamondLoupeFacet.address,
                getFunctionSelectors(diamondLoupeFacet)
            ),
        ]);

        expect(
            await diamondLoupeFacet.facetFunctionSelectors(
                DiamondCutFacet.address
            ),
            "Diamond function selectors of DiamondCutFacet"
        ).to.have.members(getFunctionSelectors(diamondCutFacet));

        expect(
            await diamondLoupeFacet.facetFunctionSelectors(
                DiamondLoupeFacet.address
            ),
            "Diamond function selectors of DiamondLoupeFacet"
        ).to.have.members(getFunctionSelectors(diamondLoupeFacet));
    });

    it("facetFunctionSelectors(): for unregistered facets", async () => {
        const { DiamondLoupeFacet } = await deployments.all();

        await diamondCut([
            getAddFacetCut(
                DiamondLoupeFacet.address,
                getFunctionSelectors(diamondLoupeFacet)
            ),
        ]);

        expect(
            await diamondLoupeFacet.facetFunctionSelectors(AddressZero),
            "Diamond function selectors of address(0)"
        ).to.have.members([]);
    });

    it("fallback(): reverts when function does not exist", async () => {
        await expect(
            foo1Facet.foo(),
            "Function 'foo' does not exist"
        ).to.be.revertedWith("Diamond: Function does not exist");
    });

    it("diamondCut(): add function", async () => {
        const { Foo1Facet } = await deployments.all();
        await diamondCut([
            getAddFacetCut(Foo1Facet.address, getFunctionSelectors(foo1Facet)),
        ]);
        expect(await foo1Facet.foo(), "Function 'foo' exists").to.equal(42);
    });

    it("diamondCut(): try adding function twice", async () => {
        const { Foo1Facet } = await deployments.all();
        const selectors = getFunctionSelectors(foo1Facet);
        await diamondCut([getAddFacetCut(Foo1Facet.address, selectors)]);
        await expect(
            diamondCut([getAddFacetCut(Foo1Facet.address, selectors)]),
            "Can't add function again"
        ).to.be.revertedWith(
            "LibDiamondCut: Can't add function that already exists'"
        );
    });

    it("diamondCut(): remove function", async () => {
        const selectors = getFunctionSelectors(diamondCutFacet);
        await diamondCut([getRemoveFacetCut(selectors)]);
        await expect(
            diamondCutFacet.diamondCut([], AddressZero, "0x"),
            "Function 'diamondCut' does not exist"
        ).to.be.revertedWith("Diamond: Function does not exist");
    });

    it("diamondCut(): try removing function that was not added", async () => {
        const selectors = getFunctionSelectors(foo1Facet);
        await expect(
            diamondCut([getRemoveFacetCut(selectors)]),
            "Can't remove function that wasn't added"
        ).to.be.revertedWith(
            "LibDiamondCut: Can't remove function that doesn't exist'"
        );
    });

    it("diamondCut(): replace function", async () => {
        const { Foo1Facet, Foo2Facet } = await deployments.all();
        const selectors = getFunctionSelectors(foo1Facet);
        await diamondCut([getAddFacetCut(Foo1Facet.address, selectors)]);
        expect(await foo1Facet.foo(), "Function 'foo' exists").to.equal(42);
        await diamondCut([getReplaceFacetCut(Foo2Facet.address, selectors)]);
        expect(await foo1Facet.foo(), "'foo' was replaced").to.equal(10);
    });

    it("diamondCut(): initialize storage", async () => {
        let x = 10;
        let y = 42;

        // Add `getX`, `setX`, `getY` and `setY` functions
        const { DS1Facet } = await deployments.all();
        const ds1Selectors = getFunctionSelectors(ds1Facet);
        await diamondCut(
            [getAddFacetCut(DS1Facet.address, ds1Selectors)],
            ds1Init.address,
            ds1Init.interface.encodeFunctionData("init", [x, y])
        );

        // Check initialized values
        expect(await ds1Facet.getX()).to.equal(x);
        expect(await ds1Facet.getY()).to.equal(y);

        // Change values
        x = 123;
        y = 456;
        await ds1Facet.setX(x);
        await ds1Facet.setY(y);

        // Check new values
        expect(await ds1Facet.getX()).to.equal(x);
        expect(await ds1Facet.getY()).to.equal(y);
    });

    it("diamondCut(): change field type", async () => {
        let x: BigNumber = BigNumber.from(10);
        let y = 42;

        // Add `getX`, `setX`, `getY` and `setY` functions
        const { DS1Facet } = await deployments.all();
        const ds1Selectors = getFunctionSelectors(ds1Facet);
        await diamondCut(
            [getAddFacetCut(DS1Facet.address, ds1Selectors)],
            ds1Init.address,
            ds1Init.interface.encodeFunctionData("init", [x, y])
        );

        // Check bounds of x
        x = BigNumber.from(2).pow(32).sub(1);
        await ds1Facet.setX(x);
        expect(await ds1Facet.getX()).to.equal(x);
        await expect(ds1Facet.setX(x.add(1)), "overflow").to.be.reverted;

        // Updgrade to version 2
        // (expand x from 32 bits to 64 bits)
        const { DS2Facet } = await deployments.all();
        await diamondCut(
            [
                getRemoveFacetCut([
                    ds1Facet.interface.getSighash("setX(uint32)"),
                ]),
                getAddFacetCut(DS2Facet.address, [
                    ds2Facet.interface.getSighash("setX(uint64)"),
                ]),
                getReplaceFacetCut(DS2Facet.address, [
                    ds1Facet.interface.getSighash("getX()"),
                ]),
            ],
            ds2Upgrade.address,
            ds2Upgrade.interface.encodeFunctionData("upgrade")
        );

        // Check removed function
        await expect(ds1Facet.setX(0)).to.be.revertedWith(
            "Diamond: Function does not exist"
        );

        // Check upgraded values
        expect(await ds2Facet.getX()).to.equal(x);
        expect(await ds2Facet.getY()).to.equal(y);

        // Check bounds of x
        x = BigNumber.from(2).pow(64).sub(1);
        await ds2Facet.setX(x);
        expect(await ds2Facet.getX()).to.equal(x);
        await expect(ds2Facet.setX(x.add(1)), "overflow").to.be.reverted;
    });

    it("diamondCut(): add field", async () => {
        let x = 10;
        let y = 42;

        // Add `getX`, `setX`, `getY` and `setY` functions
        const { DS1Facet } = await deployments.all();
        const ds1Selectors = getFunctionSelectors(ds1Facet);
        await diamondCut(
            [getAddFacetCut(DS1Facet.address, ds1Selectors)],
            ds1Init.address,
            ds1Init.interface.encodeFunctionData("init", [x, y])
        );

        // Updgrade to version 3
        // (add field z)
        let z = 12;
        const { DS3Facet } = await deployments.all();
        await diamondCut(
            [
                getAddFacetCut(DS3Facet.address, [
                    ds3Facet.interface.getSighash("getZ()"),
                    ds3Facet.interface.getSighash("setZ(uint32)"),
                ]),
            ],
            ds3Upgrade.address,
            ds3Upgrade.interface.encodeFunctionData("upgrade", [z])
        );

        // Check upgraded values
        expect(await ds3Facet.getX()).to.equal(x);
        expect(await ds3Facet.getY()).to.equal(y);
        expect(await ds3Facet.getZ()).to.equal(z);

        // Change values
        x = 789;
        y = 1011;
        z = 9999;
        await ds3Facet.setX(x);
        await ds3Facet.setY(y);
        await ds3Facet.setZ(z);

        // Check new values
        expect(await ds3Facet.getX()).to.equal(x);
        expect(await ds3Facet.getY()).to.equal(y);
        expect(await ds3Facet.getZ()).to.equal(z);
    });

    it("diamondCut(): remove field", async () => {
        let x = 10;
        let y = 42;

        // Add `getX`, `setX`, `getY` and `setY` functions
        const { DS1Facet } = await deployments.all();
        const ds1Selectors = getFunctionSelectors(ds1Facet);
        await diamondCut(
            [getAddFacetCut(DS1Facet.address, ds1Selectors)],
            ds1Init.address,
            ds1Init.interface.encodeFunctionData("init", [x, y])
        );

        // Updgrade to version 4
        // (remove field x)
        const { DS4Facet } = await deployments.all();
        await diamondCut(
            [
                getRemoveFacetCut([
                    ds1Facet.interface.getSighash("getX()"),
                    ds1Facet.interface.getSighash("setX(uint32)"),
                ]),
            ],
            ds4Upgrade.address,
            ds4Upgrade.interface.encodeFunctionData("upgrade")
        );

        // Check removed functions
        await expect(ds1Facet.getX()).to.be.revertedWith(
            "Diamond: Function does not exist"
        );
        await expect(ds1Facet.setX(0)).to.be.revertedWith(
            "Diamond: Function does not exist"
        );

        // Check upgraded values
        expect(await ds4Facet.getY()).to.equal(y);

        // Change values
        y = 1011;
        await ds4Facet.setY(y);
        expect(await ds4Facet.getY()).to.equal(y);
    });

    it("diamondCut(): dynamic types on diamond storage", async () => {
        // Add functions from DS5Facet (no initialization routine)
        const { DS5Facet } = await deployments.all();
        const ds5Selectors = getFunctionSelectors(ds5Facet);
        await diamondCut([getAddFacetCut(DS5Facet.address, ds5Selectors)]);

        // Read storage
        expect(await ds5Facet.getMappingEntry(0)).to.equal(0);
        expect(await ds5Facet.getArrayLength()).to.equal(0);

        // Write to storage
        const k = 111;
        const a = 42;
        const b = 333;
        await ds5Facet.setMappingEntry(k, a);
        await ds5Facet.addArrayElement(b);

        // Check if storage was updated
        expect(await ds5Facet.getMappingEntry(k)).to.equal(a);
        expect(await ds5Facet.getArrayLength()).to.equal(1);
        expect(await ds5Facet.getArrayElement(0)).to.equal(b);
    });

    it("diamondCut(): add field in-place", async () => {
        // Add functions from DS5Facet (no initialization routine)
        const { DS5Facet } = await deployments.all();
        const ds5Selectors = getFunctionSelectors(ds5Facet);
        await diamondCut([getAddFacetCut(DS5Facet.address, ds5Selectors)]);

        // Write to storage
        const k = 111;
        const a = 42;
        const b = 333;
        await ds5Facet.setMappingEntry(k, a);
        const n = 10;
        const c = 777;
        for (let i = 0; i < n; ++i) {
            await ds5Facet.addArrayElement(i * b + c);
        }

        // Updgrade to version 6
        // (append field x)
        let x = 555;
        const { DS6Facet } = await deployments.all();
        await diamondCut(
            [
                getAddFacetCut(DS6Facet.address, [
                    ds6Facet.interface.getSighash("getX()"),
                    ds6Facet.interface.getSighash("setX(uint256)"),
                ]),
            ],
            ds6Upgrade.address,
            ds6Upgrade.interface.encodeFunctionData("upgrade", [x])
        );

        // Check storage
        expect(await ds6Facet.getX()).to.equal(x);
        expect(await ds6Facet.getMappingEntry(k)).to.equal(a);
        expect(await ds6Facet.getArrayLength()).to.equal(n);
        for (let i = 0; i < n; ++i) {
            expect(await ds6Facet.getArrayElement(i)).to.equal(i * b + c);
        }
    });

    it("diamondCut(): remove field in-place", async () => {
        // Add functions from DS6Facet
        let x = 123;
        const { DS6Facet } = await deployments.all();
        const ds6Selectors = getFunctionSelectors(ds6Facet);
        await diamondCut(
            [getAddFacetCut(DS6Facet.address, ds6Selectors)],
            ds6Init.address,
            ds6Init.interface.encodeFunctionData("init", [x])
        );

        // Check storage variables
        const k = 111;
        expect(await ds6Facet.getX()).to.equal(x);
        expect(await ds6Facet.getMappingEntry(k)).to.equal(0);
        expect(await ds6Facet.getArrayLength()).to.equal(0);

        // Write to storage
        const a = 42;
        await ds6Facet.setMappingEntry(k, a);
        const n = 10;
        const b = 333;
        const c = 777;
        for (let i = 0; i < n; ++i) {
            await ds6Facet.addArrayElement(i * b + c);
        }

        // Downgrade to DS5
        // (remove field x)
        await diamondCut(
            [
                getRemoveFacetCut([
                    ds6Facet.interface.getSighash("getX()"),
                    ds6Facet.interface.getSighash("setX(uint256)"),
                ]),
            ],
            ds6Downgrade.address,
            ds6Downgrade.interface.encodeFunctionData("downgrade")
        );

        // Check storage variables
        expect(await ds5Facet.getMappingEntry(k)).to.equal(a);
        expect(await ds5Facet.getArrayLength()).to.equal(n);
        for (let i = 0; i < n; ++i) {
            expect(await ds5Facet.getArrayElement(i)).to.equal(i * b + c);
        }
    });

    it("diamondCut(): replace field in-place", async () => {
        // Add DS6Facet
        let x = BigNumber.from(2).pow(256).sub(1);
        const { DS6Facet } = await deployments.all();
        const ds6Selectors = getFunctionSelectors(ds6Facet);
        await diamondCut(
            [getAddFacetCut(DS6Facet.address, ds6Selectors)],
            ds6Init.address,
            ds6Init.interface.encodeFunctionData("init", [x])
        );

        // Check storage variables
        const k = 111;
        expect(await ds6Facet.getX()).to.equal(x);
        expect(await ds6Facet.getMappingEntry(k)).to.equal(0);
        expect(await ds6Facet.getArrayLength()).to.equal(0);

        // Write to storage
        const a = 42;
        await ds6Facet.setMappingEntry(k, a);
        let n = 10;
        const b = 333;
        const c = 777;
        for (let i = 0; i < n; ++i) {
            await ds6Facet.addArrayElement(i * b + c);
        }

        // Upgrade to DS7
        // (change x from 256 to 128 bits)
        // (add field y)
        let y = 123;
        const { DS7Facet } = await deployments.all();
        await diamondCut(
            [
                getAddFacetCut(DS7Facet.address, [
                    ds7Facet.interface.getSighash("getY()"),
                    ds7Facet.interface.getSighash("setY(uint128)"),
                    ds7Facet.interface.getSighash("setX(uint128)"),
                ]),
                getReplaceFacetCut(DS7Facet.address, [
                    ds6Facet.interface.getSighash("getX()"),
                ]),
                getRemoveFacetCut([
                    ds6Facet.interface.getSighash("setX(uint256)"),
                ]),
            ],
            ds7Upgrade.address,
            ds7Upgrade.interface.encodeFunctionData("upgrade", [y])
        );

        // Check storage variables
        x = BigNumber.from(2).pow(128).sub(1);
        expect(await ds7Facet.getX()).to.equal(x);
        expect(await ds7Facet.getY()).to.equal(y);
        expect(await ds7Facet.getMappingEntry(k)).to.equal(a);
        expect(await ds7Facet.getArrayLength()).to.equal(n);
        for (let i = 0; i < n; ++i) {
            expect(await ds7Facet.getArrayElement(i)).to.equal(i * b + c);
        }

        // Write to storage
        x = BigNumber.from(456);
        y = 789;
        await ds7Facet.setX(x);
        await ds7Facet.setY(y);
        const k2 = 800;
        const a2 = 666;
        await ds7Facet.setMappingEntry(k2, a2);
        for (let i = n; i < 2 * n; ++i) {
            await ds7Facet.addArrayElement(i * b + c);
        }
        n = 2 * n;

        // Check storage variables
        expect(await ds7Facet.getX()).to.equal(x);
        expect(await ds7Facet.getY()).to.equal(y);
        expect(await ds7Facet.getMappingEntry(k)).to.equal(a);
        expect(await ds7Facet.getMappingEntry(k2)).to.equal(a2);
        expect(await ds7Facet.getArrayLength()).to.equal(n);
        for (let i = 0; i < n; ++i) {
            expect(await ds7Facet.getArrayElement(i)).to.equal(i * b + c);
        }
    });

    it("diamondCut(): add dynamic field in-place", async () => {
        let x = 10;
        let y = 42;

        // Add `getX`, `setX`, `getY` and `setY` functions
        const { DS1Facet } = await deployments.all();
        const ds1Selectors = getFunctionSelectors(ds1Facet);
        await diamondCut(
            [getAddFacetCut(DS1Facet.address, ds1Selectors)],
            ds1Init.address,
            ds1Init.interface.encodeFunctionData("init", [x, y])
        );

        // Upgrade to DS8 (no initialization routine)
        const { DS8Facet } = await deployments.all();
        await diamondCut([
            getAddFacetCut(DS8Facet.address, [
                ds8Facet.interface.getSighash("getArrayLength()"),
                ds8Facet.interface.getSighash("getArrayElement(uint256)"),
                ds8Facet.interface.getSighash("getMappingEntry(uint256)"),
                ds8Facet.interface.getSighash("addArrayElement(uint256)"),
                ds8Facet.interface.getSighash(
                    "setMappingEntry(uint256, uint256)"
                ),
            ]),
        ]);

        // Check state variables
        expect(await ds8Facet.getX()).to.equal(x);
        expect(await ds8Facet.getY()).to.equal(y);
        expect(await ds8Facet.getArrayLength()).to.equal(0);

        // Change state variables
        x = 22;
        y = 13;
        const arr = [45, 76, 34, 44, 211];
        const mapKey1 = 100;
        const mapKey2 = 333;
        const mapVal1 = 44;
        const mapVal2 = 5555;
        await ds8Facet.setX(x);
        await ds8Facet.setY(y);
        for (const arrElem of arr) {
            await ds8Facet.addArrayElement(arrElem);
        }
        await ds8Facet.setMappingEntry(mapKey1, mapVal1);
        await ds8Facet.setMappingEntry(mapKey2, mapVal2);

        // Check state variables
        expect(await ds8Facet.getX()).to.equal(x);
        expect(await ds8Facet.getY()).to.equal(y);
        expect(await ds8Facet.getArrayLength()).to.equal(arr.length);
        let i = 0;
        for (const arrElem of arr) {
            expect(await ds8Facet.getArrayElement(i)).to.equal(arrElem);
            i = i + 1;
        }
        expect(await ds8Facet.getMappingEntry(mapKey1)).to.equal(mapVal1);
        expect(await ds8Facet.getMappingEntry(mapKey2)).to.equal(mapVal2);
    });

    it("diamondCut(): add dynamic field on new struct", async () => {
        let x = 10;
        let y = 42;

        // Add `getX`, `setX`, `getY` and `setY` functions
        const { DS1Facet } = await deployments.all();
        const ds1Selectors = getFunctionSelectors(ds1Facet);
        await diamondCut(
            [getAddFacetCut(DS1Facet.address, ds1Selectors)],
            ds1Init.address,
            ds1Init.interface.encodeFunctionData("init", [x, y])
        );

        // Upgrade to DS9
        const { DS9Facet } = await deployments.all();
        await diamondCut(
            [
                getAddFacetCut(DS9Facet.address, [
                    ds9Facet.interface.getSighash("getArrayLength()"),
                    ds9Facet.interface.getSighash("getArrayElement(uint256)"),
                    ds9Facet.interface.getSighash("getMappingEntry(uint256)"),
                    ds9Facet.interface.getSighash("addArrayElement(uint256)"),
                    ds9Facet.interface.getSighash(
                        "setMappingEntry(uint256, uint256)"
                    ),
                ]),
                getReplaceFacetCut(DS9Facet.address, ds1Selectors),
            ],
            ds9Upgrade.address,
            ds9Upgrade.interface.encodeFunctionData("upgrade")
        );

        // Check state variables
        expect(await ds9Facet.getX()).to.equal(x);
        expect(await ds9Facet.getY()).to.equal(y);
        expect(await ds9Facet.getArrayLength()).to.equal(0);

        // Change state variables
        x = 22;
        y = 13;
        const arr = [45, 76, 34, 44, 211];
        const mapKey1 = 100;
        const mapKey2 = 333;
        const mapVal1 = 44;
        const mapVal2 = 5555;
        await ds9Facet.setX(x);
        await ds9Facet.setY(y);
        for (const arrElem of arr) {
            await ds9Facet.addArrayElement(arrElem);
        }
        await ds9Facet.setMappingEntry(mapKey1, mapVal1);
        await ds9Facet.setMappingEntry(mapKey2, mapVal2);

        // Check state variables
        expect(await ds9Facet.getX()).to.equal(x);
        expect(await ds9Facet.getY()).to.equal(y);
        expect(await ds9Facet.getArrayLength()).to.equal(arr.length);
        let i = 0;
        for (const arrElem of arr) {
            expect(await ds9Facet.getArrayElement(i)).to.equal(arrElem);
            i = i + 1;
        }
        expect(await ds9Facet.getMappingEntry(mapKey1)).to.equal(mapVal1);
        expect(await ds9Facet.getMappingEntry(mapKey2)).to.equal(mapVal2);
    });

    it("diamondCut(): remove dynamic field on new struct", async () => {
        // Add DS7Facet functions
        let x = 123;
        let y = 456;
        const { DS7Facet } = await deployments.all();
        await diamondCut(
            [getAddFacetCut(DS7Facet.address, getFunctionSelectors(ds7Facet))],
            ds7Init.address,
            ds7Init.interface.encodeFunctionData("init", [x, y])
        );

        // Check state variables
        expect(await ds7Facet.getX()).to.equal(x);
        expect(await ds7Facet.getY()).to.equal(y);
        expect(await ds7Facet.getArrayLength()).to.equal(0);

        // Change state variables
        x = 789;
        y = 111;
        await ds7Facet.setX(x);
        await ds7Facet.setY(y);
        for (let i = 0; i < 10; ++i) {
            await ds7Facet.addArrayElement(55 * i + 37);
            await ds7Facet.setMappingEntry(i, 12 * i + 3);
        }

        // Downgrade to DS1
        const { DS1Facet } = await deployments.all();
        await diamondCut(
            [
                getRemoveFacetCut([
                    ds7Facet.interface.getSighash("getArrayLength()"),
                    ds7Facet.interface.getSighash("getArrayElement(uint256)"),
                    ds7Facet.interface.getSighash("getMappingEntry(uint256)"),
                    ds7Facet.interface.getSighash("addArrayElement(uint256)"),
                    ds7Facet.interface.getSighash(
                        "setMappingEntry(uint256, uint256)"
                    ),
                    ds7Facet.interface.getSighash("setX(uint128)"),
                    ds7Facet.interface.getSighash("setY(uint128)"),
                ]),
                getReplaceFacetCut(DS1Facet.address, [
                    ds7Facet.interface.getSighash("getX()"),
                    ds7Facet.interface.getSighash("getY()"),
                ]),
                getAddFacetCut(DS1Facet.address, [
                    ds1Facet.interface.getSighash("setX(uint32)"),
                    ds1Facet.interface.getSighash("setY(uint32)"),
                ]),
            ],
            ds7Downgrade.address,
            ds7Downgrade.interface.encodeFunctionData("downgrade")
        );

        // Check state variables
        expect(await ds1Facet.getX()).to.equal(x);
        expect(await ds1Facet.getY()).to.equal(y);
    });
});
