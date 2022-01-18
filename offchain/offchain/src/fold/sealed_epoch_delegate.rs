use offchain_core::ethers;

use crate::contracts::rollups_contract::*;

use super::input_delegate::InputFoldDelegate;
use super::types::{
    AccumulatingEpoch, Claims, EpochInputState, EpochWithClaims,
};

use offchain_core::types::Block;
use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils, DelegateAccess, StateFold,
};

use async_trait::async_trait;
use snafu::ResultExt;
use std::sync::Arc;

use ethers::prelude::EthEvent;
use ethers::providers::Middleware;
use ethers::types::{Address, H256, U256};

#[derive(Clone, Debug)]
pub enum SealedEpochState {
    SealedEpochNoClaims { sealed_epoch: AccumulatingEpoch },
    SealedEpochWithClaims { claimed_epoch: EpochWithClaims },
}

impl SealedEpochState {
    pub fn epoch_number(&self) -> U256 {
        match self {
            SealedEpochState::SealedEpochNoClaims { sealed_epoch } => {
                sealed_epoch.epoch_number
            }
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => {
                claimed_epoch.epoch_number
            }
        }
    }

    pub fn dapp_contract_address(&self) -> Address {
        match self {
            SealedEpochState::SealedEpochNoClaims { sealed_epoch } => {
                sealed_epoch.dapp_contract_address
            }
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => {
                claimed_epoch.dapp_contract_address
            }
        }
    }

    pub fn claims(&self) -> Option<Claims> {
        match self {
            SealedEpochState::SealedEpochNoClaims { .. } => None,
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => {
                Some(claimed_epoch.claims.clone())
            }
        }
    }

    pub fn inputs(&self) -> EpochInputState {
        match self {
            SealedEpochState::SealedEpochNoClaims { sealed_epoch } => {
                sealed_epoch.inputs.clone()
            }
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => {
                claimed_epoch.inputs.clone()
            }
        }
    }
}
/// Sealed epoch StateFold Delegate
pub struct SealedEpochFoldDelegate<DA: DelegateAccess> {
    input_fold: Arc<StateFold<InputFoldDelegate, DA>>,
}

impl<DA: DelegateAccess> SealedEpochFoldDelegate<DA> {
    pub fn new(
        input_fold: Arc<StateFold<InputFoldDelegate, DA>>,
    ) -> Self {
        Self {
            input_fold,
        }
    }
}

#[async_trait]
impl<DA: DelegateAccess + Send + Sync + 'static> StateFoldDelegate
    for SealedEpochFoldDelegate<DA>
{
    type InitialState = (Address, U256);
    type Accumulator = SealedEpochState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &(Address, U256),
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let (dapp_contract_address, epoch_number) =
            initial_state.clone();

        let middleware = access
            .build_sync_contract(Address::zero(), block.number, |_, m| m)
            .await;

        let contract = RollupsImpl::new(
            dapp_contract_address,
            Arc::clone(&middleware),
        );

        // Inputs of epoch
        let inputs = self
            .get_inputs_sync(dapp_contract_address, epoch_number, block.hash)
            .await?;

        // Get all claim events of epoch
        let claim_events = contract
            .claim_filter()
            .topic1(epoch_number.clone())
            .query_with_meta()
            .await
            .context(SyncContractError {
                err: "Error querying for rollups claims",
            })?;

        // If there are no claim, state is SealedEpochNoClaims
        if claim_events.is_empty() {
            return Ok(SealedEpochState::SealedEpochNoClaims {
                sealed_epoch: AccumulatingEpoch {
                    epoch_number,
                    inputs,
                    dapp_contract_address,
                },
            });
        }

        let mut claims: Option<Claims> = None;
        for (claim_event, meta) in claim_events {
            claims = Some(match claims {
                None => {
                    // If first claim, get timestamp
                    let timestamp = middleware
                        .get_block(meta.block_hash)
                        .await
                        .context(SyncAccessError {})?
                        .ok_or(snafu::NoneError)
                        .context(SyncDelegateError {
                            err: "Block not found",
                        })?
                        .timestamp;

                    Claims::new(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                        timestamp,
                    )
                }

                Some(mut c) => {
                    c.insert_claim(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                    );
                    c
                }
            });
        }

        Ok(SealedEpochState::SealedEpochWithClaims {
            claimed_epoch: EpochWithClaims {
                epoch_number,
                inputs,
                claims: claims.unwrap(),
                dapp_contract_address,
            },
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        // Check if there was (possibly) some log emited on this block.
        // As finalized epochs' inputs will not change, we can return early
        // without querying the input StateFold.
        let dapp_contract_address =
            previous_state.dapp_contract_address().clone();
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &dapp_contract_address,
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &previous_state.epoch_number(),
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &ClaimFilter::signature(),
        )) {
            return Ok(previous_state.clone());
        }

        let epoch_number = previous_state.epoch_number().clone();
        let contract = access
            .build_fold_contract(
                dapp_contract_address,
                block.hash,
                RollupsImpl::new,
            )
            .await;

        // Get claim events of epoch at this block hash
        let claim_events = contract
            .claim_filter()
            .topic1(epoch_number.clone())
            .query_with_meta()
            .await
            .context(FoldContractError {
                err: "Error querying for rollups claims",
            })?;

        // if there are no new claims, return epoch's old claims (might be empty)
        if claim_events.is_empty() {
            return Ok(previous_state.clone());
        }

        let mut claims: Option<Claims> = previous_state.claims();
        for (claim_event, _) in claim_events {
            claims = Some(match claims {
                None => {
                    // If this is the first claim in epoch, get block timestamp
                    // and create new claim
                    let timestamp = block.timestamp;
                    Claims::new(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                        timestamp,
                    )
                }

                Some(mut c) => {
                    // else there are other claims, timestamp is uninmportant
                    c.insert_claim(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                    );
                    c
                }
            });
        }
        // don't need to re-update inputs because epoch is sealed
        Ok(SealedEpochState::SealedEpochWithClaims {
            claimed_epoch: EpochWithClaims {
                epoch_number,
                inputs: previous_state.inputs(),
                claims: claims.unwrap(),
                dapp_contract_address,
            },
        })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}

impl<DA: DelegateAccess + Send + Sync + 'static> SealedEpochFoldDelegate<DA> {
    async fn get_inputs_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        dapp_contract_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> SyncResult<EpochInputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(
                &(dapp_contract_address, epoch),
                Some(block_hash),
            )
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Input state fold error: {:?}", e),
                }
                .build()
            })?
            .state)
    }
}
