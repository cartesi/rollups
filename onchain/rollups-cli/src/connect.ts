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
import ropsten from "@cartesi/rollups/export/abi/ropsten.json";
import rinkeby from "@cartesi/rollups/export/abi/rinkeby.json";
import goerli from "@cartesi/rollups/export/abi/goerli.json";
import kovan from "@cartesi/rollups/export/abi/kovan.json";
import bsc_testnet from "@cartesi/rollups/export/abi/bsc_testnet.json";
import avax_fuji from "@cartesi/rollups/export/abi/avax_fuji.json";
import polygon_mumbai from "@cartesi/rollups/export/abi/polygon_mumbai.json";

type DeploymentContract = {
    address: string;
    abi: any[];
};

type Deployment = {
    name: string;
    chainId: string;
    contracts: Record<string, DeploymentContract>;
};

const deployments: Record<number, Deployment> = {
    3: ropsten,
    4: rinkeby,
    5: goerli,
    42: kovan,
    97: bsc_testnet,
    43113: avax_fuji,
    80001: polygon_mumbai,
};

interface RollupsContracts {
    inputContract: InputFacet;
    outputContract: OutputFacet;
    erc20Portal: ERC20PortalFacet;
}

export const rollups = (
    rpc: string,
    address: string,
    mnemonic?: string
): RollupsContracts => {
    // connect to JSON-RPC provider
    const provider = new JsonRpcProvider(rpc);

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
        inputContract,
        outputContract,
        erc20Portal,
    };
};

export const factory = async (
    rpc: string,
    mnemonic?: string,
    accountIndex?: number,
    deploymentPath?: string
): Promise<CartesiDAppFactory> => {
    // connect to JSON-RPC provider
    const provider = new JsonRpcProvider(rpc);

    // create signer to be used to send transactions
    const signer = mnemonic
        ? ethers.Wallet.fromMnemonic(
              mnemonic,
              `m/44'/60'/0'/0/${accountIndex}`
          ).connect(provider)
        : undefined;

    const { chainId } = await provider.getNetwork();

    let address;
    switch (chainId) {
        case 31337: // hardhat
            if (!deploymentPath) {
                throw new Error(
                    `undefined deployment path for network ${31337}`
                );
            }
            if (!fs.existsSync(deploymentPath)) {
                throw new Error(
                    `deployment file '${deploymentPath}' not found`
                );
            }
            const deployment: Deployment = JSON.parse(
                fs.readFileSync(deploymentPath, "utf8")
            );
            address = deployment.contracts["CartesiDAppFactory"].address;
            break;
        default:
            const networkDeployment = deployments[chainId];
            if (!networkDeployment) {
                throw new Error(`unsupported network ${chainId}`);
            }
            address = networkDeployment.contracts["CartesiDAppFactory"].address;
    }
    // connect to contracts
    return CartesiDAppFactory__factory.connect(address, signer || provider);
};
