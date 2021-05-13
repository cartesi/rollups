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
use im::{HashMap, HashSet, OrdMap};
use snafu::ResultExt;
use std::convert::TryFrom;

use ethers::types::{Address, H256, U256};

#[derive(Clone, Debug, PartialEq)]
pub enum PhaseState {
    InputAccumulation,
    AwaitingConsensus,
    AwaitingDispute,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImmutableState {
    pub input_duration: U256, // duration of input accumulation phase in seconds
    pub challenge_period: U256, // duration of challenge period in seconds

    pub input_address: Address, // contract responsible for inputs
    pub output_address: Address, // contract responsible for ouputs
    pub validator_manager_address: Address, // contract responsible for validators
    pub dispute_manager_address: Address, // contract responsible for dispute resolution
}

#[derive(Clone, Debug)]
pub struct FinalizedEpoch {
    pub hash: H256,
    pub inputs: InputState,
}

#[derive(Clone, Debug)]
pub struct SealedEpoch {
    pub number: U256,
    pub claimers: HashMap<H256, HashSet<Address>>, // Claim -> Set of Addresses with that claim
    pub inputs: InputState,
}

#[derive(Clone, Debug)]
pub struct AccumulatingEpoch {
    pub number: U256,
    pub inputs: InputState,
}

#[derive(Clone, Debug)]
pub struct DescartesV2State {
    // TODO: Add these for frontend.
    // pub input_accumulation_start: U256, // Only used for frontend
    // pub first_claim_TS: Option<U256>, // Only used for frontend
    pub constants: ImmutableState, // Only used for frontend

    pub current_phase: PhaseState,
    pub initial_epoch: U256,
    pub finalized_epochs: OrdMap<U256, FinalizedEpoch>, // EpochNumber -> Epoch
    pub current_epoch: CurrentEpoch,
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
        let contract = access
            .build_sync_contract(
                self.descartesv2_address,
                block.number,
                DescartesV2Impl::new,
            )
            .await;

        let constants = {
            let create_event = {
                let e = contract
                    .descartes_v2_created_filter()
                    .query()
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
                e[0].clone()
            };

            ImmutableState::from(&create_event)
        };

        let current_phase = {
            let phase_change_events =
                contract.phase_change_filter().query().await.context(
                    SyncContractError {
                        err: "Error querying for descartes phase change",
                    },
                )?;

            if let Some(p) = phase_change_events.last() {
                PhaseState::try_from(p)
                    .map_err(|err| SyncDelegateError { err }.build())?
            } else {
                PhaseState::InputAccumulation
            }
        };

        let finalized_epochs = {
            let epoch_finalized_events =
                contract.finalize_epoch_filter().query().await.context(
                    SyncContractError {
                        err: "Error querying for descartes finalized epochs",
                    },
                )?;

            let mut finalized_epochs: OrdMap<U256, _> = OrdMap::new();
            for (epoch, ev) in epoch_finalized_events.iter().enumerate() {
                let inputs = self
                    .get_inputs_sync(
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
                .get_inputs_sync(constants.input_address, number, block.hash)
                .await?;

            let claim_events = contract
                .claim_filter()
                .topic1(U256::from(number))
                .query()
                .await
                .context(SyncContractError {
                    err: "Error querying for descartes phase change",
                })?;

            let mut claimers: HashMap<H256, _> = HashMap::new();
            for claim in claim_events {
                claimers
                    .entry(claim.epoch_hash.into())
                    .or_insert(HashSet::new())
                    .insert(claim.claimer);
            }

            CurrentEpoch {
                number,
                claimers,
                inputs,
            }
        };

        Ok(DescartesV2State {
            constants,
            initial_epoch: *initial_state,
            current_phase,
            finalized_epochs,
            current_epoch,
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

impl std::convert::TryFrom<&PhaseChangeFilter> for PhaseState {
    type Error = String;
    fn try_from(
        p: &PhaseChangeFilter,
    ) -> std::result::Result<Self, Self::Error> {
        match p.new_phase {
            0 => Ok(PhaseState::InputAccumulation),
            1 => Ok(PhaseState::AwaitingConsensus),
            2 => Ok(PhaseState::AwaitingDispute),
            _ => Err(format!(
                "Could not convert new_phase `{}` to PhaseState",
                p.new_phase
            )),
        }
    }
}

impl From<&DescartesV2CreatedFilter> for ImmutableState {
    fn from(ev: &DescartesV2CreatedFilter) -> Self {
        Self {
            input_duration: ev.input_duration,
            challenge_period: ev.challenge_period,
            input_address: ev.input,
            output_address: ev.output,
            validator_manager_address: ev.validator_manager,
            dispute_manager_address: ev.dispute_manager,
        }
    }
}
