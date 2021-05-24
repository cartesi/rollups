use super::contracts::descartesv2_contract::*;

use super::claims_delegate::ClaimsFoldDelegate;
use super::input_delegate::InputFoldDelegate;
use super::types::{
    AccumulatingEpoch, Claims, FinalizedEpoch, InputState, SealedEpoch,
};

use dispatcher::state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils, DelegateAccess, StateFold,
};
use dispatcher::types::Block;

use async_trait::async_trait;
use im::Vector;
use snafu::ResultExt;
use std::convert::TryFrom;
use std::sync::Arc;

use ethers::types::{Address, H256, U256};

///
#[derive(Clone, Debug)]
pub enum ContractPhase {
    InputAccumulation {
        current_epoch: AccumulatingEpoch,
    },

    AwaitingConsensus {
        sealed_epoch: SealedEpoch,
        current_epoch: AccumulatingEpoch,
        round_start: U256,
    },

    AwaitingDispute {
        sealed_epoch: SealedEpoch,
        current_epoch: AccumulatingEpoch,
    },
}

#[derive(Clone, Debug)]
pub struct EpochState {
    pub current_phase: ContractPhase,
    pub initial_epoch: U256,
    pub finalized_epochs: Vector<FinalizedEpoch>, // EpochNumber -> Epoch
}

/// Epoch StateActor Delegate, which implements `sync` and `fold`.
pub struct EpochFoldDelegate<DA: DelegateAccess> {
    descartesv2_address: Address,
    input_fold: StateFold<InputFoldDelegate, DA>,
    claims_fold: StateFold<ClaimsFoldDelegate, DA>,
}

impl<DA: DelegateAccess> EpochFoldDelegate<DA> {
    pub fn new(
        descartesv2_address: Address,
        input_fold: StateFold<InputFoldDelegate, DA>,
        claims_fold: StateFold<ClaimsFoldDelegate, DA>,
    ) -> Self {
        Self {
            descartesv2_address,
            input_fold,
            claims_fold,
        }
    }
}

