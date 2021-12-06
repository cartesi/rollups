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

    let _state_res = tokio::try_join!(
        state.poll(&initial_state, poll_interval),
    )?;

    Ok(())
}
