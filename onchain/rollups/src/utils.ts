// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the license at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

import { HardhatRuntimeEnvironment } from "hardhat/types";
import { IDiamondCut } from "./types";

export const productionFacetNames: string[] = [
    // essential facets
    "DiamondLoupeFacet",
    "OwnershipFacet",
    // rollups-related facets
    "ERC20PortalFacet",
    "ERC721PortalFacet",
    "EtherPortalFacet",
    "FeeManagerFacet",
    "InputFacet",
    "OutputFacet",
    "RollupsFacet",
    "ValidatorManagerFacet",
];

export const getFacetCuts = async (
    hre: HardhatRuntimeEnvironment,
    facetNames: string[]
) => {
    const { deployments, ethers } = hre;

    const facetCuts: IDiamondCut.FacetCutStruct[] = [];
    const functionSelectors: { [selector: string]: string } = {};

    for (const facetName of facetNames) {
        const facetDeployment = await deployments.get(facetName);
        const facet = await ethers.getContractAt(
            facetName,
            facetDeployment.address
        );
        let selectors: string[] = [];
        const signatures = Object.keys(facet.interface.functions);
        for (let signature of signatures) {
            if (signature !== "init(bytes") {
                const selector = facet.interface.getSighash(signature);
                if (selector in functionSelectors) {
                    const otherFacetName = functionSelectors[selector];
                    throw Error(
                        `Tried to add function ${signature} (${selector})` +
                            `from ${facetName} and ${otherFacetName}`
                    );
                }
                functionSelectors[selector] = facetName;
                selectors.push(selector);
            }
        }
        facetCuts.push({
            facetAddress: facet.address,
            action: 0, // FacetCutAction.Add
            functionSelectors: selectors,
        });
    }

    return facetCuts;
};
