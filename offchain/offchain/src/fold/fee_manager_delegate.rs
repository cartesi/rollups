use contracts::bank_contract::*;
use contracts::diamond_init::*;
use contracts::fee_manager_facet::*;
use contracts::rollups_facet::*;
use contracts::validator_manager_facet::*;

use super::types::{
    FeeManagerState, NumClaims, NumRedeemed, MAX_NUM_VALIDATORS,
};

use offchain_core::types::Block;
use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils, DelegateAccess, StateFold,
};

use crate::fold::bank_delegate::BankFoldDelegate;
use crate::fold::validator_manager_delegate::ValidatorManagerFoldDelegate;
use async_trait::async_trait;
use ethers::prelude::EthEvent;
use ethers::types::{Address, U256};
use snafu::ResultExt;
use std::convert::TryFrom;
use std::sync::Arc;

/// Fee Manager Delegate
pub struct FeeManagerFoldDelegate<DA: DelegateAccess + Send + Sync + 'static> {
    bank_fold: Arc<StateFold<BankFoldDelegate, DA>>,
    validator_manager_fold: Arc<StateFold<ValidatorManagerFoldDelegate, DA>>,
}

impl<DA: DelegateAccess + Send + Sync + 'static> FeeManagerFoldDelegate<DA> {
    pub fn new(
        bank_fold: Arc<StateFold<BankFoldDelegate, DA>>,
        validator_manager_fold: Arc<
            StateFold<ValidatorManagerFoldDelegate, DA>,
        >,
    ) -> Self {
        Self {
            bank_fold,
            validator_manager_fold,
        }
    }
}

