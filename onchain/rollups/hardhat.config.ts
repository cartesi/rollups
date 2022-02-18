// Copyright (C) 2020 Cartesi Pte. Ltd.

// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.

// This program is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
// PARTICULAR PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Note: This component currently has dependencies that are licensed under the GNU
// GPL, version 3, and so you should treat this component as a whole as being under
// the GPL version 3. But all Cartesi-written code in this component is licensed
// under the Apache License, version 2, or a compatible permissive license, and can
// be used independently under the Apache v2 license. After this component is
// rewritten, the entire component will be released under the Apache v2 license.

import path from "path";
import { HardhatUserConfig } from "hardhat/config";
import { HttpNetworkUserConfig } from "hardhat/types";

import "@nomiclabs/hardhat-ethers";
import "@nomiclabs/hardhat-waffle";
import "@typechain/hardhat";
import "hardhat-deploy";
import "@tenderly/hardhat-tenderly";
import "hardhat-abi-exporter";

// read MNEMONIC from env variable
let mnemonic = process.env.MNEMONIC;

const ppath = (packageName: string, pathname: string) => {
    return path.join(
        path.dirname(require.resolve(`${packageName}/package.json`)),
        pathname
    );
};

const infuraNetwork = (
    network: string,
    chainId?: number,
    gas?: number
): HttpNetworkUserConfig => {
    return {
        url: `https://${network}.infura.io/v3/${process.env.PROJECT_ID}`,
        chainId,
        gas,
        accounts: mnemonic ? { mnemonic } : undefined,
    };
};

const config: HardhatUserConfig = {
    networks: {
        hardhat: mnemonic ? { accounts: { mnemonic } } : {},
        localhost: {
            url: "http://localhost:8545",
            accounts: mnemonic ? { mnemonic } : undefined,
        },
        mainnet: infuraNetwork("mainnet", 1, 6283185),
        rinkeby: infuraNetwork("rinkeby", 4, 6283185),
        kovan: infuraNetwork("kovan", 42, 6283185),
        goerli: infuraNetwork("goerli", 5, 6283185),
        polygon_mumbai: infuraNetwork("polygon-mumbai", 80001),
        bsc_testnet: {
            url: "https://data-seed-prebsc-1-s1.binance.org:8545",
            chainId: 97,
            accounts: mnemonic ? { mnemonic } : undefined,
        },
    },
    solidity: {
        version: "0.8.9",
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
    external: {
        contracts: [
            {
                artifacts: ppath("@cartesi/util", "/export/artifacts"),
                deploy: ppath("@cartesi/util", "/dist/deploy"),
            },
        ],
        deployments: {
            localhost: ["deployments/localhost"],
            ropsten: [ppath("@cartesi/util", "/deployments/ropsten")],
            rinkeby: [ppath("@cartesi/util", "/deployments/rinkeby")],
            kovan: [ppath("@cartesi/util", "/deployments/kovan")],
            goerli: [ppath("@cartesi/util", "/deployments/goerli")],
            polygon_mumbai: [
                ppath("@cartesi/util", "/deployments/matic_testnet"),
            ],
            bsc_testnet: [ppath("@cartesi/util", "/deployments/bsc_testnet")],
        },
    },
    namedAccounts: {
        deployer: {
            default: 0,
        },
    },
};

export default config;
