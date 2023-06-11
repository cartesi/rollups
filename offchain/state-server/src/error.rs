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

use block_history::BlockArchiveError;
use snafu::Snafu;
use state_fold_types::ethers::providers::{Http, RetryClient};
use tonic::transport::Error as TonicError;
use url::ParseError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum StateServerError {
    #[snafu(display("tonic error"))]
    TonicError { source: TonicError },

    #[snafu(display("parser error"))]
    ParserError { source: ParseError },

    #[snafu(display("block archive error"))]
    BlockArchiveError {
        source: BlockArchiveError<
            state_fold_types::ethers::providers::Provider<RetryClient<Http>>,
        >,
    },
}
