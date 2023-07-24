// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use std::path::PathBuf;

pub mod config;
pub mod disabled;
pub mod fs_manager;

/// Cartesi Machine snapshot description
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Snapshot {
    pub path: PathBuf,
    pub epoch: u64,
    pub processed_input_count: u64,
}

#[async_trait::async_trait]
pub trait SnapshotManager {
    type Error: snafu::Error;

    /// Get the most recent snapshot
    async fn get_latest(&self) -> Result<Snapshot, Self::Error>;

    /// Get the target storage directory for the snapshot
    async fn get_storage_directory(
        &self,
        epoch: u64,
        processed_input_count: u64,
    ) -> Result<Snapshot, Self::Error>;

    /// Set the most recent snapshot
    async fn set_latest(&self, snapshot: Snapshot) -> Result<(), Self::Error>;
}
