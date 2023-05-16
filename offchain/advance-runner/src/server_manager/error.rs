// Copyright Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum ServerManagerError {
    #[snafu(display("failed to connect to server-manager"))]
    ConnectionError { source: tonic::transport::Error },

    #[snafu(display(
        "failed to call {} with request-id {}",
        method,
        request_id
    ))]
    MethodCallError {
        method: String,
        request_id: String,
        source: tonic::Status,
    },

    #[snafu(display("maximum number of retries exceeded"))]
    PendingInputsExceededError {},

    #[snafu(display("missing field {}", name))]
    MissingFieldError { name: String },

    #[snafu(display(
        "array of wrong size for {} type, expected {} but got {}",
        name,
        expected,
        got
    ))]
    WrongArraySizeError {
        name: String,
        expected: usize,
        got: usize,
    },

    #[snafu(display("missing processed input in get epoch status"))]
    MissingProcessedInputError {},

    #[snafu(display(
        "invalid last processed input index, expected {} but got {}",
        expected,
        got
    ))]
    InvalidProcessedInputError { expected: u64, got: u64 },

    #[snafu(display(
        "can't generate claim for epoch {} because it has no inputs",
        epoch_index
    ))]
    EmptyEpochError { epoch_index: u64 },
}
