// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

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
    /// If set to false, disable snapshots
    #[arg(long, env, default_value_t = true)]
    snapshot_enabled: bool,

    /// Path to the directory with the snapshots
    #[arg(long, env)]
    snapshot_dir: String,

    /// Path to the symlink of the latest snapshot
    #[arg(long, env)]
    snapshot_latest: String,
}
