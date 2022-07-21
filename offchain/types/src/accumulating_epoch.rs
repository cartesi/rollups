use crate::{
    epoch_initial_state::EpochInitialState, input::EpochInputState,
    FoldableError,
};
use async_trait::async_trait;
use ethers::providers::Middleware;
use serde::{Deserialize, Serialize};
use state_fold::{
    FoldMiddleware, Foldable, StateFoldEnvironment, SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::sync::Arc;

/// Active epoch currently receiving inputs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccumulatingEpoch {
    pub inputs: Arc<EpochInputState>,
    pub epoch_initial_state: Arc<EpochInitialState>,
}

impl AccumulatingEpoch {
    pub fn new(epoch_initial_state: Arc<EpochInitialState>) -> Arc<Self> {
        Arc::new(Self {
            inputs: EpochInputState::new(Arc::clone(&epoch_initial_state)),
            epoch_initial_state,
        })
    }
}

#[async_trait]
impl Foldable for AccumulatingEpoch {
    type InitialState = Arc<EpochInitialState>;
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        _access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let inputs =
            EpochInputState::get_state_for_block(initial_state, block, env)
                .await?
                .state;

        Ok(Self {
            inputs,
            epoch_initial_state: Arc::clone(initial_state),
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        _access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let epoch_initial_state =
            Arc::clone(&previous_state.epoch_initial_state);

        let inputs = EpochInputState::get_state_for_block(
            &epoch_initial_state,
            block,
            env,
        )
        .await?
        .state;

        Ok(Self {
            inputs,
            epoch_initial_state,
        })
    }
}
