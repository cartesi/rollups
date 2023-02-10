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

use snafu::{ensure, OptionExt, ResultExt, Snafu};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::config::FSManagerConfig;
use super::{Snapshot, SnapshotManager};

#[derive(Debug, Snafu)]
pub enum FSSnapshotError {
    #[snafu(display("failed to follow latest symlink"))]
    ReadLinkError { source: std::io::Error },

    #[snafu(display("failed to read symlink path={:?}", path))]
    BrokenLinkError { path: PathBuf },

    #[snafu(display("failed to get snapshot file name"))]
    DirNameError {},

    #[snafu(display("failed to parse the epoch from snapshot file name"))]
    ParsingError { source: std::num::ParseIntError },

    #[snafu(display("failed to remove snapshot {:?}", path))]
    RemoveError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("snapshot on wrong dir {:?}", path))]
    WrongDirError { path: PathBuf },

    #[snafu(display("snapshot path with invalid epoch {:?}", snapshot))]
    InvalidEpochError { snapshot: Snapshot },

    #[snafu(display("failed to read snapshot {:?}", path))]
    NotFoundError { path: PathBuf },

    #[snafu(display("failed to list snapshots in dir"))]
    ListDirError { source: std::io::Error },

    #[snafu(display("existing latest path exists but it is not symlink"))]
    LatestNotLinkError {},

    #[snafu(display("failed to set latest symlink"))]
    SetLatestError { source: std::io::Error },
}

#[derive(Debug)]
pub struct FSSnapshotManager {
    config: FSManagerConfig,
}

