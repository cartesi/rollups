use crate::FoldableError;

use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{
    ethers::{
        contract::LogMeta, prelude::EthEvent, providers::Middleware,
        types::Address,
    },
    Block,
};

use contracts::input_box::*;

use anyhow::{ensure, Context};
use async_trait::async_trait;
use im::Vector;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct DAppInputBoxInitialState {
    contract_address: Address,
    dapp_address: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    pub sender: Address,
    pub payload: Vec<u8>,
    pub block_added: Arc<Block>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DAppInputBox {
    pub input_box_initial_state: Arc<DAppInputBoxInitialState>,
    pub inputs: Arc<Vector<Arc<Input>>>,
}

#[async_trait]
impl Foldable for DAppInputBox {
    type InitialState = Arc<DAppInputBoxInitialState>;
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        _block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let input_box_initial_state = initial_state.clone();
        let contract_address = input_box_initial_state.contract_address;

        Ok(Self {
            input_box_initial_state,
            inputs: Arc::new(
                fetch_inputs(access, env, contract_address, None).await?,
            ),
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block, // TODO: when new version of state-fold gets released, change this to Arc
        // and save on cloning.
        env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let input_box_contract_address =
            previous_state.input_box_initial_state.contract_address;

        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &input_box_contract_address,
        ) && (fold_utils::contains_topic(
            &block.logs_bloom,
            &InputAddedFilter::signature(),
        ))) {
            return Ok(previous_state.clone());
        }

        let new_inputs = fetch_inputs(
            access,
            env,
            input_box_contract_address,
            Some(block.clone()),
        )
        .await?;

        let mut inputs = (*previous_state.inputs).clone();
        inputs.append(new_inputs);

        Ok(Self {
            input_box_initial_state: previous_state
                .input_box_initial_state
                .clone(),
            inputs: Arc::new(inputs),
        })
    }
}

async fn fetch_inputs<M1: Middleware + 'static, M2: Middleware + 'static>(
    provider: Arc<M1>,
    env: &StateFoldEnvironment<M2, ()>,
    contract_address: Address,
    block_opt: Option<Block>, // TODO: Option<Arc<Block>>,
) -> Result<Vector<Arc<Input>>, FoldableError> {
    let contract = InputBox::new(contract_address, Arc::clone(&provider));

    // Retrieve `InputAdded` events
    let inputs_futures: Vec<_> = contract
        .input_added_filter()
        .query_with_meta()
        .await
        .context("Error querying for input added events")?
        .into_iter()
        .map(|(e, meta)| Input::build_input(env, e, meta, &block_opt))
        .collect();

    let inputs_results = futures::future::join_all(inputs_futures).await;

    let inputs = {
        let inputs: Result<Vec<Arc<Input>>, _> =
            inputs_results.into_iter().collect();

        inputs?.into()
    };

    Ok(inputs)
}

impl Input {
    async fn build_input<M: Middleware + 'static>(
        env: &StateFoldEnvironment<M, ()>,
        event: InputAddedFilter,
        meta: LogMeta,
        block_opt: &Option<Block>, // TODO: &Option<Arc<Block>>
    ) -> Result<Arc<Self>, FoldableError> {
        let block =
            match block_opt {
                Some(ref b) => Arc::new(b.clone()), // TODO: remove Arc::new

                None => env.block_with_hash(&meta.block_hash).await.context(
                    format!("Could not query block `{:?}`", meta.block_hash),
                )?,
            };

        meta_consistent_with_block(&meta, &block)?;

        Ok(Arc::new(Self {
            sender: event.sender,
            payload: event.input.to_vec(),
            block_added: block,
        }))
    }
}

fn meta_consistent_with_block(
    meta: &LogMeta,
    block: &Block,
) -> Result<(), anyhow::Error> {
    ensure!(
        meta.block_hash == block.hash,
        "Sanity check failed: meta and block `block_hash` do not match"
    );

    ensure!(
        meta.block_number == block.number,
        "Sanity check failed: meta and block `block_number` do not match"
    );

    Ok(())
}
