use crate::{
    accumulating_epoch::AccumulatingEpoch,
    epoch_initial_state::EpochInitialState,
    finalized_epochs::FinalizedEpochs,
    rollups_initial_state::RollupsInitialState,
    sealed_epoch::{EpochWithClaims, SealedEpochState},
    FoldableError,
};
use anyhow::{anyhow, Context, Error};
use async_trait::async_trait;
use contracts::rollups_facet::*;
use ethers::{prelude::EthEvent, providers::Middleware, types::U256};
use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum ContractPhase {
    InputAccumulation {},
    AwaitingConsensus {
        sealed_epoch: Arc<SealedEpochState>,
        round_start: U256,
    },
    AwaitingDispute {
        sealed_epoch: Arc<EpochWithClaims>,
    },
}

#[derive(Clone, Debug)]
pub struct EpochState {
    pub current_phase: ContractPhase,
    pub finalized_epochs: Arc<FinalizedEpochs>,
    pub current_epoch: Arc<AccumulatingEpoch>,
    /// Timestamp of last contract phase change
    pub phase_change_timestamp: Option<U256>,
    pub rollups_initial_state: Arc<RollupsInitialState>,
}

/// Epoch StateActor Delegate, which implements `sync` and `fold`.
/// It uses the subdelegates to extracts the raw state from blockchain
/// emitted events
#[async_trait]
impl Foldable for EpochState {
    type InitialState = Arc<RollupsInitialState>;
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let contract = RollupsFacet::new(
            *initial_state.dapp_contract_address,
            Arc::clone(&access),
        );

        // retrieve list of finalized epochs from FinalizedEpochFoldDelegate
        let finalized_epochs =
            FinalizedEpochs::get_state_for_block(initial_state, block, env)
                .await
                .context("Finalized epoch state fold error")?
                .state;

        // The index of next epoch is the number of finalized epochs
        let epoch_initial_state = {
            let next_epoch = finalized_epochs.next_epoch();

            Arc::new(EpochInitialState {
                dapp_contract_address: Arc::clone(
                    &initial_state.dapp_contract_address,
                ),
                epoch_number: next_epoch,
            })
        };

        // Retrieve events emitted by the blockchain on phase changes
        let phase_change_events = contract
            .phase_change_filter()
            .query_with_meta()
            .await
            .context("Error querying for rollups phase change")?;

        let phase_change_timestamp = {
            match phase_change_events.last() {
                None => None,
                Some((_, meta)) => Some(
                    access
                        .get_block(meta.block_hash)
                        .await
                        .map_err(|e| FoldableError::from(Error::from(e)))?
                        .context("Block not found")?
                        .timestamp,
                ),
            }
        };

