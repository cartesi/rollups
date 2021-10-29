use offchain_core::ethers;

use super::instantiate_block_subscriber::instantiate_block_subscriber;
use super::instantiate_tx_manager::{
    instantiate_tx_manager, DescartesTxManager,
};

use super::config::LogicConfig;
use crate::config::ApplicationConfig;
use crate::contracts::descartesv2_contract::DescartesV2Impl;
use crate::error::*;
use crate::fold::types::*;
use crate::machine::{rollup_mm, EpochStatus, MachineInterface};
use crate::rollups_state_fold::RollupsStateFold;

use block_subscriber::NewBlockSubscriber;
use tx_manager::types::ResubmitStrategy;

use async_recursion::async_recursion;
use ethers::core::types::{Address, H256, U256};
use ethers::providers::{MockProvider, Provider};
use im::Vector;
use std::sync::Arc;

// pub struct Config {
//     pub sender: Address,

//     pub descartes_contract_address: Address,
//     pub signer_http_endpoint: String,
//     pub ws_endpoint: String,
//     pub state_fold_grpc_endpoint: String,

//     pub initial_epoch: U256,

//     // pub call_timeout: std::time::Duration,
//     // pub subscriber_timeout: std::time::Duration,
//     pub gas_multiplier: Option<f64>,
//     pub gas_price_multiplier: Option<f64>,
//     pub rate: usize,
//     pub confirmations: usize,

//     pub mm_endpoint: String,
//     pub session_id: String,
// }

pub struct TxConfig {
    pub gas_multiplier: Option<f64>,
    pub gas_price_multiplier: Option<f64>,
    pub rate: usize,

    pub confirmations: usize,
}

pub async fn main_loop(config: &ApplicationConfig) -> Result<()> {
    let (block_subscriber, _subscriber_handle) = instantiate_block_subscriber(
        config.logic_config.ws_endpoint.clone(),
        &config.bs_config,
    )
    .await?;

    let tx_manager = instantiate_tx_manager(
        config.logic_config.signer_http_endpoint.clone(),
        Arc::clone(&block_subscriber),
        &config.tm_config,
    )
    .await?;

    let (provider, _mock) = Provider::mocked();
    let descartesv2_contract = DescartesV2Impl::new(
        config.logic_config.descartes_contract_address,
        Arc::new(provider),
    );

    let rollups_state_fold = RollupsStateFold::new(
        config.logic_config.state_fold_grpc_endpoint.clone(),
    )
    .await?;
    // let state_fold = instantiate_state_fold(&config.into())?;

    let machine_manager = {
        let mm_config = rollup_mm::Config::new_with_default(
            config.logic_config.mm_endpoint.clone(),
            config.logic_config.session_id.clone(),
        );

        rollup_mm::MachineManager::new(mm_config).await?
    };

    let mut subscription = block_subscriber
        .subscribe()
        .await
        .ok_or(EmptySubscription {}.build())?;

    let tx_config = (&config.logic_config).into();

    loop {
        // TODO: change to n blocks in the past.
        match subscription.recv().await {
            Ok(block) => {
                let state = rollups_state_fold
                    .get_state(
                        &block.hash,
                        &(
                            config.logic_config.initial_epoch,
                            config.logic_config.descartes_contract_address,
                        ),
                    )
                    .await?;

                react(
                    &config.logic_config.sender,
                    &tx_config,
                    state,
                    &tx_manager,
                    &descartesv2_contract,
                    &machine_manager,
                )
                .await?;
            }

            Err(e) => return Err(Error::SubscriberReceiveError { source: e }),
        }
    }
}

