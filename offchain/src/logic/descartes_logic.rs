use super::instantiate_state_fold::{self, instantiate_state_fold};
use super::instantiate_tx_manager::{
    self, instantiate_tx_manager, DescartesTxManager,
};

use crate::error::*;
use crate::fold::types::*;
use crate::machine::{EpochStatus, MachineInterface, MockMachine};

use dispatcher::block_subscriber::{
    BlockSubscriber, BlockSubscriberHandle, NewBlockSubscriber,
};
use dispatcher::state_fold::types::BlockState;

use async_recursion::async_recursion;
use ethers::core::types::{Address, U256, U64};
use im::Vector;
use snafu::ResultExt;

pub struct Config {
    pub safety_margin: usize,
    pub input_contract_address: Address, // TODO: read from contract.
    pub descartes_contract_address: Address,

    pub provider_http_url: String,
    pub genesis_block: U64,
    pub query_limit_error_codes: Vec<i32>,
    pub concurrent_events_fetch: usize,

    pub http_endpoint: String,
    pub ws_endpoint: String,
    pub max_retries: usize,
    pub max_delay: std::time::Duration,

    pub call_timeout: std::time::Duration,
    pub subscriber_timeout: std::time::Duration,

    pub initial_epoch: U256,
}

async fn main_loop(config: &Config, sender: Address) -> Result<()> {
    let (subscriber_handle, block_subscriber, tx_manager) =
        instantiate_tx_manager(&config.into()).await?;
    let state_fold = instantiate_state_fold(&config.into())?;

    // Start MachineManager session request

    let mut subscription = block_subscriber
        .subscribe()
        .await
        .ok_or(EmptySubscription {}.build())?;

    loop {
        // TODO: change to n blocks in the past.
        match subscription.recv().await {
            Ok(block) => {
                let state = state_fold
                    .get_state_for_block(&config.initial_epoch, block.hash)
                    .await
                    .context(StateFoldError {})?;

                react(&sender, state, &tx_manager, &MockMachine {}).await;
            }

            Err(e) => return Err(Error::SubscriberReceiveError { source: e }),
        }
    }
}

async fn react<M: MachineInterface + Sync>(
    sender: &Address,
    state: BlockState<DescartesV2State>,
    tx_manager: &DescartesTxManager,
    machine_manager: &M,
) -> Result<()> {
    let state = state.state;

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
                    sealed_epoch_number,
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

            // TODO: Send claim
        }

        // F: I actually have the feeling that AwaitingConsensusNoConflict
        //  and AwaitingConsensusAfterConflict should be unified. The decision
        //  making for them is the same, they only differ for the delegate
        //  to check if the Consensus has timedout or not.
        PhaseState::AwaitingConsensusNoConflict { claimed_epoch }
        | PhaseState::AwaitingConsensusAfterConflict {
            claimed_epoch, ..
        } => {
            let sealed_epoch_number = state.finalized_epochs.next_epoch();
            if mm_epoch_status.epoch_number == sealed_epoch_number {
                let all_inputs_processed = update_sealed_epoch(
                    sealed_epoch_number,
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

                // TODO:  Send claim
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
                    sealed_epoch_number,
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

                // TODO:  Send claim
            } else {
                // TODO: Call blockchain finalize epoch
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

        /// Unreacheable
        PhaseState::AwaitingDispute { claimed_epoch } => {}
    }
    todo!()
}

/// Returns true if react can continue, false otherwise, as well as the new
/// `mm_epoch_status`.
#[async_recursion]
async fn enqueue_inputs_of_finalized_epochs<M: MachineInterface + Sync>(
    state: &DescartesV2State,
    mm_epoch_status: EpochStatus,
    machine_manager: &M,
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

async fn enqueue_remaning_inputs<M: MachineInterface>(
    mm_epoch_status: &EpochStatus,
    inputs: &Vector<Input>,
    machine_manager: &M,
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

async fn update_sealed_epoch<M: MachineInterface>(
    sealed_epoch_number: U256,
    sealed_inputs: &Vector<Input>,
    mm_epoch_status: &EpochStatus,
    machine_manager: &M,
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

impl From<&Config> for instantiate_state_fold::Config {
    fn from(config: &Config) -> Self {
        let config = config.clone();
        Self {
            safety_margin: config.safety_margin,
            input_contract_address: config.input_contract_address,
            descartes_contract_address: config.descartes_contract_address,

            provider_http_url: config.provider_http_url.clone(),
            genesis_block: config.genesis_block,
            query_limit_error_codes: config.query_limit_error_codes.clone(),
            concurrent_events_fetch: config.concurrent_events_fetch,
        }
    }
}

impl From<&Config> for instantiate_tx_manager::Config {
    fn from(config: &Config) -> Self {
        let config = config.clone();
        Self {
            http_endpoint: config.http_endpoint.clone(),
            ws_endpoint: config.ws_endpoint.clone(),
            max_retries: config.max_retries,
            max_delay: config.max_delay,

            call_timeout: config.call_timeout,
            subscriber_timeout: config.subscriber_timeout,
        }
    }
}

/*
use super::fold::*;
use dispatcher::state_fold::Access;

use ethers::core::types::{Address, U64};
use ethers::providers::{Http, Provider};

use super::error::*;

use snafu::ResultExt;
use std::convert::TryFrom;
use std::sync::Arc;

pub struct Config {
    safety_margin: usize,
    input_contract_address: Address, // TODO: read from contract.
    descartes_contract_address: Address,

    provider_http_url: String,
    genesis_block: U64,
    query_limit_error_codes: Vec<i32>,
    concurrent_events_fetch: usize,
}

pub type DescartesAccess = Access<Provider<Http>>;

pub fn instantiate_state_fold(
    config: &Config,
) -> Result<DescartesStateFold<DescartesAccess>> {
    let access = create_access(config)?;
    let setup_config = SetupConfig::from(config);
    let state_fold = create_descartes_state_fold(access, &setup_config);
    Ok(state_fold)
}

fn create_provider(url: String) -> Result<Arc<Provider<Http>>> {
    Ok(Arc::new(
        Provider::<Http>::try_from(url).context(UrlParseError {})?,
    ))
}

fn create_access(config: &Config) -> Result<Arc<DescartesAccess>> {
    let provider = create_provider(config.provider_http_url)?;

    Ok(Arc::new(Access::new(
        provider,
        config.genesis_block,
        config.query_limit_error_codes,
        config.concurrent_events_fetch,
    )))
}

impl From<&Config> for SetupConfig {
    fn from(config: &Config) -> Self {
        let config = config.clone();
        SetupConfig {
            safety_margin: config.safety_margin,
            input_contract_address: config.input_contract_address,
            descartes_contract_address: config.descartes_contract_address,
        }
    }
}
*/
