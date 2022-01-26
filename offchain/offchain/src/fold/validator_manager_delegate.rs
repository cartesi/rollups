use crate::contracts::erc20_contract::*;
use crate::contracts::validator_manager_contract::*;

use super::types::ValidatorManagerState;

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

use num_enum::IntoPrimitive;

/// Validator Manager Delegate
#[derive(Default)]
pub struct ValidatorManagerFoldDelegate {}

#[derive(IntoPrimitive)]
#[repr(u8)]
enum Result {
    NoConflict,
    Consensus,
    Conflict,
}

#[async_trait]
impl StateFoldDelegate for ValidatorManagerFoldDelegate {
    type InitialState = Address;
    type Accumulator = ValidatorManagerState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        validator_manager_address: &Address,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let contract = access
            .build_sync_contract(
                *validator_manager_address,
                block.number,
                ValidatorManagerClaimsCountedImpl::new,
            )
            .await;

        // initial settings
        let mut prev_epoch_claim = None;
        let mut num_claims = [None; 8];

        let mut current_claim = None;
        let mut agreeing_validator_set = Vec::new();

        let mut dispute_ended_events = contract.dispute_ended_filter().query().await.context(
            SyncContractError {
                err: "Error querying for dispute ended events",
            },
        )?;
        let mut dispute_ended_events_iter = dispute_ended_events.iter();

        let mut new_epoch_events = contract.new_epoch_filter().query().await.context(
            SyncContractError {
                err: "Error querying for new epoch events",
            },
        )?;
        let mut new_epoch_events_iter = new_epoch_events.iter();

        // `ClaimReceived` event
        let claim_received_events = contract.claim_received_filter().query().await.context(
            SyncContractError {
                err: "Error querying for claim received events",
            },
        )?;

        // increase #claims when consensus or timeout (wait for NewEpoch event)
        // assume consecutive epochs have different honest claims
        for ev in claim_received_events.iter() {
            match &ev.result{
                Result::NoConflict.into() => {
                    match current_claim {
                        None => {
                            current_claim = Some(&ev.claims); // type [u8; 32]
                            agreeing_validator_set.push(&ev.validators[0]); // type Address
                        },
                        Some(current_claim_value) => {
                            if current_claim_value == &ev.claims {
                                agreeing_validator_set.push(&ev.validators[0]);
                            }
                        }
                    }
                },
                Result::Consensus.into() => {

                },
                Result::Conflict.into() => {

                },
            }
        }

        Ok(ValidatorManagerState{
            
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}