impl FSSnapshotManager {
    pub fn new(config: FSManagerConfig) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl SnapshotManager for FSSnapshotManager {
    type Error = FSSnapshotError;

    #[tracing::instrument(level = "trace", skip_all)]
    async fn get_latest(&self) -> Result<Snapshot, Self::Error> {
        tracing::trace!("getting latest snapshot");

        let path = fs::read_link(&self.config.snapshot_latest)
            .context(ReadLinkSnafu)?;
        ensure!(path.is_dir(), BrokenLinkSnafu { path: path.clone() });
        tracing::trace!(?path, "followed latest link");

        path.try_into()
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn get_storage_directory(
        &self,
        epoch: u64,
    ) -> Result<Snapshot, Self::Error> {
        tracing::trace!(epoch, "getting storage directory");

        let mut path = self.config.snapshot_dir.clone();
        path.push(epoch.to_string());
        tracing::trace!(?path, "computed the path");

        // Make sure that the target directory for the snapshot doesn't exists
        if path.exists() {
            tracing::warn!(?path, "storage directory already exists");
            std::fs::remove_dir_all(&path)
                .context(RemoveSnafu { path: path.clone() })?;
        }

        Ok(Snapshot { path, epoch })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn set_latest(&self, snapshot: Snapshot) -> Result<(), Self::Error> {
        tracing::trace!(?snapshot, "setting latest snapshot");

        // basic verifications
        ensure!(
            snapshot.path.parent() == Some(&self.config.snapshot_dir),
            WrongDirSnafu {
                path: snapshot.path.clone()
            }
        );
        ensure!(
            get_epoch_number(&snapshot.path)? == snapshot.epoch,
            InvalidEpochSnafu { snapshot }
        );
        ensure!(
            snapshot.path.is_dir(),
            NotFoundSnafu {
                path: snapshot.path.clone()
            }
        );

        // list other snapshots
        let mut snapshots = HashSet::new();
        let dir_iterator =
            fs::read_dir(&self.config.snapshot_dir).context(ListDirSnafu)?;
        for entry in dir_iterator {
            let entry = entry.context(ListDirSnafu)?;
            let path = entry.path();
            if path != self.config.snapshot_latest && path != snapshot.path {
                snapshots.insert(path.to_owned());
            }
        }
        tracing::trace!(?snapshots, "listed other existing snapshots");

        // delete previous snapshot link
        if self.config.snapshot_latest.exists() {
            ensure!(
                self.config.snapshot_latest.is_symlink(),
                LatestNotLinkSnafu
            );
            fs::remove_file(&self.config.snapshot_latest)
                .context(SetLatestSnafu)?;
            tracing::trace!("deleted previous latest symlink");
        }

        // set latest link
        std::os::unix::fs::symlink(
            &snapshot.path,
            &self.config.snapshot_latest,
        )
        .context(SetLatestSnafu)?;
        tracing::trace!("set latest snapshot");

        // delete other snapshots
        for path in snapshots.iter() {
            fs::remove_dir_all(&path)
                .context(RemoveSnafu { path: path.clone() })?;
        }
        tracing::trace!("deleted previous snapshots");

        Ok(())
    }
}

fn get_epoch_number(path: &Path) -> Result<u64, FSSnapshotError> {
    let file_name = path
        .file_name()
        .map(|file_name| file_name.to_str())
        .flatten()
        .context(DirNameSnafu)?;
    tracing::trace!(file_name, "got snapshot file name");

    let epoch = file_name.parse::<u64>().context(ParsingSnafu)?;
    tracing::trace!(epoch, "got epoch number");

    Ok(epoch)
}

impl TryFrom<PathBuf> for Snapshot {
    type Error = FSSnapshotError;

    fn try_from(path: PathBuf) -> Result<Snapshot, FSSnapshotError> {
        let epoch = get_epoch_number(&path)?;
        Ok(Snapshot { path, epoch })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestState {
        tempdir: TempDir,
        manager: FSSnapshotManager,
    }

    impl TestState {
        fn setup() -> Self {
            let tempdir =
                tempfile::tempdir().expect("failed to create temp dir");
            let snapshot_dir = tempdir.path().to_owned();
            let mut snapshot_latest = snapshot_dir.clone();
            snapshot_latest.push("latest");
            let config = FSManagerConfig {
                snapshot_dir,
                snapshot_latest,
            };
            let manager = FSSnapshotManager::new(config);
            Self { tempdir, manager }
        }

        fn create_snapshot(&self, name: &str) -> PathBuf {
            let path = self.tempdir.path().join(name);
            fs::create_dir(&path).expect("failed to create dir");
            path
        }

        fn list_snapshots_dir(&self) -> Vec<PathBuf> {
            let mut files = vec![];
            let dir_iterator = fs::read_dir(&self.tempdir.path()).unwrap();
            for entry in dir_iterator {
                let entry = entry.unwrap();
                files.push(entry.path());
            }
            files.sort();
            files
        }
    }

    #[test_log::test(tokio::test)]
    async fn test_it_fails_to_read_latest_link() {
        let state = TestState::setup();
        let err = state
            .manager
            .get_latest()
            .await
            .expect_err("get latest should fail");
        assert!(matches!(err, FSSnapshotError::ReadLinkError { .. }));
    }

    #[test_log::test(tokio::test)]
    async fn test_it_fails_to_get_latest_when_link_is_broken() {
        let state = TestState::setup();
        std::os::unix::fs::symlink(
            state.tempdir.path().join("0"),
            state.tempdir.path().join("latest"),
        )
        .expect("failed to create link");
        let err = state
            .manager
            .get_latest()
            .await
            .expect_err("get latest should fail");
        assert!(matches!(err, FSSnapshotError::BrokenLinkError { .. }));
    }

    #[test_log::test(tokio::test)]
    async fn test_it_fails_to_get_latest_when_dirname_is_wrong() {
        let state = TestState::setup();
        state.create_snapshot("invalid-name");
        std::os::unix::fs::symlink(
            state.tempdir.path().join("invalid-name"),
            state.tempdir.path().join("latest"),
        )
        .expect("failed to create link");
        let err = state
            .manager
            .get_latest()
            .await
            .expect_err("get latest should fail");
        assert!(matches!(err, FSSnapshotError::ParsingError { .. }));
    }

    #[test_log::test(tokio::test)]
    async fn test_it_get_latest_snapshot() {
        let state = TestState::setup();
        state.create_snapshot("0");
        state.create_snapshot("1");
        state.create_snapshot("2");
        std::os::unix::fs::symlink(
            state.tempdir.path().join("1"),
            state.tempdir.path().join("latest"),
        )
        .expect("failed to create link");
        let snapshot = state
            .manager
            .get_latest()
            .await
            .expect("failed to get latest");
        assert_eq!(
            snapshot,
            Snapshot {
                path: state.tempdir.path().join("1"),
                epoch: 1
            }
        );
    }

    #[test_log::test(tokio::test)]
    async fn test_it_gets_storage_when_snapshot_does_not_exist() {
        let state = TestState::setup();
        let storage_directory = state
            .manager
            .get_storage_directory(0)
            .await
            .expect("get storage directory should not fail");
        assert_eq!(
            storage_directory,
            Snapshot {
                path: state.tempdir.path().join("0"),
                epoch: 0,
            }
        );
        assert!(state.list_snapshots_dir().is_empty());
    }

    #[test_log::test(tokio::test)]
    async fn test_it_gets_storage_when_snapshot_already_exists() {
        let state = TestState::setup();
        state.create_snapshot("0");
        state.create_snapshot("1");
        state.create_snapshot("2");
        let storage_directory = state
            .manager
            .get_storage_directory(2)
            .await
            .expect("get storage directory should not fail");
        assert_eq!(
            storage_directory,
            Snapshot {
                path: state.tempdir.path().join("2"),
                epoch: 2,
            }
        );
        assert_eq!(
            state.list_snapshots_dir(),
            vec![
                state.tempdir.path().join("0"),
                state.tempdir.path().join("1"),
            ]
        );
    }

    #[test_log::test(tokio::test)]
    async fn test_it_fails_to_set_latest_when_path_is_not_on_snapshots_dir() {
        let state = TestState::setup();
        let path = state.tempdir.path().parent().unwrap().join("0");
        let err = state
            .manager
            .set_latest(Snapshot { path, epoch: 0 })
            .await
            .expect_err("set latest should fail");
        assert!(matches!(err, FSSnapshotError::WrongDirError { .. }));
    }

    #[test_log::test(tokio::test)]
    async fn test_it_fails_to_set_latest_when_epoch_mismatches() {
        let state = TestState::setup();
        let err = state
            .manager
            .set_latest(Snapshot {
                path: state.tempdir.path().join("0"),
                epoch: 1,
            })
            .await
            .expect_err("set latest should fail");
        assert!(matches!(err, FSSnapshotError::InvalidEpochError { .. }));
    }

    #[test_log::test(tokio::test)]
    async fn test_it_fails_to_set_latest_when_dir_does_not_exist() {
        let state = TestState::setup();
        let err = state
            .manager
            .set_latest(Snapshot {
                path: state.tempdir.path().join("0"),
                epoch: 0,
            })
            .await
            .expect_err("set latest should fail");
        assert!(matches!(err, FSSnapshotError::NotFoundError { .. }));
    }

    #[test_log::test(tokio::test)]
    async fn test_it_fails_to_set_latest_when_latest_is_not_symlink() {
        let state = TestState::setup();
        state.create_snapshot("0");
        state.create_snapshot("latest");
        let err = state
            .manager
            .set_latest(Snapshot {
                path: state.tempdir.path().join("0"),
                epoch: 0,
            })
            .await
            .expect_err("set latest should fail");
        assert!(matches!(err, FSSnapshotError::LatestNotLinkError { .. }));
    }

    #[test_log::test(tokio::test)]
    async fn test_it_sets_latest_snapshot() {
        let state = TestState::setup();
        state.create_snapshot("0");
        state
            .manager
            .set_latest(Snapshot {
                path: state.tempdir.path().join("0"),
                epoch: 0,
            })
            .await
            .expect("set latest should work");
        assert_eq!(
            state.list_snapshots_dir(),
            vec![
                state.tempdir.path().join("0"),
                state.tempdir.path().join("latest"),
            ]
        );
        assert_eq!(
            fs::read_link(&state.tempdir.path().join("latest")).unwrap(),
            state.tempdir.path().join("0"),
        );
    }

    #[test_log::test(tokio::test)]
    async fn test_it_deletes_previous_snapshots_after_setting_latest() {
        let state = TestState::setup();
        state.create_snapshot("0");
        state.create_snapshot("1");
        state.create_snapshot("2");
        std::os::unix::fs::symlink(
            state.tempdir.path().join("1"),
            state.tempdir.path().join("latest"),
        )
        .expect("failed to create link");
        state
            .manager
            .set_latest(Snapshot {
                path: state.tempdir.path().join("2"),
                epoch: 2,
            })
            .await
            .expect("set latest should work");
        assert_eq!(
            state.list_snapshots_dir(),
            vec![
                state.tempdir.path().join("2"),
                state.tempdir.path().join("latest"),
            ]
        );
        assert_eq!(
            fs::read_link(&state.tempdir.path().join("latest")).unwrap(),
            state.tempdir.path().join("2"),
        );
    }
}
