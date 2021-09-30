use snafu::Snafu;

use crate::logic::instantiate_state_fold::DescartesAccess;
use middleware_factory;
use state_fold;
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

    #[snafu(display("State fold error"))]
    StateFoldError {
        source: state_fold::error::Error<DescartesAccess>,
    },

    #[snafu(display("Tonic status error"))]
    TonicStatusError { source: tonic::Status },

    #[snafu(display("Tonic transport error"))]
    TonicTransportError { source: tonic::transport::Error },

    #[snafu(display("Deserialize error"))]
    DeserializeError { source: serde_json::Error },
}

pub type Result<T> = std::result::Result<T, Error>;
