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
        (
            "DiamondInit",
            "upgrade_initializers/DiamondInit",
            "diamond_init.rs",
        ),
        (
            "DiamondCutFacet",
            "facets/DiamondCutFacet",
            "diamond_cut_facet.rs",
        ),
        (
            "DiamondLoupeFacet",
            "facets/DiamondLoupeFacet",
            "diamond_loupe_facet.rs",
        ),
        (
            "ERC20PortalFacet",
            "facets/ERC20PortalFacet",
            "erc20_portal_facet.rs",
        ),
        (
            "ERC721PortalFacet",
            "facets/ERC721PortalFacet",
            "erc721_portal_facet.rs",
        ),
        (
            "ERC1155PortalFacet",
            "facets/ERC1155PortalFacet",
            "erc1155_portal_facet.rs",
        ),
        (
            "EtherPortalFacet",
            "facets/EtherPortalFacet",
            "ether_portal_facet.rs",
        ),
        (
            "FeeManagerFacet",
            "facets/FeeManagerFacet",
            "fee_manager_facet.rs",
        ),
        ("InputFacet", "facets/InputFacet", "input_facet.rs"),
        ("OutputFacet", "facets/OutputFacet", "output_facet.rs"),
        ("RollupsFacet", "facets/RollupsFacet", "rollups_facet.rs"),
        (
            "ValidatorManagerFacet",
            "facets/ValidatorManagerFacet",
            "validator_manager_facet.rs",
        ),
        ("Bank", "Bank", "bank_contract.rs"),
        ("ERC20", "ERC20", "erc20_contract.rs"),
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
