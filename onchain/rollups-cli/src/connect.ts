// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

import fs from "fs";
import { JsonRpcProvider } from "@ethersproject/providers";
import { ethers } from "ethers";
import {
    Authority,
    Authority__factory,
    CartesiDAppFactory,
    CartesiDAppFactory__factory,
} from "@cartesi/rollups";
import arbitrum_goerli from "@cartesi/rollups/export/abi/arbitrum_goerli.json";
import optimism_goerli from "@cartesi/rollups/export/abi/optimism_goerli.json";
import sepolia from "@cartesi/rollups/export/abi/sepolia.json";

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
    421613: arbitrum_goerli,
    420: optimism_goerli,
    11155111: sepolia,
};

function getContractConnector<T>(contractName: string, contractFactory: any) {
    return async (
        rpc: string,
        mnemonic?: string,
        accountIndex?: number,
        deploymentPath?: string,
    ): Promise<T> => {
        // connect to JSON-RPC provider
        const provider = new JsonRpcProvider(rpc);

        // create signer to be used to send transactions
        const signer = mnemonic
            ? ethers.Wallet.fromMnemonic(
                  mnemonic,
                  `m/44'/60'/0'/0/${accountIndex}`,
              ).connect(provider)
            : undefined;

        const { chainId } = await provider.getNetwork();

        let address;
        switch (chainId) {
            case 31337: // hardhat
                if (!deploymentPath) {
                    throw new Error(
                        `undefined deployment path for network ${31337}`,
                    );
                }
                if (!fs.existsSync(deploymentPath)) {
                    throw new Error(
                        `deployment file '${deploymentPath}' not found`,
                    );
                }
                const deployment: Deployment = JSON.parse(
                    fs.readFileSync(deploymentPath, "utf8"),
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

export const authority = getContractConnector<Authority>(
    "Authority",
    Authority__factory,
);

export const factory = getContractConnector<CartesiDAppFactory>(
    "CartesiDAppFactory",
    CartesiDAppFactory__factory,
);
