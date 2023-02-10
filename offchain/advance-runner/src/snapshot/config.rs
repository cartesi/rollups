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

use clap::Parser;
use snafu::{ensure, Snafu};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FSManagerConfig {
    pub snapshot_dir: PathBuf,
    pub snapshot_latest: PathBuf,
}

#[derive(Debug, Clone)]
pub enum SnapshotConfig {
    FileSystem(FSManagerConfig),
    Disabled,
}

impl SnapshotConfig {
    pub fn parse_from_cli(
        cli_config: SnapshotCLIConfig,
    ) -> Result<Self, SnapshotConfigError> {
        if cli_config.snapshot_enabled {
            let snapshot_dir = PathBuf::from(cli_config.snapshot_dir);
            ensure!(snapshot_dir.is_dir(), DirSnafu);

            let snapshot_latest = PathBuf::from(cli_config.snapshot_latest);
            ensure!(snapshot_latest.is_symlink(), SymlinkSnafu);

            Ok(SnapshotConfig::FileSystem(FSManagerConfig {
                snapshot_dir,
                snapshot_latest,
            }))
        } else {
            Ok(SnapshotConfig::Disabled)
        }
    }
}

#[derive(Debug, Snafu)]
pub enum SnapshotConfigError {
    #[snafu(display("Snapshot dir isn't a directory"))]
    DirError {},

    #[snafu(display("Snapshot latest isn't a symlink"))]
    SymlinkError {},
}

#[derive(Parser, Debug)]
#[command(name = "snapshot")]
pub struct SnapshotCLIConfig {
    /// If set, disable snapshots
    #[arg(long, env, default_value_t = true)]
    snapshot_enabled: bool,

    /// Path to the directory with the snapshots
    #[arg(long, env, default_value = "/opt/cartesi/share/dapp-bin")]
    snapshot_dir: String,

    /// Path to the symlink of the latest snapshot
    #[arg(long, env, default_value = "/opt/cartesi/share/dapp-bin/latest")]
    snapshot_latest: String,
}
