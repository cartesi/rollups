use super::contracts::descartesv2_contract::*;

use super::input_delegate::InputFoldDelegate;
use super::types::{AccumulatingEpoch, Claims, EpochInputState, EpochWithClaims};

use dispatcher::state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils, DelegateAccess, StateFold,
};
use dispatcher::types::Block;

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
            SealedEpochState::SealedEpochNoClaims { sealed_epoch } => sealed_epoch.epoch_number,
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => claimed_epoch.epoch_number,
        }
    }

    pub fn claims(&self) -> Option<Claims> {
        match self {
            SealedEpochState::SealedEpochNoClaims { .. } => None,
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => Some(claimed_epoch.claims),
        }
    }

    pub fn inputs(&self) -> EpochInputState {
        match self {
            SealedEpochState::SealedEpochNoClaims { sealed_epoch } => sealed_epoch.inputs,
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => claimed_epoch.inputs,
        }
    }
}
/// Sealed epoch StateFold Delegate
pub struct SealedEpochFoldDelegate<DA: DelegateAccess> {
    descartesv2_address: Address,
    input_fold: Arc<StateFold<InputFoldDelegate, DA>>,
}

impl<DA: DelegateAccess> SealedEpochFoldDelegate<DA> {
    pub fn new(
        descartesv2_address: Address,
        input_fold: Arc<StateFold<InputFoldDelegate, DA>>,
    ) -> Self {
        Self {
            descartesv2_address,
            input_fold,
        }
    }
}

#[async_trait]
impl<DA: DelegateAccess + Send + Sync + 'static> StateFoldDelegate for SealedEpochFoldDelegate<DA> {
    type InitialState = U256;
    type Accumulator = SealedEpochState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &U256,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let epoch_number = initial_state.clone();

        let middleware = access
            .build_sync_contract(Address::zero(), block.number, |_, m| m)
            .await;

        let contract = DescartesV2Impl::new(self.descartesv2_address, Arc::clone(&middleware));

        // Inputs of epoch
        let inputs = self.get_inputs_sync(epoch_number, block.hash).await?;

        // Get all claim events of epoch
        let claim_events = contract
            .claim_filter()
            .topic1(epoch_number.clone())
            .query_with_meta()
            .await
            .context(SyncContractError {
                err: "Error querying for descartes claims",
            })?;

        if claim_events.is_empty() {
            return Ok(SealedEpochState::SealedEpochNoClaims {
                sealed_epoch: AccumulatingEpoch {
                    epoch_number,
                    inputs,
                },
            });
        }

        let mut claims: Option<Claims> = None;
        for (claim_event, meta) in claim_events {
            claims = Some(match claims {
                None => {
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

                Some(c) => {
                    c.insert_claim(claim_event.epoch_hash.into(), claim_event.claimer);
                    c
                }
            });
        }

        Ok(SealedEpochState::SealedEpochWithClaims {
            claimed_epoch: EpochWithClaims {
                epoch_number,
                inputs,
                claims: claims.unwrap(),
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
        if !(fold_utils::contains_address(&block.logs_bloom, &self.descartesv2_address)
            && fold_utils::contains_topic(&block.logs_bloom, &previous_state.epoch_number())
            && fold_utils::contains_topic(&block.logs_bloom, &ClaimFilter::signature()))
        {
            return Ok(previous_state.clone());
        }

        let epoch_number = previous_state.epoch_number().clone();
        let contract = access
            .build_fold_contract(self.descartesv2_address, block.hash, DescartesV2Impl::new)
            .await;

        // Get claim events of epoch at this block hash
        let claim_events = contract
            .claim_filter()
            .topic1(epoch_number.clone())
            .query_with_meta()
            .await
            .context(FoldContractError {
                err: "Error querying for descartes claims",
            })?;

        if claim_events.is_empty() {
            return Ok(previous_state.clone());
        }

        let mut claims: Option<Claims> = previous_state.claims();
        for (claim_event, meta) in claim_events {
            claims = Some(match claims {
                None => {
                    let timestamp = block.timestamp;
                    Claims::new(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                        timestamp,
                    )
                }

                Some(c) => {
                    c.insert_claim(claim_event.epoch_hash.into(), claim_event.claimer);
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
            },
        })
    }

    fn convert(&self, accumulator: &BlockState<Self::Accumulator>) -> Self::State {
        accumulator.clone()
    }
}

impl<DA: DelegateAccess + Send + Sync + 'static> SealedEpochFoldDelegate<DA> {
    async fn get_inputs_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        epoch: U256,
        block_hash: H256,
    ) -> SyncResult<EpochInputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(&epoch, block_hash)
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Input state fold error: {:?}", e),
                }
                .build()
            })?
            .state)
    }

    async fn get_inputs_fold<A: FoldAccess + Send + Sync + 'static>(
        &self,
        epoch: U256,
        block_hash: H256,
    ) -> FoldResult<EpochInputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(&epoch, block_hash)
            .await
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Input state fold error: {:?}", e),
                }
                .build()
            })?
            .state)
    }
}
