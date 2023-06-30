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
#[snafu(visibility(pub(crate)))]
pub enum InspectError {
    #[snafu(display("health check error"))]
    HealthCheckError {
        source: http_health_check::HealthCheckError,
    },

    #[snafu(display("server error"))]
    ServerError { source: std::io::Error },

    #[snafu(display("Failed to connect to server manager: {}", message))]
    FailedToConnect { message: String },

    #[snafu(display("Failed to inspect state: {}", message))]
    InspectFailed { message: String },
}
