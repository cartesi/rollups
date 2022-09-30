// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the license at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

import { deployments, ethers } from "hardhat";
import { Contract } from "ethers";
import { IDiamondCut } from "./types";

export enum FacetCutAction {
    Add = 0,
    Replace = 1,
    Remove = 2,
}

export const getAddFacetCut = (
    facetAddress: string,
    functionSelectors: string[]
): IDiamondCut.FacetCutStruct => {
    return {
        facetAddress,
        functionSelectors,
        action: FacetCutAction.Add,
    };
};

export const getReplaceFacetCut = (
    facetAddress: string,
    functionSelectors: string[]
): IDiamondCut.FacetCutStruct => {
    return {
        facetAddress,
        functionSelectors,
        action: FacetCutAction.Replace,
    };
};

export const getRemoveFacetCut = (
    functionSelectors: string[]
): IDiamondCut.FacetCutStruct => {
    return {
        facetAddress: ethers.constants.AddressZero,
        functionSelectors,
        action: FacetCutAction.Remove,
    };
};

export const productionFacetNames: string[] = [
    // essential facets
    "DiamondLoupeFacet",
    "OwnershipFacet",
    // rollups-related facets
    "ERC20PortalFacet",
    "ERC721PortalFacet",
    "ERC1155PortalFacet",
    "EtherPortalFacet",
    "FeeManagerFacet",
    "InputFacet",
    "OutputFacet",
    "RollupsFacet",
    "ValidatorManagerFacet",
];

export const getFunctionSelectors = (contract: Contract) => {
    let selectors: string[] = [];
    for (const signature in contract.interface.functions) {
        if (signature !== "init(bytes") {
            selectors.push(contract.interface.getSighash(signature));
        }
    }
    return selectors;
};

export const getFacetCuts = async (facetNames: string[]) => {
    const facetCuts: IDiamondCut.FacetCutStruct[] = [];
    const functionSelectors: { [selector: string]: string } = {};

    for (const facetName of facetNames) {
        const facetDeployment = await deployments.get(facetName);
        const facet = await ethers.getContractAt(
            facetDeployment.abi,
            facetDeployment.address
        );
        const selectors = getFunctionSelectors(facet);
        for (const selector of selectors) {
            if (selector in functionSelectors) {
                const otherFacetName = functionSelectors[selector];
                throw Error(
                    `Tried to add function selector ${selector} ` +
                        `from ${facetName} and ${otherFacetName}`
                );
            }
            functionSelectors[selector] = facetName;
        }
        facetCuts.push(getAddFacetCut(facet.address, selectors));
    }

    return facetCuts;
};
