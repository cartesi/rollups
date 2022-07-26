use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = dispatcher::config::DispatcherConfig::initialize_from_args()?;

    dispatcher::main_loop::run(config).await?;
    Ok(())
}
