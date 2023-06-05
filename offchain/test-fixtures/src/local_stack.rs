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

use std::fs::read_to_string;

use crate::docker_cli;

const DOCKER_TAG: &str = "cartesi/localstack";
const DOCKERFILE: &str = "local_stack";
const KEY_ID_NAME: &str = "key_id.txt";
const VOLUME_PATH: &str = "/app";

const AWS_CREATE_KEY_CMD: &str =
    "awslocal kms create-key --key-spec ECC_SECG_P256K1 --key-usage SIGN_VERIFY";

pub struct LocalStackFixture {
    container_id: String,
}

// Stops the container and removes it.
impl Drop for LocalStackFixture {
    fn drop(&mut self) {
        docker_cli::stop(&self.container_id);
        docker_cli::rm(&self.container_id);
    }
}

/// Tests that use this fixture must be executed serially to avoid port conflicts.
impl LocalStackFixture {
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn setup() -> LocalStackFixture {
        tracing::info!("setting up LocalStack fixture");

        tracing::debug!("building the docker image");
        docker_cli::build(DOCKERFILE, DOCKER_TAG, &[]);
        tracing::debug!("finished building the docker image");

        let container_id = docker_cli::run(
            "cartesi/localstack",
            &["-p", "4566:4566", "-v", VOLUME_PATH],
        );
        tracing::debug!("running container {}", container_id);

        Self { container_id }
    }

    pub const ENDPOINT: &str = "http://0.0.0.0:4566";

    /// Returns the key's ID.
    pub fn create_key(&self) -> String {
        let key_id_path = format!("{}/{}", VOLUME_PATH, KEY_ID_NAME);

        tracing::debug!("creating key");
        let aws_cmd = &format!("{} > {} 2>&1", AWS_CREATE_KEY_CMD, key_id_path);
        let docker_cmd = ["sh", "-c", aws_cmd];
        docker_cli::exec(&self.container_id, &docker_cmd);
        tracing::debug!("key created");

        tracing::debug!("copying key to host");
        let from_container = &format!("{}:{}", self.container_id, key_id_path);
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let to_host = temp_dir.path().join(KEY_ID_NAME);
        let to_host = to_host.to_str().unwrap();
        docker_cli::cp(from_container, to_host);
        tracing::debug!("key copied to host");

        let json_string = read_to_string(to_host).expect("failed to read key");
        let json_value = json::parse(&json_string).unwrap();
        let key_id = json_value["KeyMetadata"]["KeyId"].to_string();
        tracing::info!("key_id: {}", key_id);

        key_id
    }
}