#[async_trait]
impl<DA: DelegateAccess + Send + Sync + 'static> StateFoldDelegate
    for EpochFoldDelegate<DA>
{
    type InitialState = U256;
    type Accumulator = EpochState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &U256,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let middleware = access
            .build_sync_contract(Address::zero(), block.number, |_, m| m)
            .await;

        let contract = DescartesV2Impl::new(
            self.descartesv2_address,
            Arc::clone(&middleware),
        );

        let epoch_finalized_events = contract
            .finalize_epoch_filter()
            .query_with_meta()
            .await
            .context(SyncContractError {
                err: "Error querying for descartes finalized epochs",
            })?;

        let finalized_epochs = {
            let mut finalized_epochs = Vector::new();
            for (epoch_number, (ev, _)) in
                epoch_finalized_events.iter().enumerate()
            {
                assert_eq!(epoch_number, finalized_epochs.len());

                let inputs = self
                    .get_inputs_sync(epoch_number.into(), block.hash)
                    .await?;

                finalized_epochs.push_back(FinalizedEpoch {
                    epoch_number: epoch_number.into(),
                    hash: ev.epoch_hash.into(),
                    inputs,
                });
            }

            finalized_epochs
        };

        let first_non_finalized_epoch_inputs = {
            // Index of first non finalized epoch
            let number = finalized_epochs.len().into();

            // Inputs of first non finalized epoch
            let inputs = self.get_inputs_sync(number, block.hash).await?;

            AccumulatingEpoch { inputs, number }
        };

        let phase_change_events = contract
            .phase_change_filter()
            .query_with_meta()
            .await
            .context(SyncContractError {
                err: "Error querying for descartes phase change",
            })?;

        let current_phase = match phase_change_events.last() {
            // InputAccumulation
            Some((PhaseChangeFilter { new_phase: 0 }, _)) => {
                ContractPhase::InputAccumulation {
                    current_epoch: first_non_finalized_epoch_inputs,
                }
            }

            // AwaitingConsensus | AwaitingDispute
            Some((PhaseChangeFilter { new_phase: 1 }, m)) => {
                // Current epoch
                let epoch_number = (finalized_epochs.len() + 1).into();

                // Inputs of current epoch
                let inputs =
                    self.get_inputs_sync(epoch_number, block.hash).await?;

                // Claims of current epoch
                let claims =
                    self.get_claims_sync(epoch_number, block.hash).await?;

                let sealed_epoch = SealedEpoch {
                    claims,
                    epoch_number,
                    inputs,
                };

                // Timestamp of when we entered this phase.
                let round_start = middleware
                    .get_block(m.block_hash)
                    .await
                    .context(SyncAccessError {})?
                    .ok_or(snafu::NoneError)
                    .context(SyncDelegateError {
                        err: "Block not found",
                    })?
                    .timestamp;

                PhaseState::AwaitingConsensus {
                    sealed_epoch,
                    current_epoch: first_non_finalized_epoch_inputs,
                    round_start,
                }
            }

            Some((PhaseChangeFilter { new_phase: 2 }, m)) => {
                // Current epoch
                let epoch_number = (finalized_epochs.len() + 1).into();

                // Inputs of current epoch
                let inputs =
                    self.get_inputs_sync(epoch_number, block.hash).await?;

                // Claims of current epoch
                let claims =
                    self.get_claims_sync(epoch_number, block.hash).await?;

                let sealed_epoch = SealedEpoch {
                    claims,
                    epoch_number,
                    inputs,
                };

                PhaseState::AwaitingDispute {
                    sealed_epoch,
                    current_epoch: first_non_finalized_epoch_inputs,
                }
            }

            // InputAccumulation
            None => PhaseState::InputAccumulation {
                current_epoch: first_non_finalized_epoch_inputs,
            },

            // Err
            Some((PhaseChangeFilter { new_phase }, _)) => {
                return SyncDelegateError {
                    err: format!(
                        "Could not convert new_phase `{}` to PhaseState",
                        new_phase
                    ),
                }
                .fail()
            }
        };

        Ok(DescartesV2State {
            constants,
            initial_epoch: *initial_state,
            current_phase,
            finalized_epochs,
            input_accumulation_start_timestamp,
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let contract = access
            .build_fold_contract(
                self.descartesv2_address,
                block.hash,
                DescartesV2Impl::new,
            )
            .await;

        let constants = previous_state.constants.clone();

        let current_phase = {
            if fold_utils::contains_address(
                &block.logs_bloom,
                &self.descartesv2_address,
            ) {
                let phase_change_events =
                    contract.phase_change_filter().query().await.context(
                        FoldContractError {
                            err: "Error querying for descartes phase change",
                        },
                    )?;

                if let Some(p) = phase_change_events.last() {
                    PhaseState::try_from(p)
                        .map_err(|err| FoldDelegateError { err }.build())?
                } else {
                    previous_state.current_phase.clone()
                }
            } else {
                previous_state.current_phase.clone()
            }
        };

        let finalized_epochs = {
            let epoch_finalized_events =
                contract.finalize_epoch_filter().query().await.context(
                    FoldContractError {
                        err: "Error querying for descartes finalized epochs",
                    },
                )?;

            let mut finalized_epochs: OrdMap<U256, _> =
                previous_state.finalized_epochs.clone();

            for (epoch, ev) in epoch_finalized_events.iter().enumerate() {
                let inputs = self
                    .get_inputs_fold(
                        constants.input_address,
                        epoch.into(),
                        block.hash,
                    )
                    .await?;

                finalized_epochs = finalized_epochs.update(
                    epoch.into(),
                    FinalizedEpoch {
                        hash: ev.epoch_hash.into(),
                        inputs,
                    },
                )
            }
            finalized_epochs
        };

        let current_epoch = {
            let number = finalized_epochs.len().into();

            let inputs = self
                .get_inputs_fold(constants.input_address, number, block.hash)
                .await?;

            let claim_events = contract
                .claim_filter()
                .topic1(U256::from(number))
                .query()
                .await
                .context(FoldContractError {
                    err: "Error querying for descartes phase change",
                })?;

            let mut claimers = if previous_state.current_epoch.number == number
            {
                previous_state.current_epoch.claimers.clone()
            } else {
                HashMap::new()
            };

            for claim in claim_events {
                let set = match claimers.get(&H256::from(claim.epoch_hash)) {
                    None => HashSet::new().update(claim.claimer),
                    Some(set) => set.update(claim.claimer),
                };

                claimers = claimers.update(claim.epoch_hash.into(), set)
            }

            CurrentEpoch {
                number,
                claimers,
                inputs,
            }
        };

        Ok(DescartesV2State {
            constants,
            current_phase,
            initial_epoch: previous_state.initial_epoch,
            finalized_epochs,
            current_epoch,
        })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}

impl<DA: DelegateAccess + Send + Sync + 'static> EpochFoldDelegate<DA> {
    async fn get_inputs_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        epoch: U256,
        block_hash: H256,
    ) -> SyncResult<InputState, A> {
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
    ) -> FoldResult<InputState, A> {
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

    async fn get_claims_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        epoch: U256,
        block_hash: H256,
    ) -> SyncResult<Option<Claims>, A> {
        Ok(self
            .claims_fold
            .get_state_for_block(&epoch, block_hash)
            .await
            .map_err(|e| {
                SyncDelegateError {
                    err: format!("Claim state fold error: {:?}", e),
                }
                .build()
            })?
            .state)
    }

    async fn get_claims_fold<A: FoldAccess + Send + Sync + 'static>(
        &self,
        epoch: U256,
        block_hash: H256,
    ) -> FoldResult<Option<Claims>, A> {
        Ok(self
            .claims_fold
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

// impl std::convert::TryFrom<&PhaseChangeFilter> for PhaseState {
//     type Error = String;
//     fn try_from(
//         p: &PhaseChangeFilter,
//     ) -> std::result::Result<Self, Self::Error> {
//         match p.new_phase {
//             0 => Ok(PhaseState::InputAccumulation),
//             1 => Ok(PhaseState::AwaitingConsensus),
//             2 => Ok(PhaseState::AwaitingDispute),
//             _ => Err(format!(
//                 "Could not convert new_phase `{}` to PhaseState",
//                 p.new_phase
//             )),
//         }
//     }
// }
