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
