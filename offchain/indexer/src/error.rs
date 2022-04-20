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

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Tonic status error: {:?}", source))]
    TonicStatusError { source: tonic::Status },

    #[snafu(display("Tonic transport error: {:?}", source))]
    TonicTransportError { source: tonic::transport::Error },

    #[snafu(display("Serialize error: {}", source))]
    SerializeError { source: serde_json::Error },

    #[snafu(display("Deserialize error: {}", source))]
    DeserializeError { source: serde_json::Error },

    #[snafu(display("R2D2 error: {}", source))]
    R2D2Error { source: diesel::r2d2::PoolError },

    #[snafu(display("Diesel error, source: {}", source.to_string()))]
    DieselError { source: diesel::result::Error },

    #[snafu(display("Bad configuration: {}", err))]
    BadConfiguration { err: String },

    #[snafu(display("Server Manager out of sync: {}", err))]
    OutOfSync { err: String },

    #[snafu(display("Tokio join error: {:?}", source))]
    TokioError { source: tokio::task::JoinError },
}

pub type Result<T> = std::result::Result<T, Error>;
