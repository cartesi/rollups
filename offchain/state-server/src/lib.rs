use state_fold::{Foldable, StateFoldEnvironment};
use state_fold_types::ethers::providers::{Provider, Ws};
use state_server_lib::{
    config,
    grpc_server::StateServer,
    utils::{start_server, wait_for_signal},
};

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::oneshot;

#[tracing::instrument(level = "trace")]
pub async fn run_server<F: Foldable<UserData = ()> + 'static>(
    config: config::StateServerConfig,
) -> Result<()>
where
    <F as Foldable>::InitialState: serde::de::DeserializeOwned,
    F: serde::ser::Serialize,
{
    tracing::trace!("Starting rollups state-server with config `{:?}`", config);

    let provider = create_provider(&config).await?;
    let block_subscriber =
        create_block_subscriber(&config, Arc::clone(&provider)).await?;
    let env = create_env(
        &config,
        Arc::clone(&provider),
        Arc::clone(&block_subscriber.block_archive),
    )?;

    let server = StateServer::<_, _, F>::new(block_subscriber, env);

    let server_address = config.server_address;
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let (signal_handle, server_handle) = tokio::join!(
        tokio::spawn(async { wait_for_signal(shutdown_tx).await }),
        tokio::spawn(async move {
            start_server(server_address, server, shutdown_rx).await
        })
    );

    signal_handle?;
    server_handle??;

    Ok(())
}

type ServerProvider = Provider<Ws>;

async fn create_provider(
    config: &config::StateServerConfig,
) -> Result<Arc<ServerProvider>> {
    let endpoint = config.block_history.ws_endpoint.clone();
    let provider = Provider::connect(endpoint).await?;
    Ok(Arc::new(provider))
}

fn create_env(
    config: &config::StateServerConfig,
    provider: Arc<ServerProvider>,
    block_archive: Arc<block_history::BlockArchive<ServerProvider>>,
) -> Result<Arc<StateFoldEnvironment<ServerProvider, ()>>> {
    let env = StateFoldEnvironment::new(
        provider,
        Some(block_archive),
        config.state_fold.safety_margin,
        config.state_fold.genesis_block,
        config.state_fold.query_limit_error_codes.clone(),
        config.state_fold.concurrent_events_fetch,
        10000,
        (),
    );

    Ok(Arc::new(env))
}

async fn create_block_subscriber(
    config: &config::StateServerConfig,
    provider: Arc<ServerProvider>,
) -> Result<Arc<block_history::BlockSubscriber<ServerProvider>>> {
    let block_subscriber = block_history::BlockSubscriber::start(
        Arc::clone(&provider),
        config.block_history.block_timeout,
        config.block_history.max_depth,
    )
    .await?;

    Ok(Arc::new(block_subscriber))
}
