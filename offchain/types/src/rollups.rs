use crate::{
    accumulating_epoch::AccumulatingEpoch,
    epoch::{ContractPhase, EpochState},
    epoch_initial_state::EpochInitialState,
    fee_manager::FeeManagerState,
    finalized_epochs::FinalizedEpochs,
    output::OutputState,
    rollups_initial_state::RollupsInitialState,
    sealed_epoch::{EpochWithClaims, SealedEpochState},
    validator_manager::ValidatorManagerState,
    FoldableError,
};
use anyhow::{anyhow, Context, Error};
use async_trait::async_trait;
use contracts::diamond_init::*;
use ethers::{providers::Middleware, types::U256};
use serde::{Deserialize, Serialize};
use state_fold::{
    FoldMiddleware, Foldable, StateFoldEnvironment, SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PhaseState {
    /// No claims or disputes going on, the previous epoch was finalized
    /// successfully and the current epoch is still accumulating inputs
    InputAccumulation {},

    /// `current_epoch` is no longer accepting inputs but hasn't yet received
    /// a claim
    EpochSealedAwaitingFirstClaim {
        sealed_epoch: Arc<AccumulatingEpoch>,
    },

    /// Epoch has been claimed but a dispute has yet to arise
    AwaitingConsensusNoConflict { claimed_epoch: Arc<EpochWithClaims> },

    /// Epoch being claimed was previously challenged and there is a standing
    /// claim that can be challenged
    AwaitingConsensusAfterConflict {
        claimed_epoch: Arc<EpochWithClaims>,
        challenge_period_base_ts: U256,
    },

    /// Consensus was not reached but the last 'challenge_period' is over. Epoch
    /// can be finalized at any time by anyone
    ConsensusTimeout { claimed_epoch: Arc<EpochWithClaims> },

    /// Unreacheable
    AwaitingDispute { claimed_epoch: Arc<EpochWithClaims> },
    // TODO: add dispute timeout when disputes are turned on.
}

impl Default for PhaseState {
    fn default() -> Self {
        Self::InputAccumulation {}
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImmutableState {
    /// duration of input accumulation phase in seconds
    pub input_duration: U256,

    /// duration of challenge period in seconds
    pub challenge_period: U256,

    /// timestamp of the contract creation
    pub contract_creation_timestamp: U256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollupsState {
    pub constants: Arc<ImmutableState>,
    pub rollups_initial_state: Arc<RollupsInitialState>,

    pub finalized_epochs: Arc<FinalizedEpochs>,
    pub current_epoch: Arc<AccumulatingEpoch>,

    pub current_phase: Arc<PhaseState>,

    pub output_state: Arc<OutputState>,
    pub validator_manager_state: Arc<ValidatorManagerState>,
    pub fee_manager_state: Arc<FeeManagerState>,
}

#[async_trait]
impl Foldable for RollupsState {
    type InitialState = Arc<RollupsInitialState>;
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let diamond_init = DiamondInit::new(
            *initial_state.dapp_contract_address,
            Arc::clone(&access),
        );

        // Retrieve constants from contract creation event
        let constants = {
            let (create_event, meta) = {
                let e = diamond_init
                    .rollups_initialized_filter()
                    .query_with_meta()
                    .await
                    .context("Error querying for rollups initialized")?;

                if e.is_empty() {
                    return Err(FoldableError::from(anyhow!(
                        "Rollups initialization event not found"
                    )));
                }

                assert_eq!(e.len(), 1);
                e[0].clone()
            };

            // retrieve timestamp of creation
            let timestamp = access
                .get_block(meta.block_hash)
                .await
                .map_err(|e| FoldableError::from(Error::from(e)))?
                .context("Block not found")?
                .timestamp;

            Arc::new(ImmutableState::from(&(create_event, timestamp)))
        };

        let raw_contract_state =
            EpochState::get_state_for_block(&initial_state, block, env)
                .await?
                .state;

        let output_state = OutputState::get_state_for_block(
            &initial_state.dapp_contract_address,
            block,
            env,
        )
        .await?
        .state;

        let validator_manager_state =
            ValidatorManagerState::get_state_for_block(
                &initial_state.dapp_contract_address,
                block,
                env,
            )
            .await?
            .state;

        let fee_manager_state = FeeManagerState::get_state_for_block(
            &initial_state.dapp_contract_address,
            block,
            env,
        )
        .await?
        .state;

        Ok(convert_raw_to_logical(
            raw_contract_state,
            constants,
            block,
            Arc::clone(initial_state),
            output_state,
            validator_manager_state,
            fee_manager_state,
        ))
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        _access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let dapp_contract_address =
            &previous_state.rollups_initial_state.dapp_contract_address;

        let raw_contract_state = EpochState::get_state_for_block(
            &previous_state.rollups_initial_state,
            block,
            env,
        )
        .await?
        .state;

        let output_state =
            OutputState::get_state_for_block(dapp_contract_address, block, env)
                .await?
                .state;

        let validator_manager_state =
            ValidatorManagerState::get_state_for_block(
                dapp_contract_address,
                block,
                env,
            )
            .await?
            .state;

        let fee_manager_state = FeeManagerState::get_state_for_block(
            &dapp_contract_address,
            block,
            env,
        )
        .await?
        .state;

        Ok(convert_raw_to_logical(
            raw_contract_state,
            Arc::clone(&previous_state.constants),
            block,
            Arc::clone(&previous_state.rollups_initial_state),
            output_state,
            validator_manager_state,
            fee_manager_state,
        ))
    }
}

// Convert raw state to logical state. Raw state is the literal interpretation
// of what is being presented by the blockchain. Logical state is the semantic
// intepretation of that, which will be used for offchain decision making
fn convert_raw_to_logical(
    contract_state: Arc<EpochState>,
    constants: Arc<ImmutableState>,
    block: &Block,
    rollups_initial_state: Arc<RollupsInitialState>,
    output_state: Arc<OutputState>,
    validator_manager_state: Arc<ValidatorManagerState>,
    fee_manager_state: Arc<FeeManagerState>,
) -> RollupsState {
    // If the raw state is InputAccumulation but it has expired, then the raw
    // state's `current_epoch` becomes the sealed epoch, and the logic state's
    // `current_epoch` is empty.
    // This variable contains `Some(epoch_number)` in this case, and `None`
    // otherwise.
    // This is possible because a new input after InputAccumulation has expired
    // would trigger a phase change to AwaitingConsensus.
    let mut current_epoch_no_inputs: Option<U256> = None;

    let phase_state = Arc::new(match &contract_state.current_phase {
        ContractPhase::InputAccumulation {} => {
            // Last phase change timestamp is the timestamp of input
            // accumulation start if contract in InputAccumulation.
            // If there were no phase changes, it is the timestamp of
            // contract creation.
            let input_accumulation_start_timestamp =
                if let Some(ts) = contract_state.phase_change_timestamp {
                    ts
                } else {
                    constants.contract_creation_timestamp
                };

            // If input duration has passed, the logical state is epoch sealed
            // awaiting first claim. The raw state can still be InputAccumulation
            // if there were no new inputs after the phase expired.
            if block.timestamp
                > input_accumulation_start_timestamp + constants.input_duration
            {
                current_epoch_no_inputs = Some(
                    contract_state
                        .current_epoch
                        .epoch_initial_state
                        .epoch_number
                        + 1u64,
                );

                PhaseState::EpochSealedAwaitingFirstClaim {
                    sealed_epoch: contract_state.current_epoch.clone(),
                }
            } else {
                PhaseState::InputAccumulation {}
            }
        }

        ContractPhase::AwaitingConsensus {
            sealed_epoch,
            round_start,
        } => {
            let sealed_epoch = Arc::clone(sealed_epoch);

            // The raw phase change might have happened because a claim arrived
            // or because a new input arrived. This determines if the logical
            // phase is EpochAwaintFirstClaim or SealedEpochNoClaims
            match *sealed_epoch {
                SealedEpochState::SealedEpochNoClaims { ref sealed_epoch } => {
                    let sealed_epoch = Arc::clone(sealed_epoch);
                    PhaseState::EpochSealedAwaitingFirstClaim { sealed_epoch }
                }

                SealedEpochState::SealedEpochWithClaims {
                    ref claimed_epoch,
                } => {
                    let claimed_epoch = Arc::clone(claimed_epoch);

                    let first_claim_timestamp =
                        claimed_epoch.claims.first_claim_timestamp();

                    // We can safely unwrap because we can be sure
                    // there was at least one phase change event.
                    // let phase_change_timestamp =
                    //     contract_state.phase_change_timestamp.unwrap();
                    let phase_change_timestamp = round_start;

                    // Last move's timestamp is the most recent timestamp between
                    // the first claim or the phase change. This happens because
                    // the 'challenge period' starts on first claim but resets
                    // after a dispute.
                    let time_of_last_move = std::cmp::max(
                        first_claim_timestamp,
                        *phase_change_timestamp,
                    );

                    // Check if Consensus timed out or, using the first claim
                    // timestamp variable, decide if this is the first challenge
                    // period of this epoch or if it is posterior to a dispute
                    if block.timestamp
                        > time_of_last_move + constants.challenge_period
                    {
                        PhaseState::ConsensusTimeout { claimed_epoch }
                    } else if time_of_last_move == first_claim_timestamp {
                        PhaseState::AwaitingConsensusNoConflict {
                            claimed_epoch,
                        }
                    } else {
                        PhaseState::AwaitingConsensusAfterConflict {
                            claimed_epoch,
                            challenge_period_base_ts: *phase_change_timestamp,
                        }
                    }
                }
            }
        }

        // This version doesn't have disputes. They're resolved automatically
        // onchain
        ContractPhase::AwaitingDispute { .. } => {
            unreachable!()
        }
    });

    // Figures out if the current accumulating epoch is empty (new) or if it
    // was previously created. The distinction comes from the two possible
    // transitions to AwaitingConsensus, either a new input or a claim
    let current_epoch = if let Some(epoch_number) = current_epoch_no_inputs {
        let epoch_initial_state = Arc::new(EpochInitialState {
            dapp_contract_address: Arc::clone(
                &rollups_initial_state.dapp_contract_address,
            ),
            epoch_number,
        });

        AccumulatingEpoch::new(epoch_initial_state)
    } else {
        Arc::clone(&contract_state.current_epoch)
    };

    RollupsState {
        constants,
        current_phase: phase_state,
        finalized_epochs: Arc::clone(&contract_state.finalized_epochs),
        current_epoch,
        output_state,
        validator_manager_state,
        fee_manager_state,
        rollups_initial_state: Arc::clone(&rollups_initial_state),
    }
}

// Fetches the Rollups constants from the contract creation event
impl From<&(RollupsInitializedFilter, U256)> for ImmutableState {
    fn from(src: &(RollupsInitializedFilter, U256)) -> Self {
        let (ev, ts) = src;
        Self {
            input_duration: ev.input_duration,
            challenge_period: ev.challenge_period,
            contract_creation_timestamp: ts.clone(),
        }
    }
}
