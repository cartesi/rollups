use state_fold::{Access, StateFold};
use state_server_grpc::state_server::delegate_manager_server::DelegateManager;
use state_server_grpc::state_server::{GetStateRequest, GetStateResponse};

use ethers::core::types::{Address, U256};
use ethers::providers::{Http, Provider};
use tonic::{Code, Request, Response, Status};

use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct InitialState {
    initial_epoch: U256,
    descartes_address: Address,
}

pub struct RollupsDelegateManager {
    pub fold: Arc<
        StateFold<
            offchain::fold::descartesv2_delegate::DescartesV2FoldDelegate<
                Access<Provider<Http>>,
            >,
            Access<Provider<Http>>,
        >,
    >,
}

#[tonic::async_trait]
impl DelegateManager for RollupsDelegateManager {
    async fn get_state(
        &self,
        request: Request<GetStateRequest>,
    ) -> Result<Response<GetStateResponse>, Status> {
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
        let descartes_address = initial_state.descartes_address;
        let initial_epoch = initial_state.initial_epoch;

        let contract_state = self
            .fold
            .get_state_for_block(&(descartes_address, initial_epoch), None)
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
