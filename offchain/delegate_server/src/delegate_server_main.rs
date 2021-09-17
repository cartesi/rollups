#![warn(unused_extern_crates)]
use state_fold::{Access, StateFold};
use state_server_grpc::{serve_delegate_manager, wait_for_signal};

use ethers::providers::{Http, Provider};
use ethers::types::U64;
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::sync::oneshot;

static HTTP_URL: &'static str = "http://localhost:8545";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(Provider::<Http>::try_from(HTTP_URL).unwrap());

    let access =
        Arc::new(Access::new(Arc::clone(&provider), U64::from(0), vec![], 4));

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let _ = tokio::spawn(wait_for_signal(shutdown_tx));

    let input_delegate =
        offchain::fold::input_delegate::InputFoldDelegate::default();
    let input_fold = StateFold::new(input_delegate, Arc::clone(&access), 0);

    serve_delegate_manager(
        "[::1]:50051",
        delegate_server::input_server::InputDelegateManager {
            fold: input_fold,
        },
        shutdown_rx,
    )
    .await
}
