use crate::validator_manager::MAX_NUM_VALIDATORS;
use crate::{
    bank::BankState,
    validator_manager::{NumClaims, ValidatorManagerState},
    FoldableError,
};
use anyhow::Context;
use async_trait::async_trait;
use contracts::{
    bank_contract::*, diamond_init::*, fee_manager_facet::*, rollups_facet::*,
    validator_manager_facet::*,
};
use ethers::{
    prelude::{EthEvent, Middleware},
    types::{Address, U256},
};
use serde::{Deserialize, Serialize};
use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::{convert::TryFrom, sync::Arc};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NumRedeemed {
    pub validator_address: Address,
    pub num_claims_redeemed: U256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeManagerState {
    pub dapp_contract_address: Address,
    pub bank_address: Address,
    pub fee_per_claim: U256, // only the current value
    // Tuple containing (validator, #claims_redeemed_so_far)
    pub num_redeemed: [Option<NumRedeemed>; MAX_NUM_VALIDATORS],
    pub bank_balance: U256,
    // Uncommitted balance equals the balance of bank contract minus
    // the amount of to-be-redeemed fees
    // un-finalized claims are not considered
    pub uncommitted_balance: i128,
}

#[derive(Debug)]
pub struct FeeIncentiveStrategy {
    pub num_buffer_epochs: usize,
    pub num_claims_trigger_redeem: usize,
    pub minimum_required_fee: U256,
}

impl Default for FeeIncentiveStrategy {
    fn default() -> Self {
        FeeIncentiveStrategy {
            // ideally fee manager should have enough uncommitted balance for at least 4 epochs
            num_buffer_epochs: 4,
            // when the number of redeemable claims reaches this value, call `redeem`
            num_claims_trigger_redeem: 4,
            // zero means an altruistic validator
            minimum_required_fee: U256::zero(),
        }
    }
}

impl FeeManagerState {
    pub fn should_redeem(
        &self,
        validator_manager_state: &ValidatorManagerState,
        validator_address: Address,
        strategy: &FeeIncentiveStrategy,
    ) -> bool {
        let num_claims_trigger_redeem =
            U256::from(strategy.num_claims_trigger_redeem);

        let validator_claims =
            validator_manager_state.num_claims_for_validator(validator_address);
        let validator_redeemed =
            self.num_redeemed_for_validator(validator_address);
        assert!(
            validator_claims >= validator_redeemed,
            "validator_claims should be no less than validator_redeemed"
        );
        let num_redeemable_claims = validator_claims - validator_redeemed;

        num_redeemable_claims >= num_claims_trigger_redeem
    }

    pub fn num_redeemed_for_validator(
        &self,
        validator_address: Address,
    ) -> U256 {
        // number of redeemed claims for the validator
        let num_redeemed = self.num_redeemed;
        let mut validator_redeemed = U256::zero();
        for i in 0..MAX_NUM_VALIDATORS {
            // find validator address in `num_redeemed`
            if let Some(num_redeemed_struct) = &num_redeemed[i] {
                if num_redeemed_struct.validator_address == validator_address {
                    validator_redeemed =
                        num_redeemed_struct.num_claims_redeemed;
                    break;
                }
            }
        }
        validator_redeemed
    }

    pub fn sufficient_uncommitted_balance(
        &self,
        validator_manager_state: &ValidatorManagerState,
        strategy: &FeeIncentiveStrategy,
    ) -> bool {
        if strategy.minimum_required_fee == U256::zero() {
            return true;
        }

        if self.fee_per_claim < strategy.minimum_required_fee {
            return false;
        }

        let validators_removed =
            validator_manager_state.validators_removed.len();

        assert!(
            MAX_NUM_VALIDATORS >= validators_removed,
            "current_num_validators out of range"
        );

        let current_num_validators =
            (MAX_NUM_VALIDATORS - validators_removed) as i128;

        let fee_per_claim =
            i128::try_from(self.fee_per_claim.as_u128()).unwrap();

        let balance_buffer = fee_per_claim
            * current_num_validators
            * (strategy.num_buffer_epochs as i128);

        self.uncommitted_balance >= balance_buffer
    }
}

#[async_trait]
impl Foldable for FeeManagerState {
    type InitialState = Address;
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let dapp_contract_address = initial_state;

        let diamond_init =
            DiamondInit::new(*dapp_contract_address, Arc::clone(&access));

        // obtain #claims validators made from Validator Manager
        let validator_manager_state =
            ValidatorManagerState::get_state_for_block(
                &dapp_contract_address,
                block,
                env,
            )
            .await?
            .state;

        // `FeeManagerInitialized` event
        let events = diamond_init
            .fee_manager_initialized_filter()
            .query()
            .await
            .context("Error querying for fee manager initialized events")?;
        let created_event = events.first().unwrap();

        let fee_manager_facet =
            FeeManagerFacet::new(*dapp_contract_address, access);

        // `FeePerClaimReset` event
        let events = fee_manager_facet
            .fee_per_claim_reset_filter()
            .query()
            .await
            .context("Error querying for fee_per_claim reset events")?;
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
            .context("Error querying for fee redeemed events")?;
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
        let bank_state = BankState::get_state_for_block(
            &(bank_address, *dapp_contract_address),
            block,
            env,
        )
        .await?
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

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
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

        let contract = FeeManagerFacet::new(dapp_contract_address, access);

        let validator_manager_state =
            ValidatorManagerState::get_state_for_block(
                &dapp_contract_address,
                block,
                env,
            )
            .await?
            .state;

        // `FeePerClaimReset` event
        let events = contract
            .fee_per_claim_reset_filter()
            .query()
            .await
            .context("Error querying for fee_per_claim reset events")?;
        // only need the last event to set to the current value
        if let Some(e) = events.iter().last() {
            state.fee_per_claim = e.value;
        }

        // `fee_redeemed` events
        let events = contract
            .fee_redeemed_filter()
            .query()
            .await
            .context("Error querying for fee redeemed events")?;
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
        let bank_state = BankState::get_state_for_block(
            &(state.bank_address, state.dapp_contract_address),
            block,
            env,
        )
        .await?
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
}

fn calculate_uncommitted_balance(
    validator_redeemed: &[Option<NumRedeemed>; MAX_NUM_VALIDATORS],
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
        if let Some(num_redeemed_struct) = validator_redeemed[i] {
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
