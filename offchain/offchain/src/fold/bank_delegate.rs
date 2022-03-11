use crate::contracts::bank_contract::*;

use super::types::BankState;

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

/// bank delegate
#[derive(Default)]
pub struct BankFoldDelegate {}

#[async_trait]
impl StateFoldDelegate for BankFoldDelegate {
    type InitialState = (Address, Address); // bank address, dapp address
    type Accumulator = BankState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &Self::InitialState,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let (bank_address, dapp_address) = *initial_state;

        let bank_contract = access
            .build_sync_contract(bank_address, block.number, Bank::new)
            .await;

        // `Deposit` events
        // topic1: `to` address, same as dapp address
        let deposit_events = bank_contract
            .deposit_filter()
            .topic1(dapp_address)
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for bank deposit events to dapp",
            })?;

        // `Transfer` events
        // topic1: `from` address, same as dapp address
        let transfer_events = bank_contract
            .transfer_filter()
            .topic1(dapp_address)
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for bank transfer events from dapp",
            })?;

        // combine both types of events to calculate balance
        let balance =
            new_balance(U256::zero(), deposit_events, transfer_events);

        Ok(BankState {
            bank_address,
            dapp_address,
            balance,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let bank_address = previous_state.bank_address;

        // If not in bloom copy previous state
        if !(fold_utils::contains_address(&block.logs_bloom, &bank_address)
            && (fold_utils::contains_topic(
                &block.logs_bloom,
                &TransferFilter::signature(),
            ) || fold_utils::contains_topic(
                &block.logs_bloom,
                &DepositFilter::signature(),
            )))
        {
            return Ok(previous_state.clone());
        }

        let bank_contract = access
            .build_fold_contract(bank_address, block.hash, Bank::new)
            .await;

        let mut state = previous_state.clone();
        let dapp_address = state.dapp_address;

        // `Deposit` events
        // topic1: `to` address, same as dapp address
        let deposit_events = bank_contract
            .deposit_filter()
            .topic1(dapp_address)
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for bank deposit events to dapp",
            })?;

        // `Transfer` events
        // topic1: `from` address, same as dapp address
        let transfer_events = bank_contract
            .transfer_filter()
            .topic1(dapp_address)
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for bank transfer events from dapp",
            })?;

        // combine both types of events to calculate balance
        state.balance =
            new_balance(state.balance, deposit_events, transfer_events);

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
    deposit_events: Vec<DepositFilter>,
    transfer_events: Vec<TransferFilter>,
) -> U256 {
    let mut income: U256 = U256::zero();
    for ev in deposit_events.iter() {
        income = income + ev.value;
    }

    let mut expense: U256 = U256::zero();
    for ev in transfer_events.iter() {
        expense = expense + ev.value;
    }

    // balance = income - expense
    assert!(
        expense <= old_balance + income,
        "spend more than the owner has!"
    );

    old_balance + income - expense
}
