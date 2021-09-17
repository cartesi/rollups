use state_fold::{Access, StateFold};
use state_server_grpc::state_server::delegate_manager_server::DelegateManager;
use state_server_grpc::state_server::{GetStateRequest, GetStateResponse};

use ethers::core::types::{Address, U256};
use ethers::providers::{Http, Provider};
use serde::{Deserialize, Serialize};
use tonic::{Code, Request, Response, Status};

pub struct InputDelegateManager {
    pub fold: StateFold<
        offchain::fold::input_delegate::InputFoldDelegate,
        Access<Provider<Http>>,
    >,
}

#[derive(Deserialize, Serialize)]
struct InitialState {
    pub epoch_number: U256,
    pub input_address: Address,
}

#[tonic::async_trait]
impl DelegateManager for InputDelegateManager {
    async fn get_state(
        &self,
        request: Request<GetStateRequest>,
    ) -> std::result::Result<Response<GetStateResponse>, Status> {
        let client = request.remote_addr();
        let initial_state = request.into_inner().json_initial_state;

        println!(
            "Got a request from {:?}, initial state: {}",
            client, initial_state
        );

        let initial_state: InitialState = serde_json::from_str(&initial_state)
            .map_err(|e| {
                Status::new(Code::InvalidArgument, format!("{}", e))
            })?;

        let contract_state = self
            .fold
            .get_state_for_block(
                &(initial_state.input_address, initial_state.epoch_number),
                None,
            )
            .await
            .map_err(|e| Status::new(Code::Unavailable, format!("{}", e)))?
            .state;

        let reply = GetStateResponse {
            json_state: serde_json::to_string(&contract_state)
                .map_err(|e| Status::new(Code::Unknown, format!("{}", e)))?,
        };

        Ok(Response::new(reply))
    }
}
