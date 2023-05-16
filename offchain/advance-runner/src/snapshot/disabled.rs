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

use super::{Snapshot, SnapshotManager};

#[derive(Debug)]
pub struct SnapshotDisabled {}

#[derive(snafu::Snafu, Debug)]
#[snafu(display("shapshot disabled"))]
pub struct SnapshotDisabledError {}

#[async_trait::async_trait]
impl SnapshotManager for SnapshotDisabled {
    type Error = SnapshotDisabledError;

    /// Get the most recent snapshot
    #[tracing::instrument(level = "trace", skip_all)]
    async fn get_latest(&self) -> Result<Snapshot, SnapshotDisabledError> {
        tracing::trace!("snapshots disabled; returning default");
        Ok(Default::default())
    }

    /// Get the target storage directory for the snapshot
    #[tracing::instrument(level = "trace", skip_all)]
    async fn get_storage_directory(
        &self,
        _: u64,
        _: u64,
    ) -> Result<Snapshot, SnapshotDisabledError> {
        tracing::trace!("snapshots disabled; returning default");
        Ok(Default::default())
    }

    /// Set the most recent snapshot
    #[tracing::instrument(level = "trace", skip_all)]
    async fn set_latest(
        &self,
        _: Snapshot,
    ) -> Result<(), SnapshotDisabledError> {
        tracing::trace!("snapshots disabled; ignoring");
        Ok(())
    }
}
