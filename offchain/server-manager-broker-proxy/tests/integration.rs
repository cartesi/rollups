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

use fixtures::ProxyFixture;
use rand::Rng;
use rollups_events::broker::INITIAL_ID;
use rollups_events::rollups_inputs::{
    InputMetadata, RollupsData, RollupsInput,
};
use rollups_events::HASH_SIZE;
use test_fixtures::{
    BrokerFixture, MachineSnapshotsFixture, ServerManagerFixture,
};
use testcontainers::clients::Cli;

mod fixtures;

struct TestState<'d> {
    snapshots: MachineSnapshotsFixture,
    broker: BrokerFixture<'d>,
    server_manager: ServerManagerFixture<'d>,
    proxy: ProxyFixture,
}

impl TestState<'_> {
    async fn setup(docker: &Cli) -> TestState<'_> {
        let snapshots = MachineSnapshotsFixture::setup();
        let broker = BrokerFixture::setup(docker).await;
        let server_manager =
            ServerManagerFixture::setup(docker, snapshots.path()).await;
        let proxy = ProxyFixture::setup(
            server_manager.endpoint().to_owned(),
            server_manager.session_id().to_owned(),
            broker.redis_endpoint().to_owned(),
            broker.chain_id(),
            broker.dapp_contract_address().to_owned(),
            snapshots.path(),
        )
        .await;
        TestState {
            snapshots,
            broker,
            server_manager,
            proxy,
        }
    }
}

fn generate_payload() -> Vec<u8> {
    let len = rand::thread_rng().gen_range(100..200);
    (0..len).map(|_| rand::thread_rng().gen()).collect()
}

#[test_log::test(tokio::test)]
async fn test_proxy_starts_server_manager_session() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;

    tracing::info!("checking whether proxy created session");
    state.server_manager.assert_session_ready().await;
}

#[test_log::test(tokio::test)]
async fn test_proxy_sends_inputs_to_server_manager() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;

    const N: usize = 3;
    tracing::info!("producing {} inputs", N);
    let payloads: Vec<_> = (0..N).map(|_| generate_payload()).collect();
    for (i, payload) in payloads.iter().enumerate() {
        let data = RollupsData::AdvanceStateInput {
            input_metadata: InputMetadata {
                epoch_index: 0,
                input_index: i as u64,
                ..Default::default()
            },
            input_payload: payload.clone(),
        };
        state.broker.produce_input_event(data).await;
    }

    tracing::info!("waiting until the inputs are processed");
    state.server_manager.assert_session_ready().await;
    state
        .server_manager
        .assert_epoch_status_payloads(0, &payloads)
        .await;
}

#[test_log::test(tokio::test)]
async fn test_proxy_fails_when_inputs_has_wrong_epoch() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;

    tracing::info!("producing input with wrong epoch index");
    let data = RollupsData::AdvanceStateInput {
        input_metadata: InputMetadata {
            epoch_index: 0,
            input_index: 0,
            ..Default::default()
        },
        input_payload: vec![],
    };
    let input = RollupsInput {
        parent_id: INITIAL_ID.to_owned(),
        epoch_index: 1,
        inputs_sent_count: 1,
        data,
    };
    state.broker.produce_raw_input_event(input).await;

    tracing::info!("waiting for the proxy to exit with error");
    let err = state.proxy.wait_err().await;
    assert!(format!("{:?}", err).contains("incorrect active epoch index"));
}

#[test_log::test(tokio::test)]
async fn test_proxy_fails_when_inputs_has_wrong_parent_id() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;

    tracing::info!("producing input with wrong parent id");
    let data = RollupsData::AdvanceStateInput {
        input_metadata: InputMetadata {
            epoch_index: 0,
            input_index: 0,
            ..Default::default()
        },
        input_payload: vec![],
    };
    let input = RollupsInput {
        parent_id: "invalid".to_owned(),
        epoch_index: 0,
        inputs_sent_count: 1,
        data,
    };
    state.broker.produce_raw_input_event(input).await;

    tracing::info!("waiting for the proxy to exit with error");
    let err = state.proxy.wait_err().await;
    assert!(format!("{:?}", err).contains("parent id doesn't match"));
}

#[test_log::test(tokio::test)]
async fn test_proxy_generates_claim_after_finishing_epoch() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;

    const N: usize = 3;
    tracing::info!("producing {} finish epoch events", N);
    let inputs = vec![RollupsData::FinishEpoch {}; N];
    for input in inputs {
        state.broker.produce_input_event(input).await;
    }

    tracing::info!("waiting until the expected claims are generated");
    state.server_manager.assert_session_ready().await;
    let claims = state.broker.consume_n_claims(N).await;
    // We don't verify the claim hash because it is not the resposability of the
    // proxy and because it changes every time we update the Cartesi Machine.
    assert_eq!(claims.len(), N);
}

/// Send an input, an finish epoch, and another input.
/// After the second input is processed by the server-manager we know
/// for sure that the proxy finished processing the finish epoch.
/// We can't simply wait for the epoch to be finished because the proxy
/// still does tasks after that.
async fn finish_epoch_and_wait_for_next_input(state: &TestState<'_>) {
    tracing::info!("producing input, finish, and another input");
    let payload = generate_payload();
    let inputs = vec![
        RollupsData::AdvanceStateInput {
            input_metadata: InputMetadata {
                epoch_index: 0,
                input_index: 0,
                ..Default::default()
            },
            input_payload: vec![],
        },
        RollupsData::FinishEpoch {},
        RollupsData::AdvanceStateInput {
            input_metadata: InputMetadata {
                epoch_index: 1,
                input_index: 0,
                ..Default::default()
            },
            input_payload: payload.clone(),
        },
    ];
    for input in inputs {
        state.broker.produce_input_event(input).await;
    }

    tracing::info!("waiting until second input is processed");
    state.server_manager.assert_session_ready().await;
    state.server_manager.assert_epoch_status(0, 1).await;
    state
        .server_manager
        .assert_epoch_status_payloads(1, &vec![payload])
        .await;
}

#[test_log::test(tokio::test)]
async fn test_proxy_sends_inputs_after_finishing_epoch() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;
    finish_epoch_and_wait_for_next_input(&state).await;
}

#[test_log::test(tokio::test)]
async fn test_proxy_does_not_generate_duplicate_claim() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;

    tracing::info!("producing claim");
    let claim = [0xfa; HASH_SIZE];
    state.broker.produce_claim(claim.clone()).await;

    finish_epoch_and_wait_for_next_input(&state).await;

    tracing::info!("getting all claims");
    let produced_claims = state.broker.consume_all_claims().await;
    assert_eq!(produced_claims.len(), 1);
    assert_eq!(produced_claims[0].claim, claim);
}

#[test_log::test(tokio::test)]
async fn test_proxy_stores_snapshot_after_finishing_epoch() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;

    finish_epoch_and_wait_for_next_input(&state).await;

    tracing::info!("checking the snapshots dir");
    state.snapshots.assert_latest_snapshot(1);
}

#[test_log::test(tokio::test)]
async fn test_proxy_restore_session_after_restart() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;

    finish_epoch_and_wait_for_next_input(&state).await;

    tracing::info!("restarting proxy");
    state.proxy.restart().await;

    tracing::info!("producing another input and checking");
    let input = RollupsData::AdvanceStateInput {
        input_metadata: InputMetadata {
            epoch_index: 1,
            input_index: 1,
            ..Default::default()
        },
        input_payload: generate_payload(),
    };
    state.broker.produce_input_event(input).await;
    state.server_manager.assert_epoch_status(1, 2).await;
}
