use snafu::{ResultExt, Snafu};
use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
#[structopt(name = "mm_config", about = "Configuration for server manager")]
pub struct MMEnvCLIConfig {
    /// URL of server manager grpc
    #[structopt(long)]
    pub mm_endpoint: Option<String>,

    /// Default session ID
    #[structopt(long)]
    pub mm_session_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MMConfig {
    pub endpoint: String,
    pub session_id: String,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Configuration missing server manager endpoint"))]
    MissingEndpoint {},

    #[snafu(display("Configuration missing server manager session ID"))]
    MissingSessionId {},
}

pub type Result<T> = std::result::Result<T, Error>;

impl MMConfig {
    pub fn initialize_from_args() -> Result<Self> {
        let env_cli_config = MMEnvCLIConfig::from_args();
        Self::initialize(env_cli_config)
    }

    pub fn initialize(env_cli_config: MMEnvCLIConfig) -> Result<Self> {
        let endpoint = env_cli_config
            .mm_endpoint
            .ok_or(snafu::NoneError)
            .context(MissingEndpointSnafu)?;

        let session_id = env_cli_config
            .mm_session_id
            .ok_or(snafu::NoneError)
            .context(MissingSessionIdSnafu)?;

        Ok(MMConfig {
            endpoint,
            session_id,
        })
    }
}
