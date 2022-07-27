use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config =
        state_server_lib::config::StateServerConfig::initialize_from_args()?;

    state_server::run_server::<types::fee_manager::FeeManagerState>(config)
        .await?;

    Ok(())
}
