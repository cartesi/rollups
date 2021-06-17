use snafu::Snafu;

use crate::logic::instantiate_state_fold::DescartesAccess;
use dispatcher::middleware_factory;
use dispatcher::state_fold;
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
}

pub type Result<T> = std::result::Result<T, Error>;
