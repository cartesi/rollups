use ethers::core::types::U256;

use indexer::config::PollingConfig;
use indexer::create_pool;
use indexer::error::BadConfiguration;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let polling_config = PollingConfig::initialize()
        .map_err(|e| {
        BadConfiguration {
            err: format!("Fail to intialize polling config: {}", e),
        }
        .build()
    })?;

    let rollups_address = polling_config.rollups_contract_address;
    let poll_interval = std::time::Duration::from_secs(polling_config.interval);
    let initial_state =
        (U256::from(polling_config.initial_epoch), rollups_address);

    let postgres_endpoint = polling_config.postgres_endpoint;
    let pool = create_pool(postgres_endpoint.clone())?;

    let state = indexer::state::Poller::new(
        polling_config.state_server_endpoint,
        pool.clone(),
    )
    .await?;

    let rollup_machine_manager_endpoint = polling_config.mm_endpoint;
    let machine_manager_poller = indexer::machine_manager::Poller::new(
        rollup_machine_manager_endpoint,
        pool,
    )
    .await?;

    let session_id = polling_config.session_id;
    let (_state_res, _version_res, _status_res, _session_status_res) = tokio::try_join!(
        state.poll(&initial_state, poll_interval),
        machine_manager_poller.clone().poll_version(poll_interval),
        machine_manager_poller.clone().poll_status(poll_interval),
        machine_manager_poller
            .clone()
            .poll_session_status(session_id.clone(), poll_interval),
    )?;

    Ok(())
}
