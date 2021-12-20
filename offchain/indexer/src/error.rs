use snafu::Snafu;

use offchain::error;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Tonic status error: {}", source))]
    TonicStatusError { source: tonic::Status },

    #[snafu(display("Tonic transport error: {}", source))]
    TonicTransportError { source: tonic::transport::Error },

    #[snafu(display("Serialize error: {}", source))]
    SerializeError { source: serde_json::Error },

    #[snafu(display("Deserialize error: {}", source))]
    DeserializeError { source: serde_json::Error },

    #[snafu(display("R2D2 error: {}", source))]
    R2D2Error { source: diesel::r2d2::PoolError },

    #[snafu(display("Offchain error: {}", source))]
    OffchainError { source: error::Error },

    #[snafu(display("Diesel error"))]
    DieselError { source: diesel::result::Error },

    #[snafu(display("Bad configuration: {}", err))]
    BadConfiguration { err: String },

    #[snafu(display("Server Manager out of sync: {}", err))]
    OutOfSync { err: String },
}

pub type Result<T> = std::result::Result<T, Error>;
