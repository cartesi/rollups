use std::time::Duration;
use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
#[structopt(name = "broker_config", about = "Configuration for the broker")]
pub struct BrokerEnvCLIConfig {
    /// Address of the broker in the format redis://hostname:port
    #[structopt(long, env, default_value = "redis://127.0.0.1:6379")]
    redis_endpoint: String,

    /// Consume timeout when waiting for the rollups claims in ms
    #[structopt(long, env, default_value = "300000")]
    claims_consume_timeout: usize,

    /// The max elapsed time for backoff in ms
    #[structopt(long, env, default_value = "120000")]
    broker_backoff_max_elapsed_duration: u64,
}

#[derive(Clone, Debug)]
pub struct BrokerConfig {
    pub redis_endpoint: String,
    pub chain_id: u64,
    pub dapp_contract_address: [u8; 20],
    pub claims_consume_timeout: usize,
    pub backoff_max_elapsed_duration: Duration,
}

impl BrokerConfig {
    pub fn initialize(
        env_cli_config: BrokerEnvCLIConfig,
        chain_id: u64,
        dapp_contract_address: [u8; 20],
    ) -> Self {
        Self {
            redis_endpoint: env_cli_config.redis_endpoint,
            chain_id,
            dapp_contract_address,
            claims_consume_timeout: env_cli_config.claims_consume_timeout,
            backoff_max_elapsed_duration: Duration::from_millis(
                env_cli_config.broker_backoff_max_elapsed_duration,
            ),
        }
    }
}
