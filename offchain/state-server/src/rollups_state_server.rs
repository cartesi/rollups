use anyhow::Result;

use types::foldables::authority::rollups::RollupsState;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config =
        state_server_lib::config::StateServerConfig::initialize_from_args()?;

    state_server::run_server::<RollupsState>(config).await?;

    Ok(())
}
