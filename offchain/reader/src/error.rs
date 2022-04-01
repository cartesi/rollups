use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Bad configuration {}", err))]
    BadConfiguration { err: String },
    #[snafu(display("Http service error: {}", source))]
    HttpServiceError { source: std::io::Error },
}

pub type Result<T> = std::result::Result<T, Error>;
