// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use actix_web::{error, error::Error};

use crate::controller::ControllerError;
use crate::conversions::DecodeError;
use crate::model::RollupException;

use super::model::{DecodeStatusError, VoucherDecodeError};

impl From<RollupException> for Error {
    fn from(e: RollupException) -> Error {
        error::ErrorInternalServerError(e.to_string())
    }
}

impl From<ControllerError> for Error {
    fn from(e: ControllerError) -> Error {
        error::ErrorBadRequest(e.to_string())
    }
}

impl From<DecodeError> for Error {
    fn from(e: DecodeError) -> Error {
        error::ErrorBadRequest(e.to_string())
    }
}

impl From<VoucherDecodeError> for Error {
    fn from(e: VoucherDecodeError) -> Error {
        error::ErrorBadRequest(e.to_string())
    }
}

impl From<DecodeStatusError> for Error {
    fn from(e: DecodeStatusError) -> Error {
        error::ErrorBadRequest(e.to_string())
    }
}
