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
    #[snafu(display("Invalid id provided for {}, id='{}'", item, source.to_string()))]
    InvalidIdError { item: String, source: ParseIntError },

    #[snafu(display("Invalid parameter provided"))]
    InvalidParameterError {},

    #[snafu(display("Database pool connection error: {}", message))]
    DatabasePoolConnectionError { message: String },

    #[snafu(display("Database error: {}", source.to_string()))]
    DatabaseError { source: diesel::result::Error },

    #[snafu(display("Unable to find {} with id='{}'", item_type, id))]
    ItemNotFound { item_type: String, id: String },

    #[snafu(display(
        "Unable to find input with index={} from epoch index={}",
        index,
        epoch_index
    ))]
    InputNotFound { epoch_index: i32, index: i32 },

    #[snafu(display("Unable to find epoch with index={}", index))]
    EpochNotFound { index: i32 },

    #[snafu(display(
        "Unable to find notice with index={} from epoch index={}",
        index,
        epoch_index
    ))]
    NoticeNotFound { epoch_index: i32, index: i32 },

    #[snafu(display(
        "Unable to find report with index={} from epoch index={}",
        index,
        epoch_index
    ))]
    ReportNotFound { epoch_index: i32, index: i32 },
}

pub type Result<T> = std::result::Result<T, Error>;
