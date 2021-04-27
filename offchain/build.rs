use ethers::contract::Abigen;
use serde_json::Value;

fn write_contract(contract_name: &str, source: &str, destination: &str) {
    let s = std::fs::read_to_string(source).unwrap();
    let v: Value = serde_json::from_str(&s).unwrap();
    let abi_str = serde_json::to_string(&v["abi"]).unwrap();

    let bindings = Abigen::new(&contract_name, abi_str)
        .unwrap()
        .generate()
        .unwrap();

    bindings.write_to_file(destination).unwrap();

    let cargo_rerun = "cargo:rerun-if-changed";
    println!("{}={}", cargo_rerun, source);
    println!("{}={}", cargo_rerun, destination);
}

fn main() {
    let contracts = vec![
        (
            "DescartesV2Impl",
            "DescartesV2Impl",
            "descartesv2_contract.rs",
        ),
    ];

    for (name, file, rs) in contracts {
        let path = format!("../artifacts/contracts/{}.sol/{}.json", file, name);
        let destination = format!("./src/fold/contracts/{}", rs);
        write_contract(name, &path, &destination);
    }

    println!("cargo:rerun-if-changed=build.rs");
}