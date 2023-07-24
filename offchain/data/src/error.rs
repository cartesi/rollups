// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("database pool connection error"))]
    DatabaseConnectionError {
        source: backoff::Error<diesel::r2d2::PoolError>,
    },

    #[snafu(display("database error"))]
    DatabaseError { source: diesel::result::Error },

    #[snafu(display("{} not found", item_type))]
    ItemNotFound { item_type: String },

    #[snafu(display("failed to decode UTF8 cursor"))]
    DecodeUTF8CursorError { source: std::str::Utf8Error },

    #[snafu(display("failed to decode base64 cursor"))]
    DecodeBase64CursorError { source: base64::DecodeError },

    #[snafu(display("failed to parse cursor"))]
    ParseCursorError { source: std::num::ParseIntError },

    #[snafu(display(
        "cannot mix forward pagination (first, after) with backward pagination (last, before)"
    ))]
    MixedPaginationError {},

    #[snafu(display("invalid pagination cursor {}", arg))]
    PaginationCursorError { arg: String },

    #[snafu(display("invalid pagination limit {}", arg))]
    PaginationLimitError { arg: String },
}
