use ethers::contract::Abigen;

fn write_contract(
    contract_name: &str,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let abi_str = std::fs::read_to_string(source)?;
    let bindings = Abigen::new(&contract_name, abi_str)?.generate()?;

    bindings.write_to_file(destination)?;

    let cargo_rerun = "cargo:rerun-if-changed";
    println!("{}={}", cargo_rerun, source);
    println!("{}={}", cargo_rerun, destination);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contracts = vec![
        (
            "DiamondInit",
            "upgradeInitializers/DiamondInit",
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
            "SERC20PortalFacet",
            "facets/SERC20PortalFacet",
            "serc20_portal_facet.rs",
        ),
        (
            "ValidatorManagerFacet",
            "facets/ValidatorManagerFacet",
            "validator_manager_facet.rs",
        ),
    ];

    for (name, file, rs) in contracts {
        let path = format!(
            "../../onchain/rollups/abi/contracts/{}.sol/{}.json",
            file, name
        );
        let destination = format!("./src/contracts/{}", rs);
        write_contract(name, &path, &destination)?;
    }

    // create types for ERC20
    let path ="../../onchain/rollups/abi/@openzeppelin/contracts/token/ERC20/ERC20.sol/ERC20.json";
    let destination = "./src/contracts/erc20_contract.rs";
    write_contract("ERC20", &path, &destination)?;

    tonic_build::configure().build_server(false).compile(
        &[
            "../../grpc-interfaces/versioning.proto",
            "../../grpc-interfaces/cartesi-machine.proto",
            "../../grpc-interfaces/server-manager.proto",
            "../../grpc-interfaces/stateserver.proto",
        ],
        &["../../grpc-interfaces"],
    )?;

    println!("cargo:rerun-if-changed=../../grpc-interfaces/versioning.proto");
    println!(
        "cargo:rerun-if-changed=../../grpc-interfaces/cartesi-machine.proto"
    );
    println!(
        "cargo:rerun-if-changed=../../grpc-interfaces/server-manager.proto"
    );
    println!("cargo:rerun-if-changed=../../grpc-interfaces/stateserver.proto");

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
