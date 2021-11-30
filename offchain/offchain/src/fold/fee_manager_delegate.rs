use crate::contracts::fee_manager_contract::*;
use crate::contracts::erc20_contract::*;

use super::types::FeeManagerState;

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

use im::HashMap;
use crate::contracts::voucher_contract::VoucherExecutedFilter;

/// Fee Manager Delegate
#[derive(Default)]
pub struct FeeManagerFoldDelegate {}

#[async_trait]
impl StateFoldDelegate for FeeManagerFoldDelegate {
    type InitialState = Address;
    type Accumulator = FeeManagerState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        fee_manager_address: &Address,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let contract = access
            .build_sync_contract(*fee_manager_address, block.number, FeeManagerImpl::new)
            .await;

        // `fee_manager_created` event
        let events = contract.fee_manager_created_filter().query().await.context(
            SyncContractError {
                err: "Error querying for fee manager created events",
            },
        )?;
        let created_event = events.first().unwrap();

        // `FeePerClaimReset` event
        let events = contract.fee_per_claim_reset_filter().query().await.context(
            SyncContractError {
                err: "Error querying for fee_per_claim reset events",
            },
        )?;
        // only need the last event to set to the current value
        let fee_per_claim = if let Some(e) = events.iter().last() {
            e.value
        } else {
            created_event.fee_per_claim
        };

        // `fee_redeemed` events
        let events = contract.fee_redeemed_filter().query().await.context(
            SyncContractError {
                err: "Error querying for fee redeemed events",
            },
        )?;
        let mut validator_redeemed: [Option<(Address, U256)>; 8] = [None; 8];
        let mut validator_redeemed_sums: HashMap<Address, U256> = HashMap::new();
        for ev in events.iter() {
            match validator_redeemed_sums.get(&ev.validator) {
                Some(amount) => validator_redeemed_sums[&ev.validator] = amount + ev.amount,
                None => validator_redeemed_sums[&ev.validator] = ev.amount,
            }
        }
        for (index, sum) in validator_redeemed_sums.iter().enumerate() {
            validator_redeemed[index] = Some((*sum.0, *sum.1));
        }

        // obtain fee manager balance
        let erc20_address = created_event.erc20;
        let erc20_contract = access
            .build_sync_contract(erc20_address, block.number, ERC20::new)
            .await;
        // `Transfer` events
        let erc20_events = erc20_contract.transfer_filter().query().await.context(
            SyncContractError {
                err: "Error querying for erc20 transfer events",
            },
        )?;
        // balance = income - expense
        let mut income: U256 = U256::zero();
        let mut expense: U256 = U256::zero();
        for ev in erc20_events.iter() {
            if ev.to == erc20_address {
                income = income + ev.value;
            } else if ev.from == erc20_address {
                expense = expense + ev.value;
            }
        }
        if expense > income {
            panic!("spend more than fee manager has!");
        }
        let fee_manager_balance = income - expense;

        Ok(FeeManagerState {
            validator_manager_address: created_event.validator_manager_cci,
            erc20_address,
            fee_per_claim,
            validator_redeemed,
            fee_manager_balance,
            fee_manager_address: *fee_manager_address,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let fee_manager_address = previous_state.fee_manager_address;

        // If not in bloom copy previous state
        if !(fold_utils::contains_address(&block.logs_bloom, &fee_manager_address)
            && fold_utils::contains_topic(
                &block.logs_bloom,
                &VoucherExecutedFilter::signature(),
            ))
        {
            return Ok(previous_state.clone());
        }

        let contract = access
            .build_fold_contract(fee_manager_address, block.hash, FeeManagerImpl::new)
            .await;

        let mut state = previous_state.clone();

        // `FeePerClaimReset` event
        let events = contract.fee_per_claim_reset_filter().query().await.context(
            FoldContractError {
                err: "Error querying for fee_per_claim reset events",
            },
        )?;
        // only need the last event to set to the current value
        if let Some(e) = events.iter().last() {
            state.fee_per_claim = e.value;
        }

        // `fee_redeemed` events
        let events = contract.fee_redeemed_filter().query().await.context(
            FoldContractError {
                err: "Error querying for fee redeemed events",
            },
        )?;
        // newly redeemed
        let mut validator_redeemed_sums: HashMap<Address, U256> = HashMap::new();
        for ev in events.iter() {
            let amount = validator_redeemed_sums.get(&ev.validator)
                .map(|v| *v)
                .unwrap_or(U256::zero());

            validator_redeemed_sums.insert(ev.validator, amount + ev.amount);
        };
        // update to the state.validator_redeemed array
        for (&validator_address, &newly_redeemed) in validator_redeemed_sums.iter() {
            let mut found = false;
            // find if address exist in the array
            for index in 0..8 {
                if let Some((address, pre_redeemed)) = state.validator_redeemed[index] {
                    if address == validator_address { // found validator, update #redeemed
                        state.validator_redeemed[index] = Some((address, pre_redeemed+newly_redeemed));
                        found = true;
                        break;
                    }
                }
            }
            // if not found
            if found == false {
                let mut create_new = false;

                for index in 0..8 {
                    if let None = state.validator_redeemed[index] {
                        state.validator_redeemed[index] = Some((validator_address, newly_redeemed));
                        create_new = true;
                        break;
                    };
                }

                if create_new == false {
                    panic!("no space for validator {}", validator_address);
                }
            }
        };

        // update fee manager balance
        let erc20_address = previous_state.erc20_address;
        let erc20_contract = access
            .build_fold_contract(erc20_address, block.hash, ERC20::new)
            .await;
        // `Transfer` events
        let erc20_events = erc20_contract.transfer_filter().query().await.context(
            FoldContractError {
                err: "Error querying for erc20 transfer events",
            },
        )?;
        // balance change = income - expense
        let mut income: U256 = U256::zero();
        let mut expense: U256 = U256::zero();
        for ev in erc20_events.iter() {
            if ev.to == erc20_address {
                income = income + ev.value;
            } else if ev.from == erc20_address {
                expense = expense + ev.value;
            }
        }
        if expense > income + state.fee_manager_balance {
            panic!("spend more than fee manager has!");
        }
        state.fee_manager_balance = state.fee_manager_balance + income - expense;

        Ok(state)
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}
