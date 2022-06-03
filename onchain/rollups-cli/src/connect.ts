// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

import fs from "fs";
import { JsonRpcProvider } from "@ethersproject/providers";
import { ethers } from "ethers";
import {
    InputFacet,
    InputFacet__factory,
    OutputFacet,
    OutputFacet__factory,
    ERC20PortalFacet,
    ERC20PortalFacet__factory,
    CartesiDAppFactory,
    CartesiDAppFactory__factory,
} from "@cartesi/rollups";
import polygon_mumbai from "@cartesi/rollups/export/abi/polygon_mumbai.json";
import { Chain, ChainInfo, networks } from "./networks";

interface RollupsContracts {
    chain: ChainInfo;
    inputContract: InputFacet;
    outputContract: OutputFacet;
    erc20Portal: ERC20PortalFacet;
}

export const rollups = (
    chainName: Chain,
    address: string,
    mnemonic?: string
): RollupsContracts => {
    const chain = networks[chainName];
    if (!chain) {
        throw new Error(`unsupported network: ${chainName}`);
    }
    // connect to JSON-RPC provider
    const provider = new JsonRpcProvider(chain.rpc);

    // create signer to be used to send transactions
    const signer = mnemonic
        ? ethers.Wallet.fromMnemonic(mnemonic).connect(provider)
        : undefined;

    // connect to contracts
    const inputContract = InputFacet__factory.connect(
        address,
        signer || provider
    );
    const outputContract = OutputFacet__factory.connect(
        address,
        signer || provider
    );
    const erc20Portal = ERC20PortalFacet__factory.connect(
        address,
        signer || provider
    );
    return {
        chain,
        inputContract,
        outputContract,
        erc20Portal,
    };
};

export const factory = (
    chainName: Chain,
    mnemonic?: string,
    accountIndex?: number,
    deploymentPath?: string
): CartesiDAppFactory => {
    const chain = networks[chainName];
    if (!chain) {
        throw new Error(`unsupported network: ${chainName}`);
    }
    // connect to JSON-RPC provider
    const provider = new JsonRpcProvider(chain.rpc);

    // create signer to be used to send transactions
    const signer = mnemonic
        ? ethers.Wallet.fromMnemonic(
              mnemonic,
              `m/44'/60'/0'/0/${accountIndex}`
          ).connect(provider)
        : undefined;

    let address;
    switch (chainName) {
        case "polygon_mumbai":
            address = polygon_mumbai.contracts.CartesiDAppFactory.address;
            break;
        case "localhost":
            if (!deploymentPath) {
                throw new Error(
                    `undefined deployment path for network ${chainName}`
                );
            }
            if (!fs.existsSync(deploymentPath)) {
                throw new Error(
                    `deployment file '${deploymentPath}' not found`
                );
            }
            const deployment = JSON.parse(
                fs.readFileSync(deploymentPath, "utf8")
            );
            address = deployment.contracts.CartesiDAppFactory.address;
    }
    // connect to contracts
    return CartesiDAppFactory__factory.connect(address, signer || provider);
};
