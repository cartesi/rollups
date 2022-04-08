use crate::contracts::bank_contract::*;
use crate::contracts::diamond_init::*;
use crate::contracts::fee_manager_facet::*;
use crate::contracts::rollups_facet::*;
use crate::contracts::validator_manager_facet::*;

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
use im::HashMap;
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

        let mut num_redeemed_sums: HashMap<Address, U256> = HashMap::new();

        for ev in events.iter() {
            match num_redeemed_sums.get(&ev.validator) {
                Some(amount) => {
                    num_redeemed_sums[&ev.validator] = amount + ev.claims
                }
                None => num_redeemed_sums[&ev.validator] = ev.claims,
            }
        }
        for (index, sum) in num_redeemed_sums.iter().enumerate() {
            num_redeemed[index] = Some(NumRedeemed {
                validator_address: *sum.0,
                num_claims_redeemed: *sum.1,
            });
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

        // obtain #claims validators made from Validator Manager
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
        let mut num_redeemed_sums: HashMap<Address, U256> = HashMap::new();
        for ev in events.iter() {
            let amount = num_redeemed_sums
                .get(&ev.validator)
                .map(|v| *v)
                .unwrap_or(U256::zero());

            num_redeemed_sums.insert(ev.validator, amount + ev.claims);
        }
        // update to the state.num_redeemed array
        for (&validator_address, &newly_redeemed) in num_redeemed_sums.iter() {
            let mut found = false;
            // find if address exist in the array
            for index in 0..MAX_NUM_VALIDATORS {
                if let Some(num_redeemed_struct) = &state.num_redeemed[index] {
                    let address = num_redeemed_struct.validator_address;
                    let pre_redeemed = num_redeemed_struct.num_claims_redeemed;
                    if address == validator_address {
                        // found validator, update #redeemed
                        state.num_redeemed[index] = Some(NumRedeemed {
                            validator_address: address,
                            num_claims_redeemed: pre_redeemed + newly_redeemed,
                        });
                        found = true;
                        break;
                    }
                }
            }
            // if not found
            if found == false {
                let mut create_new = false;

                for index in 0..MAX_NUM_VALIDATORS {
                    if let None = state.num_redeemed[index] {
                        state.num_redeemed[index] = Some(NumRedeemed {
                            validator_address: validator_address,
                            num_claims_redeemed: newly_redeemed,
                        });
                        create_new = true;
                        break;
                    };
                }

                if create_new == false {
                    panic!("no space for validator {}", validator_address);
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
            total_claims = total_claims + num_claims_struct.num_claims_mades;
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
