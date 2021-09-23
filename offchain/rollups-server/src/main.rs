pub mod delegate_manager;

use state_server_grpc::{serve_delegate_manager, wait_for_signal};

use delegate_manager::RollupsDelegateManager;
use outputserver::instantiate_descartes_fold_delegate;

use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descartes_fold = instantiate_descartes_fold_delegate().unwrap();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let _ = tokio::spawn(wait_for_signal(shutdown_tx));

    serve_delegate_manager(
        "[::1]:50051",
        RollupsDelegateManager {
            fold: descartes_fold,
        },
        shutdown_rx,
    )
    .await
}
