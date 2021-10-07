use block_subscriber::config::BSConfig;
use configuration::Config;
use state_fold::config::SFConfig;
use tx_manager::config::TMConfig;

use structopt::StructOpt;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Bad configuration: {}", err))]
    BadConfiguration { err: String },
}

#[derive(StructOpt, Clone, Debug)]
pub struct DescartesCLIConfig {
    #[structopt(long, env)]
    pub descartes_config: Option<String>,
    #[structopt(flatten)]
    pub config: configuration::config::EnvCLIConfig,
    #[structopt(flatten)]
    pub bs_config: block_subscriber::config::BSEnvCLIConfig,
    #[structopt(flatten)]
    pub sf_config: state_fold::config::SFEnvCLIConfig,
    #[structopt(flatten)]
    pub tm_config: tx_manager::config::TMEnvCLIConfig,
}

#[derive(Clone, Debug)]
pub struct DescartesConfig {
    pub basic_config: Config,
    pub bs_config: BSConfig,
    pub sf_config: SFConfig,
    pub tm_config: TMConfig,
}

impl DescartesConfig {
    pub fn initialize(
        descartes_cli_config: DescartesCLIConfig,
    ) -> std::result::Result<Self, Error> {
        let basic_config = Config::initialize(descartes_cli_config.config)
            .map_err(|e| {
                BadConfiguration {
                    err: format!("Fail to initialize basic config: {}", e),
                }
                .build()
            })?;

        let bs_config = BSConfig::initialize(descartes_cli_config.bs_config)
            .map_err(|e| {
                BadConfiguration {
                    err: format!(
                        "Fail to initialize block subscriber config: {}",
                        e
                    ),
                }
                .build()
            })?;

        let sf_config = SFConfig::initialize(descartes_cli_config.sf_config)
            .map_err(|e| {
                BadConfiguration {
                    err: format!("Fail to initialize state fold config: {}", e),
                }
                .build()
            })?;

        let tm_config = TMConfig::initialize(descartes_cli_config.tm_config)
            .map_err(|e| {
                BadConfiguration {
                    err: format!("Fail to initialize tx manager config: {}", e),
                }
                .build()
            })?;

        Ok(DescartesConfig {
            basic_config,
            bs_config,
            sf_config,
            tm_config,
        })
    }
}
