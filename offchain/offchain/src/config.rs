use crate::error::*;

use block_subscriber::config::BSConfig;
use configuration::Config;
use tx_manager::config::TMConfig;

use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
struct ApplicationCLIConfig {
    #[structopt(long, env)]
    pub app_config: Option<String>,
    #[structopt(flatten)]
    pub config: configuration::config::EnvCLIConfig,
    #[structopt(flatten)]
    pub bs_config: block_subscriber::config::BSEnvCLIConfig,
    #[structopt(flatten)]
    pub tm_config: tx_manager::config::TMEnvCLIConfig,
}

#[derive(Clone, Debug)]
pub struct ApplicationConfig {
    pub basic_config: Config,
    pub logic_config: crate::logic::Config,
    pub bs_config: BSConfig,
    pub tm_config: TMConfig,
}

impl ApplicationConfig {
    pub fn initialize() -> Result<Self> {
        let app_cli_config = ApplicationCLIConfig::from_args();
        let basic_config =
            Config::initialize(app_cli_config.config).map_err(|e| {
                BadConfiguration {
                    err: format!("Fail to initialize basic config: {}", e),
                }
                .build()
            })?;

        let bs_config = BSConfig::initialize(app_cli_config.bs_config)
            .map_err(|e| {
                BadConfiguration {
                    err: format!(
                        "Fail to initialize block subscriber config: {}",
                        e
                    ),
                }
                .build()
            })?;

        Ok(ApplicationConfig {
            basic_config,
            bs_config,
            sf_config,
        })
    }
}
