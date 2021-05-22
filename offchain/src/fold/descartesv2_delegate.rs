use super::contracts::descartesv2_contract::*;
use super::input_delegate::{InputFoldDelegate, InputState};

use dispatcher::state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils, DelegateAccess, StateFold,
};
use dispatcher::types::Block;

use async_trait::async_trait;
use im::{HashMap, HashSet, Vector};
use snafu::ResultExt;
use std::convert::TryFrom;
use std::sync::Arc;

use ethers::contract::LogMeta;
use ethers::providers::Middleware;
use ethers::types::{Address, H256, U256};

#[derive(Clone, Debug, PartialEq)]
pub struct ImmutableState {
    pub input_duration: U256, // duration of input accumulation phase in seconds
    pub challenge_period: U256, // duration of challenge period in seconds
    pub contract_creation_timestamp: U256, // timestamp of the contract creation

    pub input_contract_address: Address, // contract responsible for inputs
    pub output_contract_address: Address, // contract responsible for ouputs
    pub validator_contract_address: Address, // contract responsible for validators
    pub dispute_contract_address: Address, // contract responsible for dispute resolution
}

#[derive(Clone, Debug)]
pub struct FinalizedEpoch {
    pub hash: H256,
    pub inputs: InputState,
}

#[derive(Clone, Debug)]
pub struct SealedEpoch {
    pub number: U256,

    // Claim -> (Set of Addresses with that claim, timestamp of first claim)
    pub claimers: HashMap<H256, (HashSet<Address>, U256)>,

    pub inputs: InputState,
}

impl SealedEpoch {
    pub fn first_claim_timestamp(&self) -> Option<U256> {
        let mut first_ts = None;
        for (k, (_, ts)) in self.claimers {
            first_ts = match first_ts {
                None => Some(ts),
                Some(x) => Some(std::cmp::min(x, ts)),
            }
        }

        first_ts
    }
}

#[derive(Clone, Debug)]
pub struct AccumulatingEpoch {
    pub number: U256,
    pub inputs: InputState,
}

#[derive(Clone, Debug)]
pub enum PhaseState {
    InputAccumulation {
        current_epoch: AccumulatingEpoch,
    },

    ExpiredInputAccumulation {
        sealing_epoch: AccumulatingEpoch,
    },

    AwaitingConsensus {
        sealed_epoch: SealedEpoch,
        current_epoch: AccumulatingEpoch,
        round_start: U256,
    },

    ConsensusTimeout {
        sealed_epoch: SealedEpoch,
        current_epoch: AccumulatingEpoch,
    },

    AwaitingDispute {
        sealed_epoch: SealedEpoch,
        current_epoch: AccumulatingEpoch,
    },
    // TODO: add dispute timeout when disputes are turned on.
}

impl PhaseState {
    pub fn consensus_round_start(&self) -> Option<U256> {
        match self {
            PhaseState::AwaitingConsensus {
                round_start,
                sealed_epoch,
                ..
            } => match sealed_epoch.first_claim_timestamp() {
                None => None,
                Some(x) => Some(std::cmp::max(*round_start, x)),
            },
            _ => None,
        }
    }

    // pub fn advance_state(
    //     self,
    //     phase_change: PhaseChangeFilter,
    // ) -> Option<Self> {
    //     todo!()
    // }
}

#[derive(Clone, Debug)]
pub struct DescartesV2State {
    // TODO: Add these for frontend.
    // pub first_claim_timestamp: Option<U256>, // Only used for frontend
    pub constants: ImmutableState,
    pub input_accumulation_start_timestamp: U256,

    pub initial_epoch: U256,
    pub finalized_epochs: Vector<FinalizedEpoch>, // EpochNumber -> Epoch

    pub current_phase: PhaseState,
}

impl DescartesV2State {
    pub fn current_epoch(&self) -> usize {
        // TODO: Fix off-by-one error.
        self.finalized_epochs.len()
            + match self.current_phase {
                PhaseState::InputAccumulation { .. } => 0,

                PhaseState::ExpiredInputAccumulation { .. }
                | PhaseState::AwaitingConsensus { .. }
                | PhaseState::ConsensusTimeout { .. }
                | PhaseState::AwaitingDispute { .. } => 1,
            }
    }
}