#[async_trait]
impl<DA: DelegateAccess + Send + Sync + 'static> StateFoldDelegate
    for FeeManagerFoldDelegate<DA>
{
    type InitialState = Address;
    type Accumulator = FeeManagerState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &Self::InitialState,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let dapp_contract_address = initial_state;

        let validator_manager_state = self
            .validator_manager_fold
            .get_state_for_block(&dapp_contract_address, Some(block.hash))
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Validator Manager state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let diamond_init = access
            .build_sync_contract(
                *dapp_contract_address,
                block.number,
                DiamondInit::new,
            )
            .await;

        // `FeeManagerInitialized` event
        let events = diamond_init
            .fee_manager_initialized_filter()
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for fee manager initialized events",
            })?;
        let created_event = events.first().unwrap();

        let fee_manager_facet = access
            .build_sync_contract(
                *dapp_contract_address,
                block.number,
                FeeManagerFacet::new,
            )
            .await;

        // `FeePerClaimReset` event
        let events = fee_manager_facet
            .fee_per_claim_reset_filter()
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for fee_per_claim reset events",
            })?;
        // only need the last event to set to the current value
        let fee_per_claim = if let Some(e) = events.iter().last() {
            e.value
        } else {
            created_event.fee_per_claim
        };

        // `fee_redeemed` events
        let events = fee_manager_facet
            .fee_redeemed_filter()
            .query()
            .await
            .context(SyncContractError {
                err: "Error querying for fee redeemed events",
            })?;

        let mut num_redeemed: [Option<NumRedeemed>; MAX_NUM_VALIDATORS] =
            [None; MAX_NUM_VALIDATORS];

        for ev in events.iter() {
            if validator_manager_state
                .validators_removed
                .contains(&ev.validator)
            {
                // skip validator if it's removed in Validator Manager state
                continue;
            }
            find_and_increase(&mut num_redeemed, &ev.validator, &ev.claims);
        }

        // obtain fee manager bank balance
        let bank_address = created_event.fee_manager_bank;
        let bank_state = self
            .bank_fold
            .get_state_for_block(
                &(bank_address, *dapp_contract_address),
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Bank state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let bank_balance = bank_state.balance;

        // uncommitted balance
        let uncommitted_balance = calculate_uncommitted_balance(
            &num_redeemed,
            &validator_manager_state.num_claims,
            &fee_per_claim,
            &bank_balance,
        );

        Ok(FeeManagerState {
            dapp_contract_address: *dapp_contract_address,
            bank_address,
            fee_per_claim,
            num_redeemed,
            bank_balance,
            uncommitted_balance,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let dapp_contract_address = previous_state.dapp_contract_address;
        let bank_address = previous_state.bank_address;

        // If not in bloom copy previous state
        if !((fold_utils::contains_address(
            &block.logs_bloom,
            &dapp_contract_address,
        ) && (fold_utils::contains_topic(
            &block.logs_bloom,
            &FeeManagerInitializedFilter::signature(),
        ) || fold_utils::contains_topic(
            &block.logs_bloom,
            &FeePerClaimResetFilter::signature(),
        ) || fold_utils::contains_topic(
            &block.logs_bloom,
            &FeeRedeemedFilter::signature(),
        ) || fold_utils::contains_topic(
            // the following evernts are to update validator manager delegate
            &block.logs_bloom,
            &DisputeEndedFilter::signature(),
        ) || fold_utils::contains_topic(
            &block.logs_bloom,
            &NewEpochFilter::signature(),
        ) || fold_utils::contains_topic(
            &block.logs_bloom,
            &ClaimFilter::signature(),
        ))) || (fold_utils::contains_address(
            &block.logs_bloom,
            &bank_address,
        ) && (fold_utils::contains_topic(
            &block.logs_bloom,
            &TransferFilter::signature(),
        ) || fold_utils::contains_topic(
            &block.logs_bloom,
            &DepositFilter::signature(),
        )))) {
            return Ok(previous_state.clone());
        }

        let mut state = previous_state.clone();

        let contract = access
            .build_fold_contract(
                dapp_contract_address,
                block.hash,
                FeeManagerFacet::new,
            )
            .await;

        let validator_manager_state = self
            .validator_manager_fold
            .get_state_for_block(&dapp_contract_address, Some(block.hash))
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Validator manager fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        // `FeePerClaimReset` event
        let events = contract
            .fee_per_claim_reset_filter()
            .query()
            .await
            .context(FoldContractError {
                err: "Error querying for fee_per_claim reset events",
            })?;
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
        for ev in events.iter() {
            if validator_manager_state
                .validators_removed
                .contains(&ev.validator)
            {
                // skip validator if it's removed in Validator Manager state
                continue;
            }
            find_and_increase(
                &mut state.num_redeemed,
                &ev.validator,
                &ev.claims,
            );
        }

        // remove validator's existing num_redeemed if it's newly removed from Validator Manager state
        for index in 0..MAX_NUM_VALIDATORS {
            if let Some(num_redeemed_struct) = &state.num_redeemed[index] {
                let address = num_redeemed_struct.validator_address;
                if validator_manager_state
                    .validators_removed
                    .contains(&address)
                {
                    state.num_redeemed[index] = None;
                }
            }
        }

        // update fee manager bank balance
        let bank_state = self
            .bank_fold
            .get_state_for_block(
                &(state.bank_address, state.dapp_contract_address),
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Bank state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        state.bank_balance = bank_state.balance;

        // uncommitted balance
        state.uncommitted_balance = calculate_uncommitted_balance(
            &state.num_redeemed,
            &validator_manager_state.num_claims,
            &state.fee_per_claim,
            &state.bank_balance,
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

fn calculate_uncommitted_balance(
    num_redeemed: &[Option<NumRedeemed>; MAX_NUM_VALIDATORS],
    num_claims: &[Option<NumClaims>; MAX_NUM_VALIDATORS],
    fee_per_claim: &U256,
    bank_balance: &U256,
) -> i128 {
    // calculate total number of claims made by all validators
    let mut total_claims = U256::zero();
    for i in 0..MAX_NUM_VALIDATORS {
        if let Some(num_claims_struct) = num_claims[i] {
            total_claims = total_claims + num_claims_struct.num_claims_made;
        }
    }

    // calculate total number of claims redeemed by all validators
    let mut total_redeems = U256::zero();
    for i in 0..MAX_NUM_VALIDATORS {
        if let Some(num_redeemed_struct) = num_redeemed[i] {
            total_redeems =
                total_redeems + num_redeemed_struct.num_claims_redeemed;
        }
    }

    // calculate uncommitted balance for fee manager
    // uncommitted_balance = current_balance - to_be_redeemed_fees
    // un-finalized claims are not considered
    let to_be_redeemed_fees = (total_claims - total_redeems) * fee_per_claim;
    let uncommitted_balance = i128::try_from(bank_balance.as_u128()).unwrap()
        - i128::try_from(to_be_redeemed_fees.as_u128()).unwrap();
    uncommitted_balance
}

fn find_and_increase(
    num_redeemed: &mut [Option<NumRedeemed>; MAX_NUM_VALIDATORS],
    addr: &Address,
    num: &U256,
) {
    // find if address exist in the array
    for i in 0..MAX_NUM_VALIDATORS {
        if let Some(num_redeemed_struct) = num_redeemed[i] {
            let address = num_redeemed_struct.validator_address;
            let number = num_redeemed_struct.num_claims_redeemed;
            if address == *addr {
                // found validator, update #redeemed
                num_redeemed[i] = Some(NumRedeemed {
                    validator_address: address,
                    num_claims_redeemed: number + num,
                });
                return;
            }
        }
    }

    // if not found
    let mut create_new = false;

    for i in 0..MAX_NUM_VALIDATORS {
        if let None = num_redeemed[i] {
            num_redeemed[i] = Some(NumRedeemed {
                validator_address: *addr,
                num_claims_redeemed: *num,
            });
            create_new = true;
            break;
        };
    }

    if create_new == false {
        panic!("no space for validator {}", addr);
    }
}
