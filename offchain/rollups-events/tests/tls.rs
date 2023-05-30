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

mod broker_redis;
use test_fixtures::docker_cli;
use testcontainers::clients::Cli;

const TEST_EXECUTABLE: &str = "broker-tls-test";

/// Currently, the redis client we're using does not provide a high-level API to
/// load certificates, it only loads the ones in the host machine's certificate
/// store. To avoid having to install certificates in the machine running the
/// test, it builds and runs a Docker image containing both the Redis server
/// and a test binary that connects to it, executing the latter after the
/// container is up.
#[test_log::test(tokio::test)]
async fn test_it_connects_via_tls() {
    let docker = Cli::default();
    let fixture = broker_redis::BrokerRedisFixture::setup(&docker);
    let output = docker_cli::exec(fixture.node.id(), TEST_EXECUTABLE);
    assert!(output.is_empty());
}
