use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Instance not found: {}", err))]
    InstanceNotFound { err: String },

    #[snafu(display("Invaid date: {}", err))]
    InvalidDate { err: String },

    #[snafu(display("Invaid amount: {}", err))]
    InvalidAmount { err: String },

    #[snafu(display("Invaid event: {}", err))]
    InvalidEvent { err: String },

    #[snafu(display("Bad configuration: {}", err))]
    BadConfiguration { err: String },
}

pub type Result<T> = std::result::Result<T, Error>;