        // Define the current_phase and current_epoch  based on the last
        // phase_change event
        let (current_phase, current_epoch) = match phase_change_events.last() {
            // InputAccumulation
            // either accumulating inputs or sealed epoch with no claims/new inputs
            Some((PhaseChangeFilter { new_phase: 0 }, _)) | None => {
                let current_epoch = AccumulatingEpoch::get_state_for_block(
                    &epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state;
                (ContractPhase::InputAccumulation {}, current_epoch)
            }

            // AwaitingConsensus
            // can be SealedEpochNoClaims or SealedEpochWithClaims
            Some((PhaseChangeFilter { new_phase: 1 }, _)) => {
                let sealed_epoch = SealedEpochState::get_state_for_block(
                    &epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state;

                let current_epoch = AccumulatingEpoch::get_state_for_block(
                    &epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state;

                // Unwrap is safe because, a phase change event guarantees
                // a phase change timestamp
                let round_start = phase_change_timestamp.unwrap();

                (
                    ContractPhase::AwaitingConsensus {
                        sealed_epoch,
                        round_start,
                    },
                    current_epoch,
                )
            }

            // AwaitingDispute
            Some((PhaseChangeFilter { new_phase: 2 }, _)) => {
                let sealed_epoch = SealedEpochState::get_state_for_block(
                    &epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state;

                let current_epoch = AccumulatingEpoch::get_state_for_block(
                    &epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state;

                (
                    ContractPhase::AwaitingDispute {
                        sealed_epoch: match &*sealed_epoch {
                            // If there are no claims then the contract can't
                            // be in AwaitingDispute phase
                            SealedEpochState::SealedEpochNoClaims {
                                sealed_epoch,
                            } => {
                                return Err(anyhow!(
                                    "Illegal state for AwaitingDispute: {:?}",
                                    sealed_epoch
                                )
                                .into());
                            }

                            SealedEpochState::SealedEpochWithClaims {
                                claimed_epoch,
                            } => Arc::clone(claimed_epoch),
                        },
                    },
                    current_epoch,
                )
            }

            // Err
            Some((PhaseChangeFilter { new_phase }, _)) => {
                return Err(anyhow!(
                    "Could not convert new_phase `{}` to PhaseState",
                    new_phase
                )
                .into());
            }
        };

        Ok(EpochState {
            current_phase,
            phase_change_timestamp,
            finalized_epochs,
            current_epoch,
            rollups_initial_state: Arc::clone(&initial_state),
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let dapp_contract_address = Arc::clone(
            &previous_state.rollups_initial_state.dapp_contract_address,
        );

        // Check if there was (possibly) some log emited on this block.
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &dapp_contract_address,
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &PhaseChangeFilter::signature(),
        )) {
            // Current phase has not changed, but we need to update the
            // sub-states.
            let current_epoch = {
                let epoch_initial_state =
                    &previous_state.current_epoch.epoch_initial_state;

                AccumulatingEpoch::get_state_for_block(
                    epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state
            };

            let current_phase = match &previous_state.current_phase {
                ContractPhase::InputAccumulation {} => {
                    ContractPhase::InputAccumulation {}
                }

                ContractPhase::AwaitingConsensus {
                    sealed_epoch,
                    round_start,
                } => {
                    let sealed_epoch = {
                        let epoch_initial_state = Arc::new(EpochInitialState {
                            dapp_contract_address,
                            epoch_number: sealed_epoch.epoch_number(),
                        });

                        SealedEpochState::get_state_for_block(
                            &epoch_initial_state,
                            block,
                            env,
                        )
                        .await?
                        .state
                    };

                    ContractPhase::AwaitingConsensus {
                        sealed_epoch,
                        round_start: *round_start,
                    }
                }

                ContractPhase::AwaitingDispute { sealed_epoch } => {
                    let sealed_epoch = {
                        let epoch_initial_state =
                            &sealed_epoch.epoch_initial_state;

                        SealedEpochState::get_state_for_block(
                            epoch_initial_state,
                            block,
                            env,
                        )
                        .await?
                        .state
                    };

                    ContractPhase::AwaitingDispute {
                        sealed_epoch: match &*sealed_epoch {
                            SealedEpochState::SealedEpochNoClaims {
                                sealed_epoch,
                            } => {
                                return Err(anyhow!(
                                    "Illegal state for AwaitingDispute: {:?}",
                                    sealed_epoch
                                )
                                .into());
                            }
                            SealedEpochState::SealedEpochWithClaims {
                                claimed_epoch,
                            } => Arc::clone(claimed_epoch),
                        },
                    }
                }
            };

            return Ok(EpochState {
                current_phase,
                current_epoch,
                phase_change_timestamp: previous_state.phase_change_timestamp,
                finalized_epochs: previous_state.finalized_epochs.clone(),
                rollups_initial_state: Arc::clone(
                    &previous_state.rollups_initial_state,
                ),
            });
        }

        let contract = RollupsFacet::new(*dapp_contract_address, access);

        let finalized_epochs = FinalizedEpochs::get_state_for_block(
            &previous_state.rollups_initial_state,
            block,
            env,
        )
        .await?
        .state;

        let (epoch_initial_state, next_epoch_initial_state) = {
            let next_epoch = finalized_epochs.next_epoch();

            (
                Arc::new(EpochInitialState {
                    dapp_contract_address: Arc::clone(&dapp_contract_address),
                    epoch_number: next_epoch,
                }),
                Arc::new(EpochInitialState {
                    dapp_contract_address,
                    epoch_number: next_epoch + 1u64,
                }),
            )
        };

        let phase_change_events = contract
            .phase_change_filter()
            .query()
            .await
            .context("Error querying for rollups phase change")?;

        let (current_phase, current_epoch) = match phase_change_events.last() {
            // InputAccumulation
            Some(PhaseChangeFilter { new_phase: 0 }) | None => {
                let current_epoch = AccumulatingEpoch::get_state_for_block(
                    &epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state;
                (ContractPhase::InputAccumulation {}, current_epoch)
            }

            // AwaitingConsensus
            Some(PhaseChangeFilter { new_phase: 1 }) => {
                // If the phase is AwaitingConsensus then there are two epochs
                // not yet finalized. One sealead, which can't receive new
                // inputs and one active, accumulating new inputs
                let sealed_epoch = SealedEpochState::get_state_for_block(
                    &epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state;

                let current_epoch = AccumulatingEpoch::get_state_for_block(
                    &next_epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state;

                // Timestamp of when we entered this phase.
                let round_start = block.timestamp;

                (
                    ContractPhase::AwaitingConsensus {
                        sealed_epoch,
                        round_start,
                    },
                    current_epoch,
                )
            }

            // AwaitingDispute
            Some(PhaseChangeFilter { new_phase: 2 }) => {
                // If the phase is AwaitingDispute then there are two epochs
                // not yet finalized. One sealead, which can't receive new
                // inputs and one active, accumulating new inputs
                let sealed_epoch = SealedEpochState::get_state_for_block(
                    &epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state;

                let current_epoch = AccumulatingEpoch::get_state_for_block(
                    &next_epoch_initial_state,
                    block,
                    env,
                )
                .await?
                .state;

                (
                    ContractPhase::AwaitingDispute {
                        sealed_epoch: match &*sealed_epoch {
                            SealedEpochState::SealedEpochNoClaims {
                                sealed_epoch,
                            } => {
                                return Err(anyhow!(
                                    "Illegal state for AwaitingDispute: {:?}",
                                    sealed_epoch
                                )
                                .into());
                            }
                            SealedEpochState::SealedEpochWithClaims {
                                claimed_epoch,
                            } => Arc::clone(&claimed_epoch),
                        },
                    },
                    current_epoch,
                )
            }

            // Err
            Some(PhaseChangeFilter { new_phase }) => {
                return Err(anyhow!(
                    "Could not convert new_phase `{}` to PhaseState",
                    new_phase
                )
                .into());
            }
        };

        let phase_change_timestamp = if phase_change_events.is_empty() {
            previous_state.phase_change_timestamp
        } else {
            Some(block.timestamp)
        };

        Ok(EpochState {
            current_phase,
            current_epoch,
            phase_change_timestamp,
            finalized_epochs,
            rollups_initial_state: Arc::clone(
                &previous_state.rollups_initial_state,
            ),
        })
    }
}
