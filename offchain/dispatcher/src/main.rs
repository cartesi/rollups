use anyhow::Result;

// NOTE: doesn't support History upgradability.
// NOTE: doesn't support changing epoch_duration in the middle of things.
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = dispatcher::config::DispatcherConfig::initialize_from_args()?;
    let hc_config = config.hc_config.clone();

    let health_handle = tokio::spawn(async move {
        dispatcher::http_health::start_health_check(
            hc_config.host_address.as_ref(),
            hc_config.port,
        )
        .await
    });

    let dispatcher_handle =
        tokio::spawn(async move { dispatcher::main_loop::run(config).await });

    tokio::select! {
        ret = health_handle => {
            tracing::error!("HTTP health-check stopped: {:?}", ret);
            ret??;
        }

        ret = dispatcher_handle => {
            tracing::error!("Dispatcher stopped: {:?}", ret);
            ret??;
        }
    }

    Ok(())
}
