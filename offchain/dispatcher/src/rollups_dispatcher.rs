use crate::machine::{EpochStatus, MachineInterface};
use crate::tx_sender::TxSender;

use types::{
    fee_manager::FeeIncentiveStrategy, input::Input, rollups::PhaseState,
    rollups::RollupsState,
};

use anyhow::Result;
use async_recursion::async_recursion;
use im::Vector;
use std::sync::Arc;

use state_fold_types::{ethers::types::Address, BlockState};

use tracing::{instrument, trace, warn};

#[derive(Debug)]
pub struct RollupsDispatcher<MM> {
    machine_manager: MM,
    fee_incentive_strategy: FeeIncentiveStrategy,
    sender: Address,
}

impl<MM> RollupsDispatcher<MM>
where
    MM: MachineInterface + Sync + Send,
{
    pub fn new(
        machine_manager: MM,
        fee_incentive_strategy: FeeIncentiveStrategy,
        sender: Address,
    ) -> Self {
        Self {
            machine_manager,
            fee_incentive_strategy,
            sender,
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn react<TS: TxSender + Sync + Send>(
        &self,
        block_state: BlockState<RollupsState>,
        mut tx_sender: TS,
    ) -> Result<TS> {
        let state = block_state.state;

        {
            // redeem fees if the number of redeemable claims has reached the
            // trigger level and the bank has enough balance.
            if state.fee_manager_state.should_redeem(
                &state.validator_manager_state,
                self.sender,
                &self.fee_incentive_strategy,
            ) {
                tx_sender = tx_sender.send_redeem_tx().await?;
            }

            // Will work if fee manager has sufficient uncommitted balance or
            // if the node is altruistic.
            let should_work = state.fee_manager_state.should_work(
                &state.validator_manager_state,
                &self.fee_incentive_strategy,
            );

            if !should_work {
                warn!("Fee Manager has insufficient uncommitted balance");
                return Ok(tx_sender);
            }
        }

        trace!("Querying machine manager for current epoch status");
        let mm_epoch_status =
            self.machine_manager.get_current_epoch_status().await?;

        trace!("Current epoch status: {:#?}", mm_epoch_status);

        let (should_continue, mm_epoch_status) = self
            .enqueue_inputs_of_finalized_epochs(&state, mm_epoch_status)
            .await?;

        if !should_continue {
            trace!("Machine manager finalization pending; exiting `react`");
            return Ok(tx_sender);
        }

        match &*state.current_phase {
            PhaseState::InputAccumulation {} => {
                trace!("InputAccumulation phase; enqueueing inputs");

                // Discover latest MM accumulating input index
                // Enqueue diff one by one
                self.enqueue_inputs_for_accumulating_epoch(
                    &mm_epoch_status,
                    &state,
                )
                .await?;

                // React idle.
                return Ok(tx_sender);
            }

            PhaseState::EpochSealedAwaitingFirstClaim { sealed_epoch } => {
                trace!("EpochSealedAwaitingFirstClaim phase");
                trace!("Sealed epoch: {:#?}", sealed_epoch);

                // On EpochSealedAwaitingFirstClaim we have two unfinalized epochs:
                // sealed and accumulating.

                // If MM is on sealed epoch, discover latest MM input index.
                // enqueue remaining inputs and SessionFinishEpochRequest.
                // React claim.

                // Then, enqueue accumulating inputs.

                // If MM is on accumulating epoch, get claim of previous
                // epoch (sealed) and React claim
                let sealed_epoch_number = state.finalized_epochs.next_epoch();
                let mm_epoch_status = if mm_epoch_status.epoch_number
                    == sealed_epoch_number
                {
                    trace!("Machine manager is on sealed epoch");

                    let all_inputs_processed = self
                        .update_sealed_epoch(
                            &sealed_epoch.inputs.inputs,
                            &mm_epoch_status,
                        )
                        .await?;

                    if !all_inputs_processed {
                        // React Idle
                        trace!("Machine manager has unprocessed inputs; exiting `react`");
                        return Ok(tx_sender);
                    }

                    self.machine_manager.get_current_epoch_status().await?
                } else {
                    mm_epoch_status
                };

                // Enqueue accumulating epoch.
                self.enqueue_inputs_for_accumulating_epoch(
                    &mm_epoch_status,
                    &state,
                )
                .await?;

                trace!("Querying machine manager for sealed epoch claim");

                let claim = self
                    .machine_manager
                    .get_epoch_claim(sealed_epoch_number)
                    .await?;

                trace!("Machine manager returned claim `{}`; sending transaction for first claim...", claim);

                tx_sender =
                    tx_sender.send_claim_tx(claim, sealed_epoch_number).await?;

                return Ok(tx_sender);
            }

            PhaseState::AwaitingConsensusNoConflict { claimed_epoch }
            | PhaseState::AwaitingConsensusAfterConflict {
                claimed_epoch,
                ..
            } => {
                trace!("AwaitingConsensusNoConflict or AwaitingConsensusAfterConflict  phase");
                trace!("Claimed epoch: {:#?}", claimed_epoch);

                let sealed_epoch_number = state.finalized_epochs.next_epoch();
                let mm_epoch_status = if mm_epoch_status.epoch_number
                    == sealed_epoch_number
                {
                    trace!("Machine manager is on sealed epoch");

                    let all_inputs_processed = self
                        .update_sealed_epoch(
                            &claimed_epoch.inputs.inputs,
                            &mm_epoch_status,
                        )
                        .await?;

                    if !all_inputs_processed {
                        // React Idle
                        trace!("Machine manager has unprocessed inputs; exiting `react`");
                        return Ok(tx_sender);
                    }

                    self.machine_manager.get_current_epoch_status().await?
                } else {
                    mm_epoch_status
                };

                // Enqueue accumulating epoch.
                self.enqueue_inputs_for_accumulating_epoch(
                    &mm_epoch_status,
                    &state,
                )
                .await?;

                let sender_claim =
                    claimed_epoch.claims.get_sender_claim(&self.sender);
                if sender_claim.is_none() {
                    trace!("Validator has sent no claims yet; getting claim from machine manager...");

                    let claim = self
                        .machine_manager
                        .get_epoch_claim(sealed_epoch_number)
                        .await?;

                    trace!("Machine manager returned claim `{}`; sending transaction for claim...", claim);

                    tx_sender = tx_sender
                        .send_claim_tx(claim, sealed_epoch_number)
                        .await?;

                    return Ok(tx_sender);
                }

                trace!("Validator has sent claim for this epoch");

                // On AwaitingConsensusConflict we have two unfinalized epochs:
                // claimed and accumulating.
                //
                // If MM is on sealed epoch, discover latest MM input index.
                // enqueue remaining inputs and SessionFinishEpochRequest.
                //
                // Check if validator's address has claimed, if not call
                // SessionFinishEpochRequest and
                // React claim.
                //
                // Then, enqueue accumulating inputs.
            }

            PhaseState::ConsensusTimeout { claimed_epoch } => {
                trace!("ConsensusTimeout phase");
                trace!("Claimed epoch: {:#?}", claimed_epoch);

                let sealed_epoch_number = state.finalized_epochs.next_epoch();
                let mm_epoch_status = if mm_epoch_status.epoch_number
                    == sealed_epoch_number
                {
                    trace!("Machine manager is on sealed epoch");

                    let all_inputs_processed = self
                        .update_sealed_epoch(
                            &claimed_epoch.inputs.inputs,
                            &mm_epoch_status,
                        )
                        .await?;

                    if !all_inputs_processed {
                        // React Idle
                        trace!("Machine manager has unprocessed inputs; exiting `react`");
                        return Ok(tx_sender);
                    }
                    self.machine_manager.get_current_epoch_status().await?
                } else {
                    mm_epoch_status
                };

                // Enqueue accumulating epoch.
                self.enqueue_inputs_for_accumulating_epoch(
                    &mm_epoch_status,
                    &state,
                )
                .await?;

                let sender_claim =
                    claimed_epoch.claims.get_sender_claim(&self.sender);
                if sender_claim.is_none() {
                    trace!("Validator has sent no claims yet; getting claim from machine manager...");

                    let claim = self
                        .machine_manager
                        .get_epoch_claim(sealed_epoch_number)
                        .await?;

                    trace!("Machine manager returned claim `{}`; sending transaction claim...", claim);

                    tx_sender = tx_sender
                        .send_claim_tx(claim, sealed_epoch_number)
                        .await?;

                    trace!("Claim sent; exiting `react`");

                    return Ok(tx_sender);
                } else {
                    trace!("Validator has sent a claim; sending finalize epoch transaction");

                    tx_sender =
                        tx_sender.send_finalize_tx(sealed_epoch_number).await?;

                    trace!("Finalize transaction sent; exiting `react`");

                    return Ok(tx_sender);
                }

                // On ConsensusTimeout we have two unfinalized epochs:
                // claimed and accumulating.
                //
                // If MM is on claimed epoch, discover latest MM input index.
                // enqueue remaining inputs and SessionFinishEpochRequest.
                //
                // Check if validator local claim for claimed epoch matches
                // the claim currently standing onchain.
                // If yes, React finalizeEpoch()
                // If not, React claim()
                //
                // Then, enqueue accumulating inputs.
            }

            // Unreacheable
            PhaseState::AwaitingDispute { .. } => {
                unreachable!()
            }
        }

        Ok(tx_sender)
    }
}

impl<MM> RollupsDispatcher<MM>
where
    MM: MachineInterface + Sync + Send,
{
    /// Returns true if react can continue, false otherwise, as well as the new
    /// `mm_epoch_status`.
    #[async_recursion]
    #[instrument(level = "trace", skip_all)]
    async fn enqueue_inputs_of_finalized_epochs(
        &self,
        state: &RollupsState,
        mm_epoch_status: EpochStatus,
    ) -> Result<(bool, EpochStatus)> {
        // Checking if there are finalized_epochs beyond the machine manager.
        // TODO: comment on index compare.
        if mm_epoch_status.epoch_number >= state.finalized_epochs.next_epoch() {
            trace!(
            "Machine manager epoch number `{}` ahead of finalized_epochs `{}`",
            mm_epoch_status.epoch_number,
            state.finalized_epochs.next_epoch()
        );
            return Ok((true, mm_epoch_status));
        }

        let inputs = &state
            .finalized_epochs
            .get_epoch(mm_epoch_status.epoch_number.as_usize())
            .expect(
                "We should have more `finalized_epochs` than machine manager",
            )
            .inputs;

        trace!(
            "Got `{}` inputs for epoch `{}`; machine manager in epoch `{}`",
            inputs.inputs.len(),
            inputs.epoch_initial_state.epoch_number,
            mm_epoch_status.epoch_number
        );

        if mm_epoch_status.processed_input_count == inputs.inputs.len() {
            trace!("Machine manager has processed all inputs of epoch");
            assert_eq!(
                mm_epoch_status.pending_input_count, 0,
                "Pending input count should be zero"
            );

            // Call finish
            trace!(
                "Finishing epoch `{}` on machine manager with `{}` inputs",
                inputs.epoch_initial_state.epoch_number,
                inputs.inputs.len()
            );
            self.machine_manager
                .finish_epoch(
                    mm_epoch_status.epoch_number,
                    mm_epoch_status.processed_input_count.into(),
                )
                .await?;

            trace!("Querying machine manager current epoch status");
            let mm_epoch_status =
                self.machine_manager.get_current_epoch_status().await?;

            trace!(
                "New machine manager epoch `{}`",
                mm_epoch_status.epoch_number
            );

            // recursively call enqueue_inputs_of_finalized_epochs
            trace!("Recursively call `enqueue_inputs_of_finalized_epochs` for new status");
            return self
                .enqueue_inputs_of_finalized_epochs(&state, mm_epoch_status)
                .await;
        }

        self.enqueue_remaning_inputs(&mm_epoch_status, &inputs.inputs)
            .await?;

        Ok((false, mm_epoch_status))
    }

    #[instrument(level = "trace", skip_all)]
    async fn enqueue_inputs_for_accumulating_epoch(
        &self,
        mm_epoch_status: &EpochStatus,
        state: &RollupsState,
    ) -> Result<bool> {
        trace!(
            "Enqueue inputs for accumulating epoch `{}`; machine manager in epoch `{}`",
            state.current_epoch.epoch_initial_state.epoch_number,
            mm_epoch_status.epoch_number
        );

        // Enqueue accumulating epoch.
        self.enqueue_remaning_inputs(
            &mm_epoch_status,
            &state.current_epoch.inputs.inputs,
        )
        .await
    }

    #[instrument(level = "trace", skip_all)]
    async fn update_sealed_epoch(
        &self,
        sealed_inputs: &Vector<Arc<Input>>,
        mm_epoch_status: &EpochStatus,
    ) -> Result<bool> {
        let all_inputs_processed = self
            .enqueue_remaning_inputs(&mm_epoch_status, &sealed_inputs)
            .await?;

        if !all_inputs_processed {
            trace!("There are still inputs to be processed");
            return Ok(false);
        }

        trace!(
            "Finishing epoch `{}` on machine manager with `{}` inputs",
            mm_epoch_status.epoch_number,
            mm_epoch_status.processed_input_count
        );

        self.machine_manager
            .finish_epoch(
                mm_epoch_status.epoch_number,
                mm_epoch_status.processed_input_count.into(),
            )
            .await?;

        Ok(true)
    }

    #[instrument(level = "trace", skip_all)]
    async fn enqueue_remaning_inputs(
        &self,
        mm_epoch_status: &EpochStatus,
        inputs: &Vector<Arc<Input>>,
    ) -> Result<bool> {
        if mm_epoch_status.processed_input_count == inputs.len() {
            trace!("Machine manager has processed all current inputs");
            return Ok(true);
        }

        let inputs_sent_count = mm_epoch_status.processed_input_count
            + mm_epoch_status.pending_input_count;

        trace!("Number of inputs in machine manager: {}", inputs_sent_count);

        let input_slice = inputs.clone().slice(inputs_sent_count..);

        trace!(
            "Machine manager enqueueing inputs from `{}` to `{}`",
            inputs_sent_count,
            inputs.len()
        );

        self.machine_manager
            .enqueue_inputs(
                mm_epoch_status.epoch_number,
                inputs_sent_count.into(),
                input_slice,
            )
            .await?;

        Ok(false)
    }
}
