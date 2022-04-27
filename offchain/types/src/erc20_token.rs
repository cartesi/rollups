use crate::FoldableError;
use anyhow::Context;
use async_trait::async_trait;
use contracts::erc20_contract::*;
use ethers::{
    prelude::EthEvent,
    providers::Middleware,
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ERC20BalanceState {
    pub erc20_address: Address,
    pub owner_address: Address,
    pub balance: U256,
}

#[async_trait]
impl Foldable for ERC20BalanceState {
    type InitialState = (Address, Address); // erc20 contract address, owner address
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        _block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let (erc20_address, owner_address) = *initial_state;

        let middleware = access.get_inner();
        let erc20_contract = ERC20::new(erc20_address, middleware);

        // `Transfer` events
        // topic1: from
        let erc20_transfer_from_events = erc20_contract
            .transfer_filter()
            .topic1(owner_address)
            .query()
            .await
            .context("Error querying for erc20 transfer events from owner")?;
        // topic2: to
        let erc20_transfer_to_events = erc20_contract
            .transfer_filter()
            .topic2(owner_address)
            .query()
            .await
            .context("Error querying for erc20 transfer events to owner")?;
        // combine both types of events to calculate balance
        let balance = new_balance(
            U256::zero(),
            erc20_transfer_from_events,
            erc20_transfer_to_events,
        );

        Ok(ERC20BalanceState {
            erc20_address,
            owner_address,
            balance,
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        _access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let erc20_address = previous_state.erc20_address;

        // If not in bloom copy previous state
        if !(fold_utils::contains_address(&block.logs_bloom, &erc20_address)
            && fold_utils::contains_topic(
                &block.logs_bloom,
                &TransferFilter::signature(),
            ))
        {
            return Ok(previous_state.clone());
        }

        let middleware = env.inner_middleware();
        let erc20_contract = ERC20::new(erc20_address, middleware);

        let mut state = previous_state.clone();

        // `Transfer` events
        // topic1: from
        let erc20_transfer_from_events = erc20_contract
            .transfer_filter()
            .topic1(state.owner_address)
            .query()
            .await
            .context("Error querying for erc20 transfer events from owner")?;
        // topic2: to
        let erc20_transfer_to_events = erc20_contract
            .transfer_filter()
            .topic2(state.owner_address)
            .query()
            .await
            .context("Error querying for erc20 transfer events to owner")?;

        // combine both types of events to calculate balance
        state.balance = new_balance(
            state.balance,
            erc20_transfer_from_events,
            erc20_transfer_to_events,
        );

        Ok(state)
    }
}

fn new_balance(
    old_balance: U256,
    from_events: Vec<TransferFilter>,
    to_events: Vec<TransferFilter>,
) -> U256 {
    let mut income: U256 = U256::zero();
    for ev in to_events.iter() {
        income = income + ev.value;
    }

    let mut expense: U256 = U256::zero();
    for ev in from_events.iter() {
        expense = expense + ev.value;
    }

    // balance = income - expense
    assert!(
        expense <= old_balance + income,
        "spend more than the owner has!"
    );

    old_balance + income - expense
}
