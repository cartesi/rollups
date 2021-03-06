use crate::error::*;

use block_subscriber::config::BSConfig;
use state_fold::config::SFConfig;
use tx_manager::config::TMConfig;

use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
pub struct ApplicationCLIConfig {
    #[structopt(long, env)]
    pub app_config: Option<String>,
    #[structopt(flatten)]
    pub logic_config: crate::logic::config::LogicEnvCLIConfig,
    #[structopt(flatten)]
    pub sf_config: state_fold::config::SFEnvCLIConfig,
    #[structopt(flatten)]
    pub bs_config: block_subscriber::config::BSEnvCLIConfig,
    #[structopt(flatten)]
    pub tm_config: tx_manager::config::TMEnvCLIConfig,
}

#[derive(Clone, Debug)]
pub struct ApplicationConfig {
    pub logic_config: crate::logic::config::LogicConfig,
    pub sf_config: SFConfig,
    pub bs_config: BSConfig,
    pub tm_config: TMConfig,
}

impl ApplicationConfig {
    pub fn initialize() -> Result<Self> {
        let app_cli_config = ApplicationCLIConfig::from_args();

        let logic_config = crate::logic::config::LogicConfig::initialize(
            app_cli_config.logic_config,
        )
        .map_err(|e| {
            BadConfiguration {
                err: format!("Fail to initialize logic config: {}", e),
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

        let sf_config = SFConfig::initialize(app_cli_config.sf_config)
            .map_err(|e| {
                BadConfiguration {
                    err: format!("Fail to initialize state fold config: {}", e),
                }
                .build()
            })?;

        let tm_config = TMConfig::initialize(app_cli_config.tm_config)
            .map_err(|e| {
                BadConfiguration {
                    err: format!(
                        "Fail to initialize transaction manager config: {}",
                        e
                    ),
                }
                .build()
            })?;

        Ok(ApplicationConfig {
            logic_config,
            bs_config,
            sf_config,
            tm_config,
        })
    }
}
