use crate::FoldableError;
use anyhow::Context;
use async_trait::async_trait;
use contracts::bank_contract::*;
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
pub struct BankState {
    pub bank_address: Address,
    pub dapp_address: Address,
    pub balance: U256,
}

#[async_trait]
impl Foldable for BankState {
    type InitialState = (Address, Address); // bank address, dapp address
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        _block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let (bank_address, dapp_address) = *initial_state;

        let bank_contract = Bank::new(bank_address, access);

        // `Deposit` events
        // topic1: `to` address, same as dapp address
        let deposit_events = bank_contract
            .deposit_filter()
            .topic1(dapp_address)
            .query()
            .await
            .context("Error querying for bank deposit events to dapp")?;

        // `Transfer` events
        // topic1: `from` address, same as dapp address
        let transfer_events = bank_contract
            .transfer_filter()
            .topic1(dapp_address)
            .query()
            .await
            .context("Error querying for bank transfer events from dapp")?;

        // combine both types of events to calculate balance
        let balance =
            new_balance(U256::zero(), deposit_events, transfer_events);

        Ok(BankState {
            bank_address,
            dapp_address,
            balance,
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
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

        let bank_contract = Bank::new(bank_address, access);

        let mut state = previous_state.clone();
        let dapp_address = state.dapp_address;

        // `Deposit` events
        // topic1: `to` address, same as dapp address
        let deposit_events = bank_contract
            .deposit_filter()
            .topic1(dapp_address)
            .query()
            .await
            .context("Error querying for bank deposit events to dapp")?;

        // `Transfer` events
        // topic1: `from` address, same as dapp address
        let transfer_events = bank_contract
            .transfer_filter()
            .topic1(dapp_address)
            .query()
            .await
            .context("Error querying for bank transfer events from dapp")?;

        // combine both types of events to calculate balance
        state.balance =
            new_balance(state.balance, deposit_events, transfer_events);

        Ok(state)
    }
}

fn new_balance(
    old_balance: U256,
    deposit_events: Vec<DepositFilter>,
    transfer_events: Vec<TransferFilter>,
) -> U256 {
    let mut income: U256 = U256::zero();
    for ev in deposit_events.iter() {
        // U256 is very unlikely to overflow
        // But developers should keep an eye on tokens
        // that has extremely high volume and turnover rate
        income = income + ev.value;
    }

    let mut expense: U256 = U256::zero();
    for ev in transfer_events.iter() {
        // U256 is very unlikely to overflow
        // But developers should keep an eye on tokens
        // that has extremely high volume and turnover rate
        expense = expense + ev.value;
    }

    // balance = income - expense
    assert!(
        expense <= old_balance + income,
        "spend more than the owner has!"
    );

    old_balance + income - expense
}
