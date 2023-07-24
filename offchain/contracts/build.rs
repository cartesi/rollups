// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use state_fold_types::contract;
use std::fs::File;

macro_rules! path {
    ($contract_file: expr, $contract_name: expr) => {
        match $contract_name {
            "ERC20" => "../../onchain/rollups/abi/@openzeppelin/contracts/token/ERC20/ERC20.sol/ERC20.json".to_owned(),
            _ => format!("../../onchain/rollups/abi/contracts/{}.sol/{}.json", $contract_file, $contract_name),
        }
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contracts = vec![
        ("InputBox", "inputs/InputBox", "input_box.rs"),
        ("Authority", "consensus/authority/Authority", "authority.rs"),
        ("History", "history/History", "history.rs"),
        // ("CartesiDApp", "dapp/CartesiDApp", "cartesi_dapp.rs"),
        // (
        //     "CartesiDAppFactory",
        //     "dapp/CartesiDAppFactory",
        //     "dapp_factory.rs",
        // ),
        // ("ERC20", "ERC20", "erc20_contract.rs"),
    ];

    for (contract_name, file, bindings_file_name) in contracts {
        let source_path = path!(file, contract_name);
        let output_path = format!(
            "{}/{}",
            std::env::var("OUT_DIR").unwrap(),
            bindings_file_name
        );

        println!("cargo:rerun-if-changed={}", source_path);
        println!("cargo:rerun-if-changed={}", output_path);

        let source = File::open(&source_path)?;
        let output = File::create(&output_path)?;

        contract::write(contract_name, source, output)?;
    }

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
