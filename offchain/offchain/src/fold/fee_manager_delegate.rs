use crate::contracts::fee_manager_facet::*;
use crate::contracts::rollups_init_facet::*;

use super::types::{FeeManagerState, NumRedeemed};

use offchain_core::types::Block;
use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils, DelegateAccess, StateFold,
};

use crate::fold::erc20_token_delegate::ERC20BalanceFoldDelegate;
use async_trait::async_trait;
use ethers::prelude::EthEvent;
use ethers::types::{Address, U256};
use im::HashMap;
use snafu::ResultExt;
use std::sync::Arc;

/// Fee Manager Delegate
pub struct FeeManagerFoldDelegate<DA: DelegateAccess + Send + Sync + 'static> {
    erc20_balance_fold: Arc<StateFold<ERC20BalanceFoldDelegate, DA>>,
}

impl<DA: DelegateAccess + Send + Sync + 'static> FeeManagerFoldDelegate<DA> {
    pub fn new(
        erc20_balance_fold: Arc<StateFold<ERC20BalanceFoldDelegate, DA>>,
    ) -> Self {
        Self { erc20_balance_fold }
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

        let rollups_init_facet = access
            .build_sync_contract(
                *dapp_contract_address,
                block.number,
                RollupsInitFacet::new,
            )
            .await;

        // `fee_manager_created` event
        let events = rollups_init_facet.
            fee_manager_initialized_filter()
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
        let events = fee_manager_facet.fee_redeemed_filter().query().await.context(
            SyncContractError {
                err: "Error querying for fee redeemed events",
            },
        )?;
        let mut validator_redeemed: [Option<NumRedeemed>; 8] = [None; 8];
        let mut validator_redeemed_sums: HashMap<Address, U256> =
            HashMap::new();
        for ev in events.iter() {
            match validator_redeemed_sums.get(&ev.validator) {
                Some(amount) => {
                    validator_redeemed_sums[&ev.validator] = amount + ev.amount
                }
                None => validator_redeemed_sums[&ev.validator] = ev.amount,
            }
        }
        for (index, sum) in validator_redeemed_sums.iter().enumerate() {
            validator_redeemed[index] = Some(NumRedeemed {
                validator_address: *sum.0,
                num_claims_redeemed: *sum.1,
            });
        }

        // obtain fee manager balance
        let erc20_address = created_event.erc_20_for_fee;
        let erc20_balance_state = self
            .erc20_balance_fold
            .get_state_for_block(
                &(erc20_address, *dapp_contract_address),
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("ERC20 balance state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        let fee_manager_balance = erc20_balance_state.balance;

        Ok(FeeManagerState {
            dapp_contract_address: *dapp_contract_address,
            erc20_address,
            fee_per_claim,
            validator_redeemed,
            fee_manager_balance,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let dapp_contract_address = previous_state.dapp_contract_address;

        // If not in bloom copy previous state
        if !(fold_utils::contains_address(
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
        ))) {
            return Ok(previous_state.clone());
        }

        let contract = access
            .build_fold_contract(
                dapp_contract_address,
                block.hash,
                FeeManagerFacet::new,
            )
            .await;

        let mut state = previous_state.clone();

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
        let mut validator_redeemed_sums: HashMap<Address, U256> =
            HashMap::new();
        for ev in events.iter() {
            let amount = validator_redeemed_sums
                .get(&ev.validator)
                .map(|v| *v)
                .unwrap_or(U256::zero());

            validator_redeemed_sums.insert(ev.validator, amount + ev.amount);
        }
        // update to the state.validator_redeemed array
        for (&validator_address, &newly_redeemed) in
            validator_redeemed_sums.iter()
        {
            let mut found = false;
            // find if address exist in the array
            for index in 0..8 {
                if let Some(num_redeemed_struct) =
                    &state.validator_redeemed[index]
                {
                    let address = num_redeemed_struct.validator_address;
                    let pre_redeemed = num_redeemed_struct.num_claims_redeemed;
                    if address == validator_address {
                        // found validator, update #redeemed
                        state.validator_redeemed[index] = Some(NumRedeemed {
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

                for index in 0..8 {
                    if let None = state.validator_redeemed[index] {
                        state.validator_redeemed[index] = Some(NumRedeemed {
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

        // update fee manager balance
        let erc20_balance_state = self
            .erc20_balance_fold
            .get_state_for_block(
                &(state.erc20_address, state.dapp_contract_address),
                Some(block.hash),
            )
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("ERC20 balance state fold error: {:?}", e),
                }
                .build()
            })?
            .state;

        state.fee_manager_balance = erc20_balance_state.balance;

        Ok(state)
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}
