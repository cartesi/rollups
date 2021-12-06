pub mod versioning {
    tonic::include_proto!("versioning");
}

pub mod cartesi_machine {
    tonic::include_proto!("cartesi_machine");
}

pub mod rollup_machine_manager {
    tonic::include_proto!("cartesi_rollup_machine_manager");
}

pub mod state_server {
    tonic::include_proto!("state_server");
}
