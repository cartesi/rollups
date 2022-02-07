use offchain_core::ethers;

use super::instantiate_block_subscriber::instantiate_block_subscriber;
use super::instantiate_tx_manager::{instantiate_tx_manager, RollupsTxManager};

use super::config::LogicConfig;
use crate::config::ApplicationConfig;
use crate::contracts::rollups_facet::RollupsFacet;
use crate::error::*;
use crate::fold::types::*;
use crate::machine::{rollup_server, EpochStatus, MachineInterface};
use crate::rollups_state_fold::RollupsStateFold;

use block_subscriber::NewBlockSubscriber;
use tx_manager::types::ResubmitStrategy;

use async_recursion::async_recursion;
use ethers::core::types::{Address, H256, U256};
use ethers::providers::{MockProvider, Provider};
use im::Vector;
use std::sync::Arc;

use tracing::{error, info, instrument};

#[derive(Debug)]
pub struct TxConfig {
    pub gas_multiplier: Option<f64>,
    pub gas_price_multiplier: Option<f64>,
    pub rate: usize,

    pub confirmations: usize,
}

#[instrument(skip_all)]
pub async fn main_loop(config: &ApplicationConfig) -> Result<()> {
    info!(
        "Creating block subscriber with endpoint `{}`",
        &config.logic_config.ws_endpoint
    );

    let (block_subscriber, _subscriber_handle) = instantiate_block_subscriber(
        config.logic_config.ws_endpoint.clone(),
        &config.bs_config,
    )
    .await?;

    info!(
        "Creating transaction manager with endpoint `{}`",
        &config.logic_config.provider_http_endpoint
    );

    let (tx_manager, sender) = instantiate_tx_manager(
        config.logic_config.provider_http_endpoint.clone(),
        config.logic_config.mnemonic.clone(),
        config.logic_config.chain_id,
        Arc::clone(&block_subscriber),
        &config.tm_config,
    )
    .await?;

    info!("Dispatcher running with validator address `{}`", sender);

    info!(
        "Instantiating rollups facet at address `{}`",
        config.logic_config.dapp_contract_address,
    );

    let (provider, _mock) = Provider::mocked();
    let rollups_facet = RollupsFacet::new(
        config.logic_config.dapp_contract_address,
        Arc::new(provider),
    );

    info!(
        "Creating state-fold server gRPC connection at endpoint `{}`",
        &config.logic_config.state_fold_grpc_endpoint
    );

    let rollups_state_fold = RollupsStateFold::new(
        config.logic_config.state_fold_grpc_endpoint.clone(),
    )
    .await?;

    // For a local statefold, use the following:
    // let state_fold = instantiate_state_fold(&config.into())?;

    info!(
        "Creating rollup machine manager gRPC connection at endpoint `{}` with session id `{}`",
        &config.logic_config.mm_endpoint,
        &config.logic_config.session_id,
    );

    let machine_manager = {
        let mm_config = rollup_server::Config::new_with_default(
            config.logic_config.mm_endpoint.clone(),
            config.logic_config.session_id.clone(),
        );

        rollup_server::MachineManager::new(mm_config).await?
    };

    info!("Starting block-subscriber subscription");

    let mut subscription = block_subscriber
        .subscribe()
        .await
        .ok_or(EmptySubscription {}.build())?;

    let tx_config = (&config.logic_config).into();

    info!("Entering main loop...");

    loop {
        // TODO: change to n blocks in the past.
        //
        info!("Awaiting new blocks...");

        match subscription.recv().await {
            Ok(block) => {
                info!("Dispatcher received new block `{:#?}`", &block);

                info!("Querying state-fold server latest state");

                let state = rollups_state_fold
                    .get_state(
                        &block.hash,
                        &(
                            config.logic_config.initial_epoch,
                            config.logic_config.dapp_contract_address,
                        ),
                    )
                    .await?;

                info!("Reacting on state `{:#?}`", &state);

                react(
                    &sender,
                    &tx_config,
                    state,
                    &tx_manager,
                    &rollups_facet,
                    &machine_manager,
                )
                .await?;

                info!("Reacting done");
            }

            Err(e) => {
                error!("Block subscription failed! Error: {}", e);
                return Err(Error::SubscriberReceiveError { source: e });
            }
        }
    }
}

