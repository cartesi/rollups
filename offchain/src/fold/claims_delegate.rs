use super::contracts::descartesv2_contract::*;
use super::types::Claims;

use dispatcher::state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils,
};
use dispatcher::types::Block;

use async_trait::async_trait;
use snafu::ResultExt;
use std::sync::Arc;

use ethers::providers::Middleware;
use ethers::types::{Address, U256};

/// Claims StateFold Delegate
#[derive(Clone, Debug)]
pub struct ClaimsState {
    claims: Option<Claims>,
    epoch: U256,
}

/// Claims StateFold Delegate
pub struct ClaimsFoldDelegate {
    descartesv2_address: Address,
}

impl ClaimsFoldDelegate {
    pub fn new(descartesv2_address: Address) -> Self {
        Self {
            descartesv2_address,
        }
    }
}

#[async_trait]
impl StateFoldDelegate for ClaimsFoldDelegate {
    type InitialState = U256;
    type Accumulator = ClaimsState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &U256,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let epoch = initial_state.clone();

        let middleware = access
            .build_sync_contract(Address::zero(), block.number, |_, m| m)
            .await;

        let contract = DescartesV2Impl::new(
            self.descartesv2_address,
            Arc::clone(&middleware),
        );

        // Get all claim events of epoch
        let claim_events = contract
            .claim_filter()
            .topic1(epoch.clone())
            .query_with_meta()
            .await
            .context(SyncContractError {
                err: "Error querying for descartes claims",
            })?;

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
                    c.insert_claim(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                    );
                    c
                }
            });
        }

        Ok(ClaimsState { claims, epoch })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let epoch = previous_state.epoch.clone();

        // If bloom doesn't contain the contract's address or the epoch number,
        // return the previous state.
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &self.descartesv2_address,
        ) && fold_utils::contains_topic(&block.logs_bloom, &epoch))
        {
            return Ok(previous_state.clone());
        }

        let contract = access
            .build_fold_contract(
                self.descartesv2_address,
                block.hash,
                DescartesV2Impl::new,
            )
            .await;

        // Get all claim events of epoch at this block hash
        let claim_events = contract
            .claim_filter()
            .topic1(epoch.clone())
            .query_with_meta()
            .await
            .context(FoldContractError {
                err: "Error querying for descartes claims",
            })?;

        let mut claims: Option<Claims> = previous_state.claims.clone();
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
                    c.insert_claim(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                    );
                    c
                }
            });
        }

        Ok(ClaimsState { claims, epoch })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}
