/* Copyright 2022 Cartesi Pte. Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not
 * use this file except in compliance with the License. You may obtain a copy of
 * the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations under
 * the License.
 */

use snafu::Snafu;
use std::num::ParseIntError;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Invalid id provided: {}", source.to_string()))]
    InvalidIdError { source: ParseIntError },

    #[snafu(display("Database pool connection error: {}", message))]
    DatabasePoolConnectionError { message: String },

    #[snafu(display("Database error: {}", source.to_string()))]
    DatabaseError { source: diesel::result::Error },

    #[snafu(display("Unable to find item with id {}", id))]
    ItemNotFound { id: String },
}

pub type Result<T> = std::result::Result<T, Error>;
