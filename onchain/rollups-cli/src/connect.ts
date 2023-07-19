// Copyright Cartesi Pte. Ltd.

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
    CartesiDAppFactory,
    CartesiDAppFactory__factory,
    AuthorityFactory,
    AuthorityFactory__factory,
    AuthorityHistoryPairFactory,
    AuthorityHistoryPairFactory__factory,
} from "@cartesi/rollups";
import goerli from "@cartesi/rollups/export/abi/goerli.json";
import sepolia from "@cartesi/rollups/export/abi/sepolia.json";
import polygon_mumbai from "@cartesi/rollups/export/abi/polygon_mumbai.json";
import arbitrum_goerli from "@cartesi/rollups/export/abi/arbitrum_goerli.json";
import optimism_goerli from "@cartesi/rollups/export/abi/optimism_goerli.json";
import bsc_testnet from "@cartesi/rollups/export/abi/bsc_testnet.json";
import iotex_testnet from "@cartesi/rollups/export/abi/iotex_testnet.json";
import chiado from "@cartesi/rollups/export/abi/chiado.json";

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
    5: goerli,
    11155111: sepolia,
    80001: polygon_mumbai,
    421613: arbitrum_goerli,
    420: optimism_goerli,
    97: bsc_testnet,
    4690: iotex_testnet,
    10200: chiado,
};

function getContractConnector<T>(contractName: string, contractFactory: any) {
    return async (
        rpc: string,
        mnemonic?: string,
        accountIndex?: number,
        deploymentPath?: string
    ): Promise<T> => {
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
                address = deployment.contracts[contractName].address;
                break;
            default:
                const networkDeployment = deployments[chainId];
                if (!networkDeployment) {
                    throw new Error(`unsupported network ${chainId}`);
                }
                address = networkDeployment.contracts[contractName].address;
        }
        // connect to contracts
        return contractFactory.connect(address, signer || provider);
    };
}

export const dappFactory = getContractConnector<CartesiDAppFactory>(
    "CartesiDAppFactory",
    CartesiDAppFactory__factory
);

export const authorityFactory = getContractConnector<AuthorityFactory>(
    "AuthorityFactory",
    AuthorityFactory__factory
);

export const authorityHistoryPairFactory =
    getContractConnector<AuthorityHistoryPairFactory>(
        "AuthorityHistoryPairFactory",
        AuthorityHistoryPairFactory__factory
    );
