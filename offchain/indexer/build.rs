fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().build_server(false).compile(
        &[
            "../../grpc-interfaces/versioning.proto",
            "../../grpc-interfaces/server-manager.proto",
            "../../grpc-interfaces/stateserver.proto",
        ],
        &["../../grpc-interfaces"],
    )?;

    println!("cargo:rerun-if-changed=../../grpc-interfaces/versioning.proto");

    println!(
        "cargo:rerun-if-changed=../../grpc-interfaces/server-manager.proto"
    );

    println!("cargo:rerun-if-changed=../../grpc-interfaces/stateserver.proto");

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
