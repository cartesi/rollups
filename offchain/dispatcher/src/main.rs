use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = dispatcher::config::DispatcherConfig::initialize_from_args()?;

    dispatcher::main_loop::run(config).await?;
    Ok(())
}
