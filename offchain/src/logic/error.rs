use snafu::Snafu;

use dispatcher::middleware_factory;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Parse error: {}", source))]
    UrlParseError { source: url::ParseError },

    #[snafu(display("Parse error: {}", source))]
    MiddlewareFactoryError { source: middleware_factory::Error },
}

pub type Result<T> = std::result::Result<T, Error>;