#[instrument(skip_all)]
async fn react<MM: MachineInterface + Sync>(
    sender: &Address,
    config: &TxConfig,
    state: RollupsState,
    tx_manager: &RollupsTxManager,
    rollups_facet: &RollupsFacet<Provider<MockProvider>>,
    machine_manager: &MM,
) -> Result<()> {
    info!("Querying machine manager for current epoch status");
    let mm_epoch_status = machine_manager.get_current_epoch_status().await?;

    info!("Current epoch status: {:#?}", mm_epoch_status);

    let (should_continue, mm_epoch_status) =
        enqueue_inputs_of_finalized_epochs(
            &state,
            mm_epoch_status,
            machine_manager,
        )
        .await?;

    if !should_continue {
        info!("Machine manager finalization pending; exiting `react`");
        return Ok(());
    }

    match &state.current_phase {
        PhaseState::InputAccumulation {} => {
            info!("InputAccumulation phase; enqueueing inputs");

            // Discover latest MM accumulating input index
            // Enqueue diff one by one
            enqueue_inputs_for_accumulating_epoch(
                &mm_epoch_status,
                &state,
                machine_manager,
            )
            .await?;

            // React idle.
            return Ok(());
        }

        PhaseState::EpochSealedAwaitingFirstClaim { sealed_epoch } => {
            info!("EpochSealedAwaitingFirstClaim phase");
            info!("Sealed epoch: {:#?}", sealed_epoch);

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
                info!("Machine manager is on sealed epoch");

                let all_inputs_processed = update_sealed_epoch(
                    &sealed_epoch.inputs.inputs,
                    &mm_epoch_status,
                    machine_manager,
                )
                .await?;

                if !all_inputs_processed {
                    // React Idle
                    info!("Machine manager has unprocessed inputs; exiting `react`");
                    return Ok(());
                }

                machine_manager.get_current_epoch_status().await?
            } else {
                mm_epoch_status
            };

            // Enqueue accumulating epoch.
            enqueue_inputs_for_accumulating_epoch(
                &mm_epoch_status,
                &state,
                machine_manager,
            )
            .await?;

            info!("Querying machine manager for sealed epoch claim");

            let claim =
                machine_manager.get_epoch_claim(sealed_epoch_number).await?;

            info!("Machine manager returned claim `{}`; sending transaction for first claim...", claim);

            send_claim_tx(
                sender.clone(),
                claim,
                sealed_epoch_number,
                config,
                tx_manager,
                rollups_facet,
            )
            .await;

            return Ok(());
        }

        PhaseState::AwaitingConsensusNoConflict { claimed_epoch }
        | PhaseState::AwaitingConsensusAfterConflict {
            claimed_epoch, ..
        } => {
            info!("AwaitingConsensusNoConflict or AwaitingConsensusAfterConflict  phase");
            info!("Claimed epoch: {:#?}", claimed_epoch);

            let sealed_epoch_number = state.finalized_epochs.next_epoch();
            let mm_epoch_status = if mm_epoch_status.epoch_number
                == sealed_epoch_number
            {
                info!("Machine manager is on sealed epoch");

                let all_inputs_processed = update_sealed_epoch(
                    &claimed_epoch.inputs.inputs,
                    &mm_epoch_status,
                    machine_manager,
                )
                .await?;

                if !all_inputs_processed {
                    // React Idle
                    info!("Machine manager has unprocessed inputs; exiting `react`");
                    return Ok(());
                }

                machine_manager.get_current_epoch_status().await?
            } else {
                mm_epoch_status
            };

            // Enqueue accumulating epoch.
            enqueue_inputs_for_accumulating_epoch(
                &mm_epoch_status,
                &state,
                machine_manager,
            )
            .await?;

            let sender_claim = claimed_epoch.claims.get_sender_claim(sender);
            if sender_claim.is_none() {
                info!("Validator has sent no claims yet; getting claim from machine manager...");

                let claim = machine_manager
                    .get_epoch_claim(sealed_epoch_number)
                    .await?;

                info!("Machine manager returned claim `{}`; sending transaction for claim...", claim);

                send_claim_tx(
                    sender.clone(),
                    claim,
                    sealed_epoch_number,
                    config,
                    tx_manager,
                    rollups_facet,
                )
                .await;

                return Ok(());
            }

            info!("Validator has sent claim for this epoch");

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
            info!("ConsensusTimeout phase");
            info!("Claimed epoch: {:#?}", claimed_epoch);

            let sealed_epoch_number = state.finalized_epochs.next_epoch();
            let mm_epoch_status = if mm_epoch_status.epoch_number
                == sealed_epoch_number
            {
                info!("Machine manager is on sealed epoch");

                let all_inputs_processed = update_sealed_epoch(
                    &claimed_epoch.inputs.inputs,
                    &mm_epoch_status,
                    machine_manager,
                )
                .await?;

                if !all_inputs_processed {
                    // React Idle
                    info!("Machine manager has unprocessed inputs; exiting `react`");
                    return Ok(());
                }
                machine_manager.get_current_epoch_status().await?
            } else {
                mm_epoch_status
            };

            // Enqueue accumulating epoch.
            enqueue_inputs_for_accumulating_epoch(
                &mm_epoch_status,
                &state,
                machine_manager,
            )
            .await?;

            let sender_claim = claimed_epoch.claims.get_sender_claim(sender);
            if sender_claim.is_none() {
                info!("Validator has sent no claims yet; getting claim from machine manager...");

                let claim = machine_manager
                    .get_epoch_claim(sealed_epoch_number)
                    .await?;

                info!("Machine manager returned claim `{}`; sending transaction claim...", claim);

                send_claim_tx(
                    sender.clone(),
                    claim,
                    sealed_epoch_number,
                    config,
                    tx_manager,
                    rollups_facet,
                )
                .await;

                info!("Claim sent; exiting `react`");

                return Ok(());
            } else {
                info!("Validator has sent a claim; sending finalize epoch transaction");

                send_finalize_tx(
                    sender.clone(),
                    sealed_epoch_number,
                    config,
                    tx_manager,
                    rollups_facet,
                )
                .await;

                info!("Finalize transaction sent; exiting `react`");

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

    Ok(())
}

/// Returns true if react can continue, false otherwise, as well as the new
/// `mm_epoch_status`.
#[async_recursion]
#[instrument(skip_all)]
async fn enqueue_inputs_of_finalized_epochs<MM: MachineInterface + Sync>(
    state: &RollupsState,
    mm_epoch_status: EpochStatus,
    machine_manager: &MM,
) -> Result<(bool, EpochStatus)> {
    // Checking if there are finalized_epochs beyond the machine manager.
    // TODO: comment on index compare.
    if mm_epoch_status.epoch_number >= state.finalized_epochs.next_epoch() {
        info!(
            "Machine manager epoch number `{}` ahead of finalized_epochs `{}`",
            mm_epoch_status.epoch_number,
            state.finalized_epochs.next_epoch()
        );
        return Ok((true, mm_epoch_status));
    }

    let inputs = state
        .finalized_epochs
        .get_epoch(mm_epoch_status.epoch_number.as_usize())
        .expect("We should have more `finalized_epochs` than machine manager")
        .inputs;

    info!(
        "Got `{}` inputs for epoch `{}`; machine manager in epoch `{}`",
        inputs.inputs.len(),
        inputs.epoch_number,
        mm_epoch_status.epoch_number
    );

    if mm_epoch_status.processed_input_count == inputs.inputs.len() {
        info!("Machine manager has processed all inputs of epoch");
        assert_eq!(
            mm_epoch_status.pending_input_count, 0,
            "Pending input count should be zero"
        );

        // Call finish
        info!(
            "Finishing epoch `{}` on machine manager with `{}` inputs",
            inputs.epoch_number,
            inputs.inputs.len()
        );
        machine_manager
            .finish_epoch(
                mm_epoch_status.epoch_number,
                mm_epoch_status.processed_input_count.into(),
            )
            .await?;

        info!("Querying machine manager current epoch status");
        let mm_epoch_status =
            machine_manager.get_current_epoch_status().await?;

        info!(
            "New machine manager epoch `{}`",
            mm_epoch_status.epoch_number
        );

        // recursively call enqueue_inputs_of_finalized_epochs
        info!("Recursively call `enqueue_inputs_of_finalized_epochs` for new status");
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

#[instrument(skip_all)]
async fn enqueue_inputs_for_accumulating_epoch<MM: MachineInterface>(
    mm_epoch_status: &EpochStatus,
    state: &RollupsState,
    machine_manager: &MM,
) -> Result<bool> {
    info!(
        "Enqueue inputs for accumulating epoch `{}`; machine manager in epoch `{}`",
        state.current_epoch.epoch_number,
        mm_epoch_status.epoch_number
    );

    // Enqueue accumulating epoch.
    enqueue_remaning_inputs(
        &mm_epoch_status,
        &state.current_epoch.inputs.inputs,
        machine_manager,
    )
    .await
}

#[instrument(skip_all)]
async fn enqueue_remaning_inputs<MM: MachineInterface>(
    mm_epoch_status: &EpochStatus,
    inputs: &Vector<Input>,
    machine_manager: &MM,
) -> Result<bool> {
    if mm_epoch_status.processed_input_count == inputs.len() {
        info!("Machine manager has processed all current inputs");
        return Ok(true);
    }

    let inputs_sent_count = mm_epoch_status.processed_input_count
        + mm_epoch_status.pending_input_count;

    info!("Number of inputs in machine manager: {}", inputs_sent_count);

    let input_slice = inputs.clone().slice(inputs_sent_count..);

    info!(
        "Machine manager enqueueing inputs from `{}` to `{}`",
        inputs_sent_count,
        inputs.len()
    );

    machine_manager
        .enqueue_inputs(
            mm_epoch_status.epoch_number,
            inputs_sent_count.into(),
            input_slice,
        )
        .await?;

    Ok(false)
}

#[instrument(skip_all)]
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
        info!("There are still inputs to be processed");
        return Ok(false);
    }

    info!(
        "Finishing epoch `{}` on machine manager with `{}` inputs",
        mm_epoch_status.epoch_number, mm_epoch_status.processed_input_count
    );

    machine_manager
        .finish_epoch(
            mm_epoch_status.epoch_number,
            mm_epoch_status.processed_input_count.into(),
        )
        .await?;

    Ok(true)
}

#[instrument(skip_all)]
async fn send_claim_tx(
    sender: Address,
    claim: H256,
    epoch_number: U256,
    config: &TxConfig,
    tx_manager: &RollupsTxManager,
    rollups_facet: &RollupsFacet<Provider<MockProvider>>,
) {
    let claim_tx = rollups_facet.claim(claim.to_fixed_bytes()).from(sender);
    info!("Built claim transaction: `{:?}`", claim_tx);

    let label = format!("claim_for_epoch:{}", epoch_number);
    info!("Claim transaction label: {}", &label);

    info!("Sending claim transaction");
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

#[instrument(skip_all)]
async fn send_finalize_tx(
    sender: Address,
    epoch_number: U256,
    config: &TxConfig,
    tx_manager: &RollupsTxManager,
    rollups_facet: &RollupsFacet<Provider<MockProvider>>,
) {
    let finalize_tx = rollups_facet.finalize_epoch().from(sender);
    info!("Built finalize transaction: `{:?}`", finalize_tx);

    let label = format!("finalize_epoch:{}", epoch_number);
    info!("Finalize transaction label: {}", &label);

    info!("Sending finalize");
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
