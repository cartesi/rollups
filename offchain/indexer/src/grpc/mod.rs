pub mod cartesi_machine;
pub mod cartesi_server_manager;

pub mod versioning {
    tonic::include_proto!("versioning");
}

// pub mod cartesi_machine {
//     tonic::include_proto!("cartesi_machine");
// }
//
// pub mod cartesi_server_manager {
//     tonic::include_proto!("cartesi_server_manager");
// }

pub mod state_server {
    tonic::include_proto!("state_server");
}
