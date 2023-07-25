// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

import path from "path";
import { HardhatUserConfig } from "hardhat/config";
import { HttpNetworkUserConfig } from "hardhat/types";

import "@nomiclabs/hardhat-ethers";
import "@nomicfoundation/hardhat-verify";
import "@typechain/hardhat";
import "hardhat-deploy";
import "hardhat-abi-exporter";
import "hardhat-gas-reporter";

import {
    Chain,
    arbitrum,
    arbitrumGoerli,
    mainnet,
    optimism,
    optimismGoerli,
    sepolia,
} from "@wagmi/chains";

// read MNEMONIC from env variable
let mnemonic = process.env.MNEMONIC;

const ppath = (packageName: string, pathname: string) => {
    return path.join(
        path.dirname(require.resolve(`${packageName}/package.json`)),
        pathname
    );
};

const networkConfig = (chain: Chain): HttpNetworkUserConfig => {
    let url = chain.rpcUrls.public.http.at(0);

    // support for infura and alchemy URLs through env variables
    if (process.env.INFURA_ID && chain.rpcUrls.infura?.http) {
        url = `${chain.rpcUrls.infura.http}/${process.env.INFURA_ID}`;
    } else if (process.env.ALCHEMY_ID && chain.rpcUrls.alchemy?.http) {
        url = `${chain.rpcUrls.alchemy.http}/${process.env.ALCHEMY_ID}`;
    }

    return {
        chainId: chain.id,
        url,
        accounts: mnemonic ? { mnemonic } : undefined,
    };
};

const config: HardhatUserConfig = {
    networks: {
        hardhat: mnemonic ? { accounts: { mnemonic } } : {},
        localhost: {
            url: process.env.RPC_URL || "http://localhost:8545",
            accounts: mnemonic ? { mnemonic } : undefined,
        },
        arbitrum: networkConfig(arbitrum),
        arbitrum_goerli: networkConfig(arbitrumGoerli),
        mainnet: networkConfig(mainnet),
        sepolia: networkConfig(sepolia),
        optimism: networkConfig(optimism),
        optimism_goerli: networkConfig(optimismGoerli),
    },
    solidity: {
        version: "0.8.19",
        settings: {
            optimizer: {
                enabled: true,
            },
        },
    },
    paths: {
        artifacts: "artifacts",
        deploy: "deploy",
        deployments: "deployments",
    },
    abiExporter: {
        runOnCompile: true,
        clear: true,
    },
    typechain: {
        outDir: "src/types",
        target: "ethers-v5",
    },
    etherscan: {
        apiKey: process.env.ETHERSCAN_API_KEY,
    },
    external: {
        contracts: [
            {
                artifacts: ppath("@cartesi/util", "/export/artifacts"),
                deploy: ppath("@cartesi/util", "/dist/deploy"),
            },
        ],
        deployments: {
            localhost: ["deployments/localhost"],
            arbitrum: [ppath("@cartesi/util", "/deployments/arbitrum")],
            arbitrum_goerli: [
                ppath("@cartesi/util", "/deployments/arbitrum_goerli"),
            ],
            mainnet: [ppath("@cartesi/util", "/deployments/mainnet")],
            optimism: [ppath("@cartesi/util", "/deployments/optimism")],
            optimism_goerli: [
                ppath("@cartesi/util", "/deployments/optimism_goerli"),
            ],
            sepolia: [ppath("@cartesi/util", "/deployments/sepolia")],
        },
    },
    namedAccounts: {
        deployer: {
            default: 0,
        },
    },
    gasReporter: {
        enabled: process.env.REPORT_GAS ? true : false,
    },
};

export default config;
