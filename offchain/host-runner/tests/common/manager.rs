// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use std::process::{Child, Command, Stdio};
use std::time::Duration;

use super::config;
use super::grpc_client::{ServerManagerClient, Void};

pub struct Wrapper {
    child: Child,
}

impl Wrapper {
    /// Start the manager and waits until it is ready to answer
    pub async fn new() -> Self {
        let mut command = Command::new(config::get_host_runner_path());
        command
            .env("RUST_LOG", "host_runner=debug,info")
            .arg("--grpc-server-manager-port")
            .arg(config::GRPC_SERVER_MANAGER_PORT.to_string())
            .arg("--http-inspect-port")
            .arg(config::HTTP_INSPECT_PORT.to_string())
            .arg("--http-rollup-server-port")
            .arg(config::HTTP_ROLLUP_SERVER_PORT.to_string())
            .arg("--finish-timeout")
            .arg(config::FINISH_TIMEOUT.to_string());
        if !config::get_test_verbose() {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }
        // Wait for a bit to clean up the port from previous test
        tokio::time::sleep(Duration::from_millis(10)).await;
        let child = command.spawn().expect("failed to start manager process");
        wait_for_manager().await;
        Self { child }
    }
}

impl Drop for Wrapper {
    fn drop(&mut self) {
        self.child.kill().expect("failed to kill manager process");
    }
}

async fn wait_for_manager() {
    const RETRIES: u64 = 100;
    for _ in 0..RETRIES {
        let address = config::get_grpc_server_manager_address();
        if let Ok(mut client) = ServerManagerClient::connect(address).await {
            if let Ok(_) = client.get_version(Void {}).await {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("manager timed out");
}
