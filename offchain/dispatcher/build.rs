fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_server(false)
        .compile(
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
