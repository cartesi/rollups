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

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use graphql_server::CLIConfig;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tracing_format = tracing_subscriber::fmt::format()
        .without_time()
        .with_level(true)
        .with_target(true)
        .with_ansi(false)
        .compact();
    if std::env::var(EnvFilter::DEFAULT_ENV).is_ok() {
        tracing_subscriber::fmt()
            .event_format(tracing_format)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::fmt()
            .event_format(tracing_format)
            .with_max_level(LevelFilter::INFO)
            .init();
    }

    let config = CLIConfig::parse().into();

    graphql_server::run(config).await.map_err(|e| e.into())
}
