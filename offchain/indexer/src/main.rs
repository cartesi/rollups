use ethers::core::types::U256;
use indexer::config::IndexerConfig;
use indexer::error::Error::BadConfiguration;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let indexer_config = IndexerConfig::initialize().map_err(|e| {
        BadConfiguration {
            err: format!("Fail to initialize indexer config: {}", e),
        }
    })?;

    println!("Starting indexer with config `{:?}`", &indexer_config);

    let dapp_contract_address = indexer_config.dapp_contract_address;
    let _poll_interval = std::time::Duration::from_secs(indexer_config.interval);
    let _initial_state = (
        U256::from(indexer_config.initial_epoch),
        dapp_contract_address,
    );

    let _postgres_endpoint = indexer_config.postgres_endpoint;
    let _rollup_machine_manager_endpoint = indexer_config.mm_endpoint;
    let _session_id = indexer_config.session_id;

    println!("Indexer started!!!");

    Ok(())
}
