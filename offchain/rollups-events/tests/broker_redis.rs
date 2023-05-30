// Copyright Cartesi Pte. Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use testcontainers::{
    clients::Cli, core::WaitFor, images::generic::GenericImage, Container,
};

use test_fixtures::docker_cli;

const DOCKERFILE: &str = "tests/tls.Dockerfile";
const DOCKER_TAG: &str = "cartesi/broker-tls-test";
const BUILD_ROOT: &str = ".."; //offchain dir

pub struct BrokerRedisFixture<'d> {
    pub node: Container<'d, GenericImage>,
}

impl BrokerRedisFixture<'_> {
    pub fn setup(docker: &Cli) -> BrokerRedisFixture<'_> {
        docker_cli::build(DOCKERFILE, DOCKER_TAG, &[], Some(BUILD_ROOT));
        let image = GenericImage::new(DOCKER_TAG, "latest").with_wait_for(
            WaitFor::message_on_stdout("Ready to accept connections"),
        );
        let node = docker.run(image);
        BrokerRedisFixture { node }
    }
}
