use snafu::Snafu;

use middleware_factory;
use offchain_core::ethers::signers::WalletError;
use tokio::sync::broadcast::error::RecvError;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Parse error: {}", source))]
    UrlParseError { source: url::ParseError },

    #[snafu(display("Middleware factory error: {}", source))]
    MiddlewareFactoryError { source: middleware_factory::Error },

    #[snafu(display("Failed to subscribe"))]
    EmptySubscription {},

    #[snafu(display("Failed to receive"))]
    SubscriberReceiveError { source: RecvError },

    #[snafu(display("Tonic status error"))]
    TonicStatusError { source: tonic::Status },

    #[snafu(display("Tonic transport error"))]
    TonicTransportError { source: tonic::transport::Error },

    #[snafu(display("Deserialize error"))]
    DeserializeError { source: serde_json::Error },

    #[snafu(display("Bad configuration: {}", err))]
    BadConfiguration { err: String },

    #[snafu(display("Mnemonic builder error: {}", source))]
    MnemonicError { source: WalletError },
}

pub type Result<T> = std::result::Result<T, Error>;
