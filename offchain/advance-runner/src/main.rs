// Copyright 2022 Cartesi Pte. Ltd.
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

use anyhow::{Context, Result};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use advance_runner::config::Config;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        tracing::error!("{:?}", e);
    }
}

async fn run() -> Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = Config::parse().context("config error")?;

    advance_runner::run(config)
        .await
        .context("advance runner error")?;

    Ok(())
}
