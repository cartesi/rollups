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

/// Claims accumulator for StateFold Delegate
#[derive(Clone, Debug)]
pub struct ClaimsAccumulator {
    claims: Option<Claims>,
    epoch_number: U256,
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
    type Accumulator = ClaimsAccumulator;
    type State = BlockState<Option<Claims>>;

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

        let contract = DescartesV2Impl::new(
            self.descartesv2_address,
            Arc::clone(&middleware),
        );

        // Get all claim events of epoch
        let claim_events = contract
            .claim_filter()
            .topic1(epoch_number.clone())
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

        Ok(ClaimsAccumulator {
            claims,
            epoch_number,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let epoch_number = previous_state.epoch_number.clone();

        // Check if there was (possibly) some log emited on this block.
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &self.descartesv2_address,
        ) && fold_utils::contains_topic(&block.logs_bloom, &epoch_number))
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
            .topic1(epoch_number.clone())
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

        Ok(ClaimsAccumulator {
            claims,
            epoch_number,
        })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        BlockState {
            block: accumulator.block,
            state: accumulator.state.claims,
        }
    }
}
