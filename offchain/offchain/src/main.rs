use offchain::config::ApplicationConfig;
use offchain::error::*;
use offchain::logic::rollups_logic::main_loop;

use tracing::{info, instrument, trace};
use tracing_subscriber;

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    trace!("Initializing application config");
    let config = ApplicationConfig::initialize()?;

    info!("Starting rollups dispatcher with config `{:#?}`", &config);
    main_loop(&config).await
}