async fn react<MM: MachineInterface + Sync>(
    sender: &Address,
    config: &TxConfig,
    state: DescartesV2State,
    tx_manager: &DescartesTxManager,
    descartesv2_contract: &DescartesV2Impl<Provider<MockProvider>>,
    machine_manager: &MM,
) -> Result<()> {
    let state = state;

    let mm_epoch_status = machine_manager.get_current_epoch_status().await?;

    let (should_continue, mm_epoch_status) =
        enqueue_inputs_of_finalized_epochs(
            &state,
            mm_epoch_status,
            machine_manager,
        )
        .await?;

    if !should_continue {
        return Ok(());
    }

    match state.current_phase {
        PhaseState::InputAccumulation {} => {
            // Discover latest MM accumulating input index
            // Enqueue diff one by one
            enqueue_remaning_inputs(
                &mm_epoch_status,
                &state.current_epoch.inputs.inputs,
                machine_manager,
            )
            .await?;

            // React idle.
            return Ok(());
        }

        PhaseState::EpochSealedAwaitingFirstClaim { sealed_epoch } => {
            // On EpochSealedAwaitingFirstClaim we have two unfinalized epochs:
            // sealed and accumulating.

            // If MM is on sealed epoch, discover latest MM input index.
            // enqueue remaining inputs and SessionFinishEpochRequest.
            // React claim.

            // Then, enqueue accumulating inputs.

            // If MM is on accumulating epoch, get claim of previous
            // epoch (sealed) and
            // React claim
            let sealed_epoch_number = state.finalized_epochs.next_epoch();
            if mm_epoch_status.epoch_number == sealed_epoch_number {
                let all_inputs_processed = update_sealed_epoch(
                    &sealed_epoch.inputs.inputs,
                    &mm_epoch_status,
                    machine_manager,
                )
                .await?;

                if !all_inputs_processed {
                    // React Idle
                    return Ok(());
                }
            }

            // Enqueue accumulating epoch.
            enqueue_remaning_inputs(
                &mm_epoch_status,
                &state.current_epoch.inputs.inputs,
                machine_manager,
            )
            .await?;

            let claim =
                machine_manager.get_epoch_claim(sealed_epoch_number).await?;
            send_claim_tx(
                sender.clone(),
                claim,
                sealed_epoch_number,
                config,
                tx_manager,
                descartesv2_contract,
            )
            .await;

            return Ok(());
        }

        PhaseState::AwaitingConsensusNoConflict { claimed_epoch }
        | PhaseState::AwaitingConsensusAfterConflict {
            claimed_epoch, ..
        } => {
            let sealed_epoch_number = state.finalized_epochs.next_epoch();
            if mm_epoch_status.epoch_number == sealed_epoch_number {
                let all_inputs_processed = update_sealed_epoch(
                    &claimed_epoch.inputs.inputs,
                    &mm_epoch_status,
                    machine_manager,
                )
                .await?;

                if !all_inputs_processed {
                    // React Idle
                    return Ok(());
                }
            }

            // Enqueue accumulating epoch.
            enqueue_remaning_inputs(
                &mm_epoch_status,
                &state.current_epoch.inputs.inputs,
                machine_manager,
            )
            .await?;

            let sender_claim = claimed_epoch.claims.get_sender_claim(sender);
            if sender_claim.is_none() {
                let claim = machine_manager
                    .get_epoch_claim(sealed_epoch_number)
                    .await?;

                send_claim_tx(
                    sender.clone(),
                    claim,
                    sealed_epoch_number,
                    config,
                    tx_manager,
                    descartesv2_contract,
                )
                .await;

                return Ok(());
            }

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
            let sealed_epoch_number = state.finalized_epochs.next_epoch();
            if mm_epoch_status.epoch_number == sealed_epoch_number {
                let all_inputs_processed = update_sealed_epoch(
                    &claimed_epoch.inputs.inputs,
                    &mm_epoch_status,
                    machine_manager,
                )
                .await?;

                if !all_inputs_processed {
                    // React Idle
                    return Ok(());
                }
            }

            // Enqueue accumulating epoch.
            enqueue_remaning_inputs(
                &mm_epoch_status,
                &state.current_epoch.inputs.inputs,
                machine_manager,
            )
            .await?;

            let sender_claim = claimed_epoch.claims.get_sender_claim(sender);
            if sender_claim.is_none() {
                let claim = machine_manager
                    .get_epoch_claim(sealed_epoch_number)
                    .await?;

                send_claim_tx(
                    sender.clone(),
                    claim,
                    sealed_epoch_number,
                    config,
                    tx_manager,
                    descartesv2_contract,
                )
                .await;

                return Ok(());
            } else {
                send_finalize_tx(
                    sender.clone(),
                    sealed_epoch_number,
                    config,
                    tx_manager,
                    descartesv2_contract,
                )
                .await;

                return Ok(());
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
    todo!()
}

/// Returns true if react can continue, false otherwise, as well as the new
/// `mm_epoch_status`.
#[async_recursion]
async fn enqueue_inputs_of_finalized_epochs<MM: MachineInterface + Sync>(
    state: &DescartesV2State,
    mm_epoch_status: EpochStatus,
    machine_manager: &MM,
) -> Result<(bool, EpochStatus)> {
    // Checking if there are finalized_epochs beyond the machine manager.
    // TODO: comment on index compare.
    if mm_epoch_status.epoch_number >= state.finalized_epochs.next_epoch() {
        return Ok((true, mm_epoch_status));
    }

    let inputs = state
        .finalized_epochs
        .get_epoch(mm_epoch_status.epoch_number.as_usize())
        .expect("We should have more `finalized_epochs` than machine manager")
        .inputs;

    if mm_epoch_status.processed_input_count == inputs.inputs.len() {
        assert_eq!(
            mm_epoch_status.pending_input_count, 0,
            "Pending input count should be zero"
        );

        // Call finish
        machine_manager
            .finish_epoch(
                mm_epoch_status.epoch_number,
                mm_epoch_status.processed_input_count.into(),
            )
            .await?;

        let mm_epoch_status =
            machine_manager.get_current_epoch_status().await?;

        // recursively call enqueue_inputs_of_finalized_epochs
        return enqueue_inputs_of_finalized_epochs(
            &state,
            mm_epoch_status,
            machine_manager,
        )
        .await;
    }

    enqueue_remaning_inputs(&mm_epoch_status, &inputs.inputs, machine_manager)
        .await?;

    Ok((false, mm_epoch_status))
}

async fn enqueue_remaning_inputs<MM: MachineInterface>(
    mm_epoch_status: &EpochStatus,
    inputs: &Vector<Input>,
    machine_manager: &MM,
) -> Result<bool> {
    if mm_epoch_status.processed_input_count == inputs.len() {
        return Ok(true);
    }

    let inputs_sent_count = mm_epoch_status.processed_input_count
        + mm_epoch_status.pending_input_count;

    let input_slice = inputs.clone().slice(inputs_sent_count..);
    machine_manager
        .enqueue_inputs(
            mm_epoch_status.epoch_number,
            inputs_sent_count.into(),
            input_slice,
        )
        .await?;

    Ok(false)
}

async fn update_sealed_epoch<MM: MachineInterface>(
    sealed_inputs: &Vector<Input>,
    mm_epoch_status: &EpochStatus,
    machine_manager: &MM,
) -> Result<bool> {
    let all_inputs_processed = enqueue_remaning_inputs(
        &mm_epoch_status,
        &sealed_inputs,
        machine_manager,
    )
    .await?;

    if !all_inputs_processed {
        return Ok(false);
    }

    machine_manager
        .finish_epoch(
            mm_epoch_status.epoch_number,
            mm_epoch_status.processed_input_count.into(),
        )
        .await?;

    Ok(true)
}

async fn send_claim_tx(
    sender: Address,
    claim: H256,
    epoch_number: U256,
    config: &TxConfig,
    tx_manager: &DescartesTxManager,
    descartesv2_contract: &DescartesV2Impl<Provider<MockProvider>>,
) {
    let claim_tx = descartesv2_contract
        .claim(claim.to_fixed_bytes())
        .from(sender);

    let label = format!("claim_for_epoch:{}", epoch_number);

    tx_manager
        .send_transaction(
            label,
            claim_tx,
            ResubmitStrategy {
                gas_multiplier: config.gas_multiplier,
                gas_price_multiplier: config.gas_price_multiplier,
                rate: config.rate,
            },
            config.confirmations,
        )
        .await
        .expect("Transaction conversion should never fail");
}

async fn send_finalize_tx(
    sender: Address,
    epoch_number: U256,
    config: &TxConfig,
    tx_manager: &DescartesTxManager,
    descartesv2_contract: &DescartesV2Impl<Provider<MockProvider>>,
) {
    let finalize_tx = descartesv2_contract.finalize_epoch().from(sender);

    let label = format!("finalize_epoch:{}", epoch_number);

    tx_manager
        .send_transaction(
            label,
            finalize_tx,
            ResubmitStrategy {
                gas_multiplier: config.gas_multiplier,
                gas_price_multiplier: config.gas_price_multiplier,
                rate: config.rate,
            },
            config.confirmations,
        )
        .await
        .expect("Transaction conversion should never fail");
}

impl From<&LogicConfig> for TxConfig {
    fn from(config: &LogicConfig) -> Self {
        config.clone().into()
    }
}

impl From<LogicConfig> for TxConfig {
    fn from(config: LogicConfig) -> Self {
        Self {
            gas_multiplier: config.gas_multiplier,
            gas_price_multiplier: config.gas_price_multiplier,
            rate: config.rate,
            confirmations: config.confirmations,
        }
    }
}
