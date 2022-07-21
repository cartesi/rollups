use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use state_fold_types::ethers;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RollupsInitialState {
    pub dapp_contract_address: Arc<Address>,
    pub initial_epoch: U256,
}
