use state_server_grpc::{serve_delegate_manager, wait_for_signal};

use outputserver::delegate_manager::OutputDelegateManager;
use outputserver::instantiate_output_fold_delegate;

use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_fold = instantiate_output_fold_delegate();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let _ = tokio::spawn(wait_for_signal(shutdown_tx));

    serve_delegate_manager(
        "[::1]:50051",
        OutputDelegateManager { fold: output_fold },
        shutdown_rx,
    )
    .await
}
