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

use std::fs;
use std::os::unix;
use std::path::Path;
use tempfile::TempDir;

use crate::docker_cli;

const TAG: &str = "cartesi/test-machine-snapshot";
const DOCKERFILE: &str = "../test-fixtures/docker/machine_snapshot.Dockerfile";
const CONTAINER_SNAPSHOT_DIR: &str = "/opt/cartesi/share/dapp-bin";
const SNAPSHOT_NAME: &str = "0_0";

pub struct MachineSnapshotsFixture {
    dir: TempDir,
}

impl MachineSnapshotsFixture {
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn setup() -> Self {
        tracing::info!("setting up machine snapshots fixture");

        let dir = tempfile::tempdir().expect("failed to create temp dir");
        docker_cli::build(DOCKERFILE, TAG, &[]);
        let id = docker_cli::create(TAG);
        let from_container = format!("{}:{}", id, CONTAINER_SNAPSHOT_DIR);
        let to_host = dir.path().join(SNAPSHOT_NAME);
        docker_cli::cp(&from_container, to_host.to_str().unwrap());
        docker_cli::rm(&id);
        unix::fs::symlink(
            dir.path().join(SNAPSHOT_NAME),
            dir.path().join("latest"),
        )
        .expect("failed to create latest link");
        Self { dir }
    }

    /// Return the path of directory that contains the snapshots
    pub fn path(&self) -> &Path {
        &self.dir.path()
    }

    /// Check whether the given snapshot is the latest
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn assert_latest_snapshot(
        &self,
        epoch_index: u64,
        processed_input_count: u64,
    ) {
        tracing::trace!(
            epoch_index,
            processed_input_count,
            "checking the latest snapshot"
        );
        let snapshot_name =
            format!("{}_{}", epoch_index, processed_input_count);
        let snapshot = self.path().join(snapshot_name);
        assert!(snapshot.is_dir(), "snapshot not found");
        let latest = self.path().join("latest");
        assert!(latest.is_symlink(), "latest link not found");
        assert_eq!(
            fs::read_link(&latest).unwrap(),
            snapshot,
            "invalid latest link"
        );

        tracing::trace!("checking whether the other snapshots were deleted");
        let dir_iterator = fs::read_dir(&self.path()).unwrap();
        for entry in dir_iterator {
            let path = entry.unwrap().path();
            assert!(
                path == latest || path == snapshot,
                "previous snapshots not deleted"
            );
        }
    }
}
