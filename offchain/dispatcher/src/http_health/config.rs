use structopt::StructOpt;

#[derive(StructOpt, Clone)]
#[structopt(
    name = "hc_config",
    about = "Configuration for rollups dispatcher health check"
)]
pub struct HealthCheckEnvCLIConfig {
    /// Host address of health check
    #[structopt(long, env)]
    pub hc_host_address: Option<String>,

    /// Port of health check
    #[structopt(long, env)]
    pub hc_port: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct HealthCheckConfig {
    pub host_address: String,
    pub port: u16,
}

const DEFAULT_HOST_ADDRESS: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 80;

impl HealthCheckConfig {
    pub fn initialize_from_args() -> Self {
        let env_cli_config = HealthCheckEnvCLIConfig::from_args();
        Self::initialize(env_cli_config)
    }

    pub fn initialize(env_cli_config: HealthCheckEnvCLIConfig) -> Self {
        let host_address = env_cli_config
            .hc_host_address
            .unwrap_or(DEFAULT_HOST_ADDRESS.to_owned());

        let port = env_cli_config.hc_port.unwrap_or(DEFAULT_PORT);

        Self { host_address, port }
    }
}
