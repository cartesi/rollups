use crate::{input::EpochInputState, FoldableError};
use async_trait::async_trait;
use ethers::{
    providers::Middleware,
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use state_fold::{
    FoldMiddleware, Foldable, StateFoldEnvironment, SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::sync::Arc;

/// Active epoch currently receiving inputs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccumulatingEpoch {
    pub epoch_number: U256,
    pub inputs: EpochInputState,
    pub dapp_contract_address: Address,
}

impl AccumulatingEpoch {
    pub fn new(dapp_contract_address: Address, epoch_number: U256) -> Self {
        Self {
            epoch_number,
            inputs: EpochInputState::new(epoch_number, dapp_contract_address),
            dapp_contract_address,
        }
    }
}

#[async_trait]
impl Foldable for AccumulatingEpoch {
    type InitialState = (Address, U256);
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        _access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let (dapp_contract_address, epoch_number) = *initial_state;

        let inputs = EpochInputState::get_state_for_block(
            &(dapp_contract_address, epoch_number),
            block,
            env,
        )
        .await?
        .state;

        Ok(AccumulatingEpoch {
            inputs,
            epoch_number,
            dapp_contract_address,
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        _access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let epoch_number = previous_state.epoch_number.clone();
        let dapp_contract_address =
            previous_state.dapp_contract_address.clone();

        let inputs = EpochInputState::get_state_for_block(
            &(dapp_contract_address, epoch_number),
            block,
            env,
        )
        .await?
        .state;

        Ok(AccumulatingEpoch {
            epoch_number,
            inputs,
            dapp_contract_address,
        })
    }
}