/// DescartesV2 StateActor Delegate, which implements `sync` and `fold`.
pub struct DescartesV2FoldDelegate<DA: DelegateAccess> {
    descartesv2_address: Address,
    input_fold: StateFold<InputFoldDelegate, DA>,
}

impl<DA: DelegateAccess> DescartesV2FoldDelegate<DA> {
    pub fn new(
        descartesv2_address: Address,
        input_fold: StateFold<InputFoldDelegate, DA>,
    ) -> Self {
        Self {
            descartesv2_address,
            input_fold,
        }
    }
}

#[async_trait]
impl<DA: DelegateAccess + Send + Sync + 'static> StateFoldDelegate
    for DescartesV2FoldDelegate<DA>
{
    type InitialState = U256;
    type Accumulator = DescartesV2State;
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

        let constants = {
            let (create_event, meta) = {
                let e = contract
                    .descartes_v2_created_filter()
                    .query_with_meta()
                    .await
                    .context(SyncContractError {
                        err: "Error querying for descartes created",
                    })?;

                if e.is_empty() {
                    return SyncDelegateError {
                        err: "Descartes create event not found",
                    }
                    .fail();
                }

                assert_eq!(e.len(), 1);
                e[0]
            };

            let timestamp = middleware
                .get_block(meta.block_hash)
                .await
                .context(SyncAccessError {})?
                .ok_or(snafu::NoneError)
                .context(SyncDelegateError {
                    err: "Block not found",
                })?
                .timestamp;

            ImmutableState::from(&(create_event, timestamp))
        };

        let epoch_finalized_events = contract
            .finalize_epoch_filter()
            .query_with_meta()
            .await
            .context(SyncContractError {
                err: "Error querying for descartes finalized epochs",
            })?;

        let finalized_epochs = {
            let mut finalized_epochs = Vector::new();
            for (epoch, (ev, _)) in epoch_finalized_events.iter().enumerate() {
                let inputs = self
                    .get_inputs_sync(
                        constants.input_contract_address,
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

        let input_accumulation_start_timestamp = {
            match epoch_finalized_events.last() {
                None => constants.contract_creation_timestamp,
                Some((_, meta)) => {
                    middleware
                        .get_block(block.hash)
                        .await
                        .context(SyncAccessError {})?
                        .ok_or(snafu::NoneError)
                        .context(SyncDelegateError {
                            err: "Block not found",
                        })?
                        .timestamp
                }
            }
        };

        let phase_change_events = contract
            .phase_change_filter()
            .query_with_meta()
            .await
            .context(SyncContractError {
                err: "Error querying for descartes phase change",
            })?;

        let first_non_finalized_epoch_inputs = {
            // Index of first non finalized epoch
            let number = finalized_epochs.len().into();

            // Inputs of first non finalized epoch
            let inputs = self
                .get_inputs_sync(
                    constants.input_contract_address,
                    number,
                    block.hash,
                )
                .await?;

            AccumulatingEpoch { inputs, number }
        };

        let current_phase = match phase_change_events.last() {
            // InputAccumulation
            Some((PhaseChangeFilter { new_phase: 0 }, _)) => {
                if block.timestamp
                    > input_accumulation_start_timestamp
                        + constants.input_duration
                {
                    PhaseState::ExpiredInputAccumulation {
                        sealing_epoch: first_non_finalized_epoch_inputs,
                    }
                } else {
                    PhaseState::InputAccumulation {
                        current_epoch: first_non_finalized_epoch_inputs,
                    }
                }
            }

            // AwaitingConsensus | AwaitingDispute
            Some((phase @ PhaseChangeFilter { new_phase: 1 }, m))
            | Some((phase @ PhaseChangeFilter { new_phase: 2 }, m)) => {
                // Current epoch
                let number = (finalized_epochs.len() + 1).into();

                // Inputs of current epoch
                let inputs = self
                    .get_inputs_sync(
                        constants.input_contract_address,
                        number,
                        block.hash,
                    )
                    .await?;

                // Get all claim events of current epoch
                let claim_events = contract
                    .claim_filter()
                    .topic1(U256::from(number))
                    .query_with_meta()
                    .await
                    .context(SyncContractError {
                        err: "Error querying for descartes phase change",
                    })?;

                let mut claimers: HashMap<H256, _> = HashMap::new();
                for (claim_event, meta) in claim_events {
                    let x = claimers
                        .entry(claim_event.epoch_hash.into())
                        .or_insert((
                            HashSet::new(),
                            middleware
                                .get_block(meta.block_hash)
                                .await
                                .context(SyncAccessError {})?
                                .ok_or(snafu::NoneError)
                                .context(SyncDelegateError {
                                    err: "Block not found",
                                })?
                                .timestamp,
                        ));

                    x.0.insert(claim_event.claimer);
                }

                let sealed_epoch = SealedEpoch {
                    claimers,
                    number,
                    inputs,
                };

                if phase.new_phase == 1 {
                    // AwaitingConsensus

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

                    let awaiting_consensus = PhaseState::AwaitingConsensus {
                        sealed_epoch,
                        current_epoch: first_non_finalized_epoch_inputs,
                        round_start,
                    };

                    match awaiting_consensus.consensus_round_start() {
                        // No claims
                        None => awaiting_consensus,

                        // Some claims
                        Some(ts) => {
                            if block.timestamp > ts + constants.challenge_period
                            {
                                PhaseState::ConsensusTimeout {
                                    sealed_epoch,
                                    current_epoch:
                                        first_non_finalized_epoch_inputs,
                                }
                            } else {
                                awaiting_consensus
                            }
                        }
                    }
                } else {
                    // AwaitingDispute
                    PhaseState::AwaitingDispute {
                        sealed_epoch,
                        current_epoch: first_non_finalized_epoch_inputs,
                    }
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

        let first_non_finalized_epoch_inputs = {
            // Index of first non finalized epoch
            let number = finalized_epochs.len().into();

            // Inputs of first non finalized epoch
            let inputs = self
                .get_inputs_fold(
                    constants.input_contract_address,
                    number,
                    block.hash,
                )
                .await?;

            AccumulatingEpoch { inputs, number }
        };

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
                    match p.new_phase {
                        0 => PhaseState::InputAccumulation {
                            current_epoch: first_non_finalized_epoch_inputs,
                        },
                        1 => PhaseState::AwaitingConsensus,
                        2 => PhaseState::AwaitingDispute,
                        _ => {
                            return Err(format!(
                            "Could not convert new_phase `{}` to PhaseState",
                            p.new_phase
                        ))
                        }
                    }
                    // PhaseState::try_from(p)
                    //     .map_err(|err| FoldDelegateError { err }.build())?
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

impl<DA: DelegateAccess + Send + Sync + 'static> DescartesV2FoldDelegate<DA> {
    async fn get_inputs_sync<A: SyncAccess + Send + Sync + 'static>(
        &self,
        input_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> SyncResult<InputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(&(input_address, epoch), block_hash)
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
        input_address: Address,
        epoch: U256,
        block_hash: H256,
    ) -> FoldResult<InputState, A> {
        Ok(self
            .input_fold
            .get_state_for_block(&(input_address, epoch), block_hash)
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

impl From<&(DescartesV2CreatedFilter, U256)> for ImmutableState {
    fn from(src: &(DescartesV2CreatedFilter, U256)) -> Self {
        let (ev, ts) = src;
        Self {
            input_duration: ev.input_duration,
            challenge_period: ev.challenge_period,
            contract_creation_timestamp: ts.clone(),
            input_contract_address: ev.input,
            output_contract_address: ev.output,
            validator_contract_address: ev.validator_manager,
            dispute_contract_address: ev.dispute_manager,
        }
    }
}
