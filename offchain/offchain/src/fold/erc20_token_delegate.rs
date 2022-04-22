use contracts::erc20_contract::*;

use super::types::ERC20BalanceState;

use offchain_core::types::Block;
use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils,
};

use async_trait::async_trait;
use snafu::ResultExt;

use ethers::prelude::EthEvent;
use ethers::types::{Address, U256};

/// erc20 token delegate
#[derive(Default)]
pub struct ERC20BalanceFoldDelegate {}

#[async_trait]
impl StateFoldDelegate for ERC20BalanceFoldDelegate {
    type InitialState = (Address, Address); // erc20 contract address, owner address
    type Accumulator = ERC20BalanceState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &Self::InitialState,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let (erc20_address, owner_address) = *initial_state;

        let erc20_contract = access
            .build_sync_contract(erc20_address, block.number, ERC20::new)
            .await;

        // `Transfer` events
        // topic1: from
        let erc20_transfer_from_events = erc20_contract
            .transfer_filter()
            .topic1(owner_address)
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for erc20 transfer events from owner",
            })?;
        // topic2: to
        let erc20_transfer_to_events = erc20_contract
            .transfer_filter()
            .topic2(owner_address)
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for erc20 transfer events to owner",
            })?;
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

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
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

        let erc20_contract = access
            .build_fold_contract(erc20_address, block.hash, ERC20::new)
            .await;

        let mut state = previous_state.clone();

        // `Transfer` events
        // topic1: from
        let erc20_transfer_from_events = erc20_contract
            .transfer_filter()
            .topic1(state.owner_address)
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for erc20 transfer events from owner",
            })?;
        // topic2: to
        let erc20_transfer_to_events = erc20_contract
            .transfer_filter()
            .topic2(state.owner_address)
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for erc20 transfer events to owner",
            })?;

        // combine both types of events to calculate balance
        state.balance = new_balance(
            state.balance,
            erc20_transfer_from_events,
            erc20_transfer_to_events,
        );

        Ok(state)
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
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
