use super::contracts::descartesv2_contract::*;

use dispatcher::state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils,
};
use dispatcher::types::Block;

use async_trait::async_trait;
use im::{HashMap, HashSet, OrdMap};
use snafu::ResultExt;
use std::convert::{TryFrom, TryInto};

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

    pub input: Address,  // contract responsible for inputs
    pub output: Address, // contract responsible for ouputs
    pub validator_manager: Address, // contract responsible for validators
    pub dispute_manager: Address, // contract responsible for dispute resolution
}

#[derive(Clone, Debug)]
pub struct FinalizedEpoch {
    pub hash: H256,
    pub inputs: OrdMap<U256, Input>,
}

#[derive(Clone, Debug, Default)]
pub struct CurrentEpoch {
    pub hash: Option<H256>,
    pub number: U256,
    pub claimers: HashMap<H256, HashSet<Address>>, // Claim -> Set of Addresses with that claim
    pub inputs: OrdMap<U256, Input>,
}

#[derive(Clone, Debug)]
pub struct Input {
    pub sender: Address,
    pub hash: H256,
    pub timestamp: U256,
}

// #[derive(Clone, Debug)]
// pub struct Inputs {
// }

#[derive(Clone, Debug)]
pub struct DescartesV2State {
    pub constants: ImmutableState, // Only used for frontend
    pub input_accumulation_start: U256, // Only used for frontend
    pub first_claim_TS: U256,      // Only used for frontend

    pub current_phase: PhaseState,
    pub finalized_epochs: OrdMap<U256, FinalizedEpoch>, // EpochNumber -> Epoch
    pub current_epoch: CurrentEpoch,
    // pub inputs_per_epoch: OrdMap<U256, Inputs>, // EpochNumber -> Inputs
}

/// Partition StateActor Delegate, which implements `sync` and `fold`.
pub struct DescartesV2FoldDelegate {
    descartesv2_address: Address,
}

impl DescartesV2FoldDelegate {
    pub fn new(descartesv2_address: Address) -> Self {
        Self {
            descartesv2_address,
        }
    }
}

#[async_trait]
impl StateFoldDelegate for DescartesV2FoldDelegate {
    type InitialState = ();
    type Accumulator = DescartesV2State;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        initial_state: &(),
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
                e[0]
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

        todo!()
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        // Check if is on final state.
        if let Some(p) = &previous_state.partition {
            if p.current_state.is_final_state() {
                return Ok(previous_state.clone());
            }
        }

        // Check if there was (possibly) some log emited on this block.
        let bloom = block.logs_bloom;
        let instance = previous_state.instance.clone();
        if !(fold_utils::contains_address(&bloom, &self.partition_address)
            && fold_utils::contains_topic(&bloom, &instance))
        {
            return Ok(previous_state.clone());
        }

        // Create contract type
        let contract = access
            .build_fold_contract(
                self.partition_address,
                block.hash,
                PartitionInstantiator::new,
            )
            .await;

        // Check if is `None` and if there was a create event on this block.
        if let None = &previous_state.partition {
            // Get all start and end instance events.
            let created = {
                let start_events = contract
                    .partition_created_filter()
                    .topic1(instance)
                    .query()
                    .await
                    .context(FoldContractError {
                        err: "Error querying for partition created",
                    })?;

                !start_events.is_empty()
            };

            // If no create event on this block, return None.
            if !created {
                return Ok(PartitionState {
                    instance,
                    partition: None,
                });
            }
        }

        // If we reached here, we have to fetch the new state.
        let partition = contract
            .get_state(instance, Address::zero())
            .call()
            .await
            .context(FoldContractError {
                err: "Error calling get_state",
            })?
            .try_into()
            .map_err(|e| {
                FoldDelegateError {
                    err: format!("Could not parse partition state: {}", e),
                }
                .build()
            })?;

        Ok(PartitionState {
            instance,
            partition: Some(partition),
        })
    }

    fn convert(&self, state: &BlockState<Self::Accumulator>) -> Self::State {
        state.clone()
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
            input: ev.input,
            output: ev.output,
            validator_manager: ev.validator_manager,
            dispute_manager: ev.dispute_manager,
        }
    }
}
